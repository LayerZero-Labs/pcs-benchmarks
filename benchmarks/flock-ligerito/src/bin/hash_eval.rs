//! Single-shot Flock Ligerito Fast worker (packed bit-MLE).

use flock_core::challenger::FsChallenger;
use flock_core::field::F128;
use flock_core::merkle::HashKind;
use flock_core::pcs::commit::{commit, PcsParams};
use flock_core::pcs::ligerito::{
    embedded_initial_k_or_default, prover_config_for, verifier_config_for, LigeritoProfile,
};
use flock_core::pcs::ring_switch::{build_eq_split, split_n_lo};
use flock_core::pcs::{
    open_batch_mixed_ligerito_with_precomputed_s_hat_v_and_grinding,
    verify_opening_batch_ligerito_mixed_with_grinding, DirectEqInd, PackedDirectClaim,
    PackedDirectClaimRef,
};
use flock_core::zerocheck::PaddingSpec;
use pcs_bench_core::{RunStatus, WorkerOutput, FLOCK_LOG_PACKING};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

fn main() -> ExitCode {
    let threads = parse_u32_flag("--threads").unwrap_or(1).max(1);
    init_thread_pool(threads);
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = emit(&WorkerOutput {
                status: RunStatus::Error,
                status_detail: Some(error),
                log2_n: None,
                timings_ns: BTreeMap::new(),
                proof_bytes: None,
                commitment_bytes: None,
                evaluation_bytes: None,
                public_context_bytes: None,
                state_bytes: None,
                peak_rss_bytes: peak_rss_bytes(),
            });
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let m = parse_u32_flag("--log2-n")?;
    emit(&timed_ligerito(m)?)
}

