//! Single-shot ProveKit WHIR worker (Goldilocks base-field coeffs, deg-3 challenges).

use pcs_bench_core::{
    RunStatus, WorkerOutput, PROVEKIT_SECURITY_BITS, PROVEKIT_WHIR_FOLD, PROVEKIT_WHIR_LOG_INV_RATE,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;
use whir::algebra::embedding::Basefield;
use whir::algebra::fields::{Field64, Field64_3};
use whir::algebra::linear_form::{Evaluate, LinearForm, MultilinearExtension};
use whir::buffer::Buffer;
use whir::hash;
use whir::parameters::ProtocolParameters;
use whir::protocols::params::DecodingRegime;
use whir::protocols::whir::Config;
use whir::transcript::codecs::Empty;
use whir::transcript::{DomainSeparator, NargSerialize, ProverState, VerifierState};

type M = Basefield<Field64_3>;

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
    emit(&timed_whir(log2_n)?)
}

fn timed_whir(log2_n: u32) -> Result<WorkerOutput, String> {
    let num_variables = log2_n as usize;
    let num_coeffs = 1usize << num_variables;
    let t0 = Instant::now();
    let whir_params = ProtocolParameters {
        security_level: PROVEKIT_SECURITY_BITS as usize,
        pow_bits: 20,
        initial_folding_factor: PROVEKIT_WHIR_FOLD,
        folding_factor: PROVEKIT_WHIR_FOLD,
        decoding_regime: DecodingRegime::Johnson,
        starting_log_inv_rate: PROVEKIT_WHIR_LOG_INV_RATE,
        batch_size: 1,
        hash_id: hash::SHA2,
    };
    let params = Config::<M>::new(num_coeffs, &whir_params);

    let ds = DomainSeparator::protocol(&params)
        .session(&"akita-benchmark whir-provekit".to_owned())
        .instance(&Empty);
    let mut prover_state = ProverState::new_std(&ds);
    let setup_ns = elapsed_ns(t0);

    let mut rng = StdRng::seed_from_u64(configured_seed(0));
    let vector: Vec<Field64> = (0..num_coeffs)
        .map(|_| random_goldilocks(&mut rng))
        .collect();
    let vector_buffer = Buffer::from(vector.as_slice());

    let point: Vec<<M as whir::algebra::embedding::Embedding>::Target> = (0..num_variables)
        .map(|_| {
            Field64_3::new(
                random_goldilocks(&mut rng),
                random_goldilocks(&mut rng),
                random_goldilocks(&mut rng),
            )
        })
        .collect();
    let linear_form = MultilinearExtension::new(point);

    let t0 = Instant::now();
    let witness = params.commit(&mut prover_state, &[&vector_buffer]);
    let commit_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    let evaluation = linear_form.evaluate(params.embedding(), &vector);
    let _ = params.prove(
        &mut prover_state,
        &[&vector_buffer],
        vec![&witness],
        vec![Box::new(linear_form.clone())],
        Buffer::from(std::slice::from_ref(&evaluation)),
    );
    let open_ns = elapsed_ns(t0);
    let proof = prover_state.proof();
    let mut root_narg = Vec::new();
    hash::Hash::default().serialize_into_narg(&mut root_narg);
    let mut ood_narg = Vec::new();
    evaluation.serialize_into_narg(&mut ood_narg);
    let commitment_bytes = (root_narg.len()
        + params.initial_out_domain_samples
            * params.initial_committer.num_vectors()
            * ood_narg.len()) as u64;
    let transcript_and_hints = (proof.narg_string.len() + proof.hints.len()) as u64;
    let proof_bytes = transcript_and_hints.saturating_sub(commitment_bytes);

    let t0 = Instant::now();
    let mut verifier_state = VerifierState::new_std(&ds, &proof);
    let commitment = params
        .receive_commitment(&mut verifier_state)
        .map_err(|error| format!("receive commitment: {error:?}"))?;
    let final_claim = params
        .verify(&mut verifier_state, &[&commitment], &[evaluation])
        .map_err(|error| format!("verify: {error:?}"))?;
    final_claim
        .verify(std::iter::once(
            &linear_form as &dyn LinearForm<<M as whir::algebra::embedding::Embedding>::Target>,
        ))
        .map_err(|error| format!("final claim: {error:?}"))?;
    let verify_ns = elapsed_ns(t0);

    if negative_check_enabled() {
        let mut negative_state = VerifierState::new_std(&ds, &proof);
        let negative_commitment = params
            .receive_commitment(&mut negative_state)
            .map_err(|error| format!("negative receive commitment: {error:?}"))?;
        let altered_evaluation =
            evaluation + <M as whir::algebra::embedding::Embedding>::Target::from(1u64);
        if let Ok(altered_final_claim) = params.verify(
            &mut negative_state,
            &[&negative_commitment],
            &[altered_evaluation],
        ) {
            if altered_final_claim
                .verify(std::iter::once(
                    &linear_form
                        as &dyn LinearForm<<M as whir::algebra::embedding::Embedding>::Target>,
                ))
                .is_ok()
            {
                return Err("ProveKit WHIR verifier accepted an altered opening claim".into());
            }
        }
    }

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(format!(
            "statement=multilinear,distribution=full-field-uniform,point=full-extension-uniform,evaluation=separate,goldilocks3,johnson,rate=1/{},fold={},pow_bits={},security={}",
            1usize << whir_params.starting_log_inv_rate,
            whir_params.folding_factor,
            whir_params.pow_bits,
            whir_params.security_level
        )),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes: Some(commitment_bytes),
        evaluation_bytes: Some(ood_narg.len() as u64),
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

fn random_goldilocks(rng: &mut StdRng) -> Field64 {
    const MODULUS: u64 = u64::MAX - (1u64 << 32) + 2;
    loop {
        let candidate = rng.gen::<u64>();
        if candidate < MODULUS {
            return Field64::from(candidate);
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
