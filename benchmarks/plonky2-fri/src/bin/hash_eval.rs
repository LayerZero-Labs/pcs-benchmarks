//! Single-shot Plonky2 univariate FRI worker (Goldilocks, Poseidon2).

use pcs_bench_core::{
    RunStatus, WorkerOutput, PLONKY2_CAP_HEIGHT, PLONKY2_FRI_POW_BITS, PLONKY2_FRI_QUERIES,
    PLONKY2_FRI_RATE_BITS,
};
use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::field::types::Field;
use plonky2::fri::oracle::PolynomialBatch;
use plonky2::fri::reduction_strategies::FriReductionStrategy;
use plonky2::fri::structure::{
    FriBatchInfo, FriInstanceInfo, FriOpeningBatch, FriOpenings, FriOracleInfo, FriPolynomialInfo,
};
use plonky2::fri::verifier::verify_fri_proof;
use plonky2::fri::FriConfig;
use plonky2::iop::challenger::Challenger;
use plonky2::plonk::config::{GenericConfig, Poseidon2GoldilocksConfig};
use plonky2::util::timing::TimingTree;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type C = Poseidon2GoldilocksConfig;
type F = GoldilocksField;
const D: usize = 2;

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
    let log2_n = parse_u32_flag("--log2-n")?;
    emit(&timed_fri(log2_n)?)
}

fn timed_fri(log2_n: u32) -> Result<WorkerOutput, String> {
    let degree_bits = log2_n as usize;
    let n = 1usize << degree_bits;
    let mut rng = StdRng::seed_from_u64(configured_seed(0));
    let values = PolynomialValues::new((0..n).map(|_| random_goldilocks(&mut rng)).collect());

    let t0 = Instant::now();
    let fri_config = FriConfig {
        rate_bits: PLONKY2_FRI_RATE_BITS,
        cap_height: PLONKY2_CAP_HEIGHT,
        proof_of_work_bits: PLONKY2_FRI_POW_BITS as u32,
        reduction_strategy: FriReductionStrategy::ConstantArityBits(4, 5),
        num_query_rounds: PLONKY2_FRI_QUERIES,
    };
    let fri_params = fri_config.fri_params(degree_bits, false);
    let mut timing = TimingTree::default();
    let setup_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    let oracle = PolynomialBatch::<F, C, D>::from_values(
        vec![values],
        fri_config.rate_bits,
        false,
        fri_config.cap_height,
        &mut timing,
        None,
    );
    let commit_ns = elapsed_ns(t0);

    // End-to-end opening starts with transcript reconstruction and includes
    // deriving and evaluating the point whose claim is supplied to FRI.
    let t0 = Instant::now();
    let mut prover_challenger = Challenger::<F, <C as GenericConfig<D>>::Hasher>::new();
    prover_challenger.observe_cap(&oracle.merkle_tree.cap);
    let zeta = prover_challenger.get_extension_challenge::<D>();
    let claimed = oracle.polynomials[0].to_extension::<D>().eval(zeta);

    let instance = FriInstanceInfo {
        oracles: vec![FriOracleInfo {
            num_polys: 1,
            blinding: false,
        }],
        batches: vec![FriBatchInfo {
            point: zeta,
            polynomials: vec![FriPolynomialInfo {
                oracle_index: 0,
                polynomial_index: 0,
            }],
        }],
    };
    let openings = FriOpenings {
        batches: vec![FriOpeningBatch {
            values: vec![claimed],
        }],
    };

    // Same transcript order as `plonk::prover`: bind the claimed openings before FRI.
    prover_challenger.observe_openings(&openings);

    let proof = PolynomialBatch::<F, C, D>::prove_openings(
        &instance,
        &[&oracle],
        &mut prover_challenger,
        &fri_params,
        None,
        None,
        &mut timing,
    );
    let open_ns = elapsed_ns(t0);

    // Complete verification includes transcript reconstruction and all
    // proof-dependent Fiat–Shamir challenge derivation.
    let t0 = Instant::now();
    let mut verifier_challenger = Challenger::<F, <C as GenericConfig<D>>::Hasher>::new();
    verifier_challenger.observe_cap(&oracle.merkle_tree.cap);
    let zeta_v = verifier_challenger.get_extension_challenge::<D>();
    if zeta_v != zeta {
        return Err("verifier zeta drifted from prover".into());
    }
    verifier_challenger.observe_openings(&openings);
    let challenges = verifier_challenger.fri_challenges::<C, D>(
        &proof.commit_phase_merkle_caps,
        &proof.final_poly,
        proof.pow_witness,
        degree_bits,
        &fri_config,
        None,
        None,
    );

    verify_fri_proof::<F, C, D>(
        &instance,
        &openings,
        &challenges,
        &[oracle.merkle_tree.cap.clone()],
        &proof,
        &fri_params,
    )
    .map_err(|error| error.to_string())?;
    let verify_ns = elapsed_ns(t0);

    if negative_check_enabled() {
        let altered_openings = FriOpenings {
            batches: vec![FriOpeningBatch {
                values: vec![
                    claimed
                        + <<F as Extendable<D>>::Extension as FieldExtension<D>>::from_basefield(
                            F::ONE,
                        ),
                ],
            }],
        };
        let mut negative_challenger = Challenger::<F, <C as GenericConfig<D>>::Hasher>::new();
        negative_challenger.observe_cap(&oracle.merkle_tree.cap);
        let _ = negative_challenger.get_extension_challenge::<D>();
        negative_challenger.observe_openings(&altered_openings);
        let altered_challenges = negative_challenger.fri_challenges::<C, D>(
            &proof.commit_phase_merkle_caps,
            &proof.final_poly,
            proof.pow_witness,
            degree_bits,
            &fri_config,
            None,
            None,
        );
        if verify_fri_proof::<F, C, D>(
            &instance,
            &altered_openings,
            &altered_challenges,
            &[oracle.merkle_tree.cap.clone()],
            &proof,
            &fri_params,
        )
        .is_ok()
        {
            return Err("Plonky2 verifier accepted an altered opening claim".into());
        }
    }

    let proof_bytes = bincode_len(&proof);
    let commitment_bytes = bincode_len(&oracle.merkle_tree.cap);

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(format!(
            "plonky2-fri-100,statement=univariate,distribution=full-field-uniform,point=transcript-extension,rate=1/{},queries={},pow_bits={}",
            1usize << PLONKY2_FRI_RATE_BITS,
            PLONKY2_FRI_QUERIES,
            PLONKY2_FRI_POW_BITS
        )),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes,
        commitment_bytes,
        evaluation_bytes: bincode_len(&claimed),
        public_context_bytes: Some(0),
        state_bytes: Some(0),
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn bincode_len<T: serde::Serialize>(value: &T) -> Option<u64> {
    bincode::serialized_size(value).ok()
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

fn random_goldilocks(rng: &mut StdRng) -> F {
    const MODULUS: u64 = 0xffff_ffff_0000_0001;
    loop {
        let candidate = rng.gen::<u64>();
        if candidate < MODULUS {
            return F::from_canonical_u64(candidate);
        }
    }
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