fn timed_ligerito(m: u32) -> Result<WorkerOutput, String> {
    let m = m as usize;
    if m < FLOCK_LOG_PACKING as usize {
        return Err(format!("m={m} is below Flock packing width"));
    }
    let t0 = Instant::now();
    let profile = LigeritoProfile::Fast;
    let log_batch_size = embedded_initial_k_or_default(m, profile);
    let log_n = m - FLOCK_LOG_PACKING as usize;
    let params = PcsParams {
        m,
        log_inv_rate: profile.log_inv_rate(),
        log_batch_size,
        profile,
        num_lanes: None,
        merkle_hash: HashKind::Sha256,
    };
    let lig_p = prover_config_for(log_n, log_batch_size, profile).map_err(|e| e)?;
    let lig_v = verifier_config_for(log_n, log_batch_size, profile).map_err(|e| e)?;
    let setup_ns = elapsed_ns(t0);

    let mut rng = StdRng::seed_from_u64(configured_seed(0xF10C_0000 ^ m as u64));
    // Generate the declared packed-field witness directly. Uniform F128
    // elements are exactly uniformly random groups of 128 input bits, without
    // a byte-per-bit fixture that dominates process RSS.
    let packed: Vec<F128> = (0..(1usize << log_n))
        .map(|_| F128 {
            lo: rng.gen(),
            hi: rng.gen(),
        })
        .collect();
    let point: Vec<F128> = (0..log_n)
        .map(|_| F128 {
            lo: rng.gen(),
            hi: rng.gen(),
        })
        .collect();

    let t0 = Instant::now();
    let (commitment, prover_data) = commit(&packed, &params);
    let commit_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    // The supplied claim is useful point-dependent prover input, so its
    // computation belongs to end-to-end opening. Factor the equality tensor
    // to avoid a second full witness-sized allocation.
    let n_lo = split_n_lo(log_n);
    let (eq_lo, eq_hi) = build_eq_split(&point, n_lo);
    let low_mask = (1usize << n_lo) - 1;
    let value = packed
        .par_iter()
        .enumerate()
        .map(|(index, &coefficient)| coefficient * eq_lo[index & low_mask] * eq_hi[index >> n_lo])
        .reduce(|| F128::ZERO, |left, right| left + right);
    drop(eq_lo);
    drop(eq_hi);

    let claim = PackedDirectClaim {
        point: point.clone(),
        value,
        // The prover constructs the point-dependent factored basis inside the
        // timed opening phase.
        eq_ind: DirectEqInd::EqPoint(point.clone()),
    };
    let grinding = params.opening_grinding();
    let mut prover_ch = FsChallenger::new(b"akita-bench-flock");
    let proof = open_batch_mixed_ligerito_with_precomputed_s_hat_v_and_grinding(
        packed,
        &prover_data,
        &commitment,
        &[],
        &[],
        std::slice::from_ref(&claim),
        &PaddingSpec::dense(m),
        &lig_p,
        grinding,
        &mut prover_ch,
    );
    let open_ns = elapsed_ns(t0);

    let pd = PackedDirectClaimRef {
        point: &point,
        value,
    };
    let t0 = Instant::now();
    let mut verifier_ch = FsChallenger::new(b"akita-bench-flock");
    verify_opening_batch_ligerito_mixed_with_grinding(
        &commitment,
        &[],
        &[],
        &[],
        std::slice::from_ref(&pd),
        &proof,
        &lig_v,
        grinding,
        &mut verifier_ch,
    )
    .map_err(|error| format!("ligerito verify failed: {error:?}"))?;
    let verify_ns = elapsed_ns(t0);

    if negative_check_enabled() {
        let altered = PackedDirectClaimRef {
            point: &point,
            value: value + F128::ONE,
        };
        let mut negative_ch = FsChallenger::new(b"akita-bench-flock");
        if verify_opening_batch_ligerito_mixed_with_grinding(
            &commitment,
            &[],
            &[],
            &[],
            std::slice::from_ref(&altered),
            &proof,
            &lig_v,
            grinding,
            &mut negative_ch,
        )
        .is_ok()
        {
            return Err("Flock verifier accepted an altered opening claim".into());
        }
    }

    let proof_bytes = bincode::serialized_size(&proof).unwrap_or(0);
    let commitment_bytes = bincode::serialized_size(&commitment).unwrap_or(0);

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(format!(
            "flock-ligerito-fast,statement=packed-field-mle,input_bits=2^{m},variables={log_n},field=F128,batch={log_batch_size},hash=sha256"
        )),
        log2_n: Some(m as u32),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes: Some(commitment_bytes),
        evaluation_bytes: Some(bincode::serialized_size(&value).unwrap_or(0)),
        public_context_bytes: Some(0),
        state_bytes: Some(0),
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn init_thread_pool(threads: u32) {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1) as usize)
        .stack_size(64 * 1024 * 1024)
        .build_global();
}

fn configured_seed(domain: u64) -> u64 {
    std::env::var("PCS_BENCH_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(domain, |seed| seed ^ domain)
}

fn negative_check_enabled() -> bool {
    std::env::var("PCS_BENCH_NEGATIVE_CHECK").as_deref() == Ok("1")
}

fn parse_u32_flag(name: &str) -> Result<u32, String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == name {
            return args
                .next()
                .ok_or_else(|| format!("{name} requires a value"))?
                .parse()
                .map_err(|_| format!("invalid {name}"));
        }
        if let Some(value) = arg.strip_prefix(&format!("{name}=")) {
            return value.parse().map_err(|_| format!("invalid {name}"));
        }
    }
    Err(format!("missing {name}"))
}

fn elapsed_ns(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn emit(output: &WorkerOutput) -> Result<(), String> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, output).map_err(|error| error.to_string())?;
    stdout.write_all(b"\n").map_err(|error| error.to_string())
}

fn peak_rss_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        let Some(rest) = line.strip_prefix("VmHWM:") else {
            continue;
        };
        let kb: u64 = rest.split_whitespace().next()?.parse().ok()?;
        return Some(kb.saturating_mul(1024));
    }
    None
}
