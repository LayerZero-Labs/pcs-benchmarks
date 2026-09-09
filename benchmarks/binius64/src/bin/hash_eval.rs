//! Single-shot Binius64 BaseFold worker (unique decoding, 100-bit, SHA-256).

use binius_compute::GlobalAllocator;
use binius_field::{arch::OptimalPackedB128, Field};
use binius_hash::{StdDigest, StdHashSuite};
use binius_iop::basefold as verifier_basefold;
use binius_iop::channel::OracleSpec;
use binius_iop::fri::{calculate_n_test_queries, FRIParams};
use binius_iop::merkle_channel::{MerkleIPVerifierChannel, VerifierMerkleTranscriptChannel};
use binius_iop_prover::basefold::prove_mlecheck_basefold;
use binius_iop_prover::fri::{encode_interleaved, FRIFoldProver};
use binius_iop_prover::merkle_channel::{MerkleIPProverChannel, ProverMerkleTranscriptChannel};
use binius_iop_prover::merkle_tree::prover::BinaryMerkleTreeProver;
use binius_math::inner_product::inner_product_buffers;
use binius_math::multilinear::eq::eq_ind_partial_eval;
use binius_math::ntt::{domain_context::GaoMateerPreExpanded, NeighborsLastMultiThread};
use binius_math::test_utils::{random_field_buffer, random_scalars};
use binius_transcript::fiat_shamir::HasherChallenger;
use binius_transcript::ProverTranscript;
use pcs_bench_core::{RunStatus, WorkerOutput, HASH_SECURITY_BITS_100};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type F = binius_field::Ghash128b;
type P = OptimalPackedB128;
type StdChallenger = HasherChallenger<StdDigest>;
const LOG_INV_RATE: usize = 1;

fn main() -> ExitCode {
    let threads = parse_u32_flag("--threads").unwrap_or(1).max(1);
    init_thread_pool(threads);
    match run(threads) {
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

fn run(threads: u32) -> Result<(), String> {
    let log2_n = parse_u32_flag("--log2-n")?;
    emit(&timed_basefold(log2_n, threads)?)
}

fn timed_basefold(log2_n: u32, threads: u32) -> Result<WorkerOutput, String> {
    let n_vars = log2_n as usize;
    let n_test_queries = calculate_n_test_queries(HASH_SECURITY_BITS_100 as usize, LOG_INV_RATE);
    let mut rng = StdRng::seed_from_u64(configured_seed(0));
    let witness = random_field_buffer::<P>(&mut rng, n_vars);
    let evaluation_point: Vec<F> = random_scalars(&mut rng, n_vars);

    let merkle_prover = BinaryMerkleTreeProver::<F, StdHashSuite>::new();
    let t0 = Instant::now();
    let (fri_params, _) = FRIParams::optimal_for_batch(
        merkle_prover.scheme(),
        &[OracleSpec::new(n_vars)],
        LOG_INV_RATE,
        n_test_queries,
    );
    let domain_context = GaoMateerPreExpanded::generate(fri_params.log_len());
    let log_num_shares = threads.ilog2() as usize;
    let ntt = NeighborsLastMultiThread::new(domain_context, log_num_shares);
    let setup_ns = elapsed_ns(t0);

    let oracle_spec = &fri_params.input_oracles()[0];
    let leaf_width = 1usize << oracle_spec.log_batch_size();
    // Per-oracle Merkle depth: the committed codeword is this oracle's RS codeword,
    // not the batched first-round `FRIParams::log_len()` (those differ when
    // `optimal_for_batch` chooses `log_batch_size > 0`).
    let merkle_depth = (fri_params.rs_code().log_dim() - oracle_spec.log_lift)
        + fri_params.rs_code().log_inv_rate();

    let t0 = Instant::now();
    let codeword = encode_interleaved(&fri_params, 0, &ntt, witness.as_view(), &GlobalAllocator);
    let mut prover_transcript = ProverTranscript::new(StdChallenger::default());
    let codeword_commitment = {
        let mut prover_channel =
            ProverMerkleTranscriptChannel::<_, StdChallenger, _, StdHashSuite>::with_merkle_prover(
                &mut prover_transcript,
                merkle_prover,
            );
        prover_channel.send_merkle_commitment(codeword.as_view(), leaf_width)
    };
    let commit_ns = elapsed_ns(t0);
    let commitment_bytes = prover_transcript.clone().finalize().len() as u64;

    // The evaluation is supplied to the prover and is point-dependent, so it
    // is part of the end-to-end opening interval.
    let t0 = Instant::now();
    let eval_point_eq = eq_ind_partial_eval::<P>(&evaluation_point);
    let eval_claim = inner_product_buffers(&witness, &eval_point_eq);

    let mut prover_channel =
        ProverMerkleTranscriptChannel::<_, StdChallenger, _, StdHashSuite>::with_merkle_prover(
            &mut prover_transcript,
            BinaryMerkleTreeProver::<F, StdHashSuite>::new(),
        );
    let fri_folder =
        FRIFoldProver::new_batch(&fri_params, &ntt, vec![(codeword, codeword_commitment)]);
    prove_mlecheck_basefold(
        witness,
        &evaluation_point,
        eval_claim,
        None,
        &[],
        fri_folder,
        &mut prover_channel,
        &GlobalAllocator,
    );
    let open_ns = elapsed_ns(t0);
    prover_channel.into_transcript();
    let transcript_bytes = prover_transcript.clone().finalize().len() as u64;
    let proof_bytes = transcript_bytes.saturating_sub(commitment_bytes);
    let negative_source = negative_check_enabled().then(|| prover_transcript.clone());
    let t0 = Instant::now();
    let mut verifier_transcript = prover_transcript.into_verifier();
    let mut verifier_channel =
        VerifierMerkleTranscriptChannel::<_, StdChallenger, _, StdHashSuite>::new(
            &mut verifier_transcript,
        );
    let retrieved = verifier_channel
        .recv_merkle_commitment(leaf_width, merkle_depth)
        .map_err(|error| error.to_string())?;

    verifier_basefold::verify_mlecheck_basefold(
        &fri_params,
        &[retrieved],
        eval_claim,
        &evaluation_point,
        None,
        &[],
        &mut verifier_channel,
    )
    .map_err(|error| error.to_string())?;
    let verify_ns = elapsed_ns(t0);

    if let Some(negative_source) = negative_source {
        let mut negative_transcript = negative_source.into_verifier();
        let mut negative_channel =
            VerifierMerkleTranscriptChannel::<_, StdChallenger, _, StdHashSuite>::new(
                &mut negative_transcript,
            );
        let altered_commitment = negative_channel
            .recv_merkle_commitment(leaf_width, merkle_depth)
            .map_err(|error| error.to_string())?;
        if verifier_basefold::verify_mlecheck_basefold(
            &fri_params,
            &[altered_commitment],
            eval_claim + F::ONE,
            &evaluation_point,
            None,
            &[],
            &mut negative_channel,
        )
        .is_ok()
        {
            return Err("Binius verifier accepted an altered opening claim".into());
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
            "binius64-basefold-udr-100,statement=multilinear,distribution=full-field-uniform,point=full-field-uniform,rate=1/2,queries={n_test_queries},hash=std,packed=arch-optimal,ntt=multithread,shares={}",
            1u32 << log_num_shares
        )),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes: Some(commitment_bytes),
        evaluation_bytes: Some(std::mem::size_of::<F>() as u64),
        public_context_bytes: Some(0),
        // Gao–Mateer twiddles are reusable, but the pinned API does not expose
        // their retained allocation size.
        state_bytes: None,
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
