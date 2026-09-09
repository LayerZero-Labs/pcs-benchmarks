//! Single-shot dense BaseFold worker (SP1 SLOP stacked BaseFold).

use pcs_bench_core::{
    RunStatus, WorkerOutput, BASEFOLD_FRI_LOG_BLOWUP, BASEFOLD_FRI_POW_BITS, BASEFOLD_FRI_QUERIES,
    BASEFOLD_LOG_STACKING_HEIGHT,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use slop_algebra::extension::BinomialExtensionField;
use slop_algebra::AbstractField;
use slop_basefold::{BasefoldVerifier, FriConfig};
use slop_basefold_prover::BasefoldProver;
use slop_challenger::{CanObserve, IopCtx};
use slop_commit::{Message, Rounds};
use slop_koala_bear::{KoalaBear, KoalaBearDegree4Duplex};
use slop_merkle_tree::Poseidon2KoalaBear16Prover;
use slop_multilinear::{Mle, MultilinearPcsProver, Point};
use slop_stacked::{StackedPcsProver, StackedPcsVerifier};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type GC = KoalaBearDegree4Duplex;
type F = KoalaBear;
type EF = BinomialExtensionField<KoalaBear, 4>;
type PcsProver = BasefoldProver<GC, Poseidon2KoalaBear16Prover>;

const INPUT_SEED: u64 = 0xDEAD_BEEF;
const POINT_SEED: u64 = 0xCAFE_BABE;

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
    if log2_n < BASEFOLD_LOG_STACKING_HEIGHT {
        return Err(format!(
            "log2_n={log2_n} is below stacking height {BASEFOLD_LOG_STACKING_HEIGHT}"
        ));
    }
    let output = timed_basefold(log2_n)?;
    emit(&output)
}

#[allow(clippy::too_many_lines)]
fn timed_basefold(log2_n: u32) -> Result<WorkerOutput, String> {
    let num_variables = log2_n as usize;
    let log_stacking_height = BASEFOLD_LOG_STACKING_HEIGHT;
    let batch_size = 1usize << (log2_n - log_stacking_height);

    let evals = dense_evaluations(num_variables);
    let mle = Mle::<F>::from(evals);
    let point = opening_point(num_variables);
    let messages: Message<Mle<F>> = Message::from(mle);
    let claim_source = messages[0].clone();

    let t0 = Instant::now();
    let fri_config = FriConfig::new(
        BASEFOLD_FRI_LOG_BLOWUP,
        BASEFOLD_FRI_QUERIES,
        BASEFOLD_FRI_POW_BITS,
    );
    let pcs_verifier = BasefoldVerifier::<GC>::new(fri_config, 1);
    let pcs_prover = PcsProver::new(&pcs_verifier);
    let verifier = StackedPcsVerifier::new(pcs_verifier, log_stacking_height);
    let prover = StackedPcsProver::new(pcs_prover, log_stacking_height, batch_size);
    let setup_ns = elapsed_ns(t0);

    let mut prover_challenger = GC::default_challenger();
    let t0 = Instant::now();
    let (commitment, prover_data, _) = prover
        .commit_multilinears(messages)
        .map_err(|error| error.to_string())?;
    prover_challenger.observe(commitment);
    let commit_ns = elapsed_ns(t0);

    let mut prover_data_rounds = Rounds::new();
    prover_data_rounds.push(prover_data);

    let t0 = Instant::now();
    let evaluation_claim = claim_source.eval_at(&point)[0];
    drop(claim_source);
    let proof = prover
        .prove_trusted_evaluation(
            point.clone(),
            evaluation_claim,
            prover_data_rounds,
            &mut prover_challenger,
        )
        .map_err(|error| error.to_string())?;
    let open_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    let mut verifier_challenger = GC::default_challenger();
    verifier_challenger.observe(commitment);
    verifier
        .verify_trusted_evaluation(
            &[commitment],
            &[1usize << log2_n],
            &point,
            &proof,
            evaluation_claim,
            &mut verifier_challenger,
        )
        .map_err(|error| error.to_string())?;
    let verify_ns = elapsed_ns(t0);

    if negative_check_enabled() {
        let mut negative_challenger = GC::default_challenger();
        negative_challenger.observe(commitment);
        if verifier
            .verify_trusted_evaluation(
                &[commitment],
                &[1usize << log2_n],
                &point,
                &proof,
                evaluation_claim + EF::one(),
                &mut negative_challenger,
            )
            .is_ok()
        {
            return Err("BaseFold verifier accepted an altered opening claim".into());
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
        status_detail: Some(
            "statement=multilinear,distribution=full-field-uniform,point=full-extension-uniform"
                .into(),
        ),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes: Some(commitment_bytes),
        evaluation_bytes: Some(bincode::serialized_size(&evaluation_claim).unwrap_or(0)),
        public_context_bytes: Some(0),
        state_bytes: None,
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn dense_evaluations(num_vars: usize) -> Vec<F> {
    let mut rng = StdRng::seed_from_u64(configured_seed(INPUT_SEED));
    (0..(1usize << num_vars)).map(|_| rng.gen()).collect()
}

fn opening_point(num_vars: usize) -> Point<EF> {
    let mut rng = StdRng::seed_from_u64(configured_seed(POINT_SEED));
    Point::<EF>::rand(&mut rng, num_vars as u32)
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

fn init_thread_pool(threads: u32) {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1) as usize)
        .stack_size(64 * 1024 * 1024)
        .build_global();
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
