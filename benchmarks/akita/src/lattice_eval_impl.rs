use akita_algebra::poly::multilinear_eval;
use akita_config::proof_optimized::fp32;
use akita_config::CommitmentConfig;
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{ComputeBackendSetup, CpuBackend, DensePoly, SelectedProverOpeningData};
use akita_serialization::{AkitaSerialize, Compress};
use akita_transcript::AkitaTranscript;
use akita_types::{
    AkitaCommitmentHint, BasisMode, CommittedGroup, CommittedGroupBatchProfile,
    GroupBatchStatement, OpeningClaims, OpeningClaimsLayout, OpeningScheduleSelection,
    PolynomialGroupClaims,
};
use jolt_field::{ExtField, Ring};
use pcs_bench_core::{RunStatus, WorkerOutput};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type Cfg = fp32::Dense;
type F = fp32::Field;
type E = fp32::ExtensionField;

const INPUT_SEED: u64 = 0xDEAD_BEEF;
const POINT_SEED: u64 = 0xCAFE_BABE;
const TRANSCRIPT_DOMAIN: &[u8] = b"pcs-benchmark/lattice-eval/v1";

fn main() -> ExitCode {
    init_single_thread_pool();
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
                state_bytes: None,
                peak_rss_bytes: peak_rss_bytes(),
            });
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let log2_n = parse_u32_flag("--log2-n")?;
    let _payload_log2 = parse_u32_flag("--payload-log2").ok();

    if OpeningClaimsLayout::new(log2_n as usize, 1)
        .ok()
        .and_then(|layout| Cfg::resolve_catalog_row_for_opening(&layout).ok())
        .is_none()
    {
        emit(&WorkerOutput {
            status: RunStatus::Unsupported,
            status_detail: Some(format!(
                "pinned Akita fp32 dense catalog has no row for nv={log2_n}"
            )),
            log2_n: Some(log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: None,
            peak_rss_bytes: peak_rss_bytes(),
        })?;
        return Ok(());
    }

    let output = std::thread::Builder::new()
        .name("akita-lattice-eval".into())
        .stack_size(256 * 1024 * 1024)
        .spawn(move || timed_dense(log2_n))
        .map_err(|error| error.to_string())?
        .join()
        .map_err(|_| "Akita worker thread panicked".to_owned())??;

    emit(&output)
}

#[allow(clippy::too_many_lines)]
fn timed_dense(log2_n: u32) -> Result<WorkerOutput, String> {
    let num_vars = log2_n as usize;
    let evaluations = dense_evaluations(num_vars);
    let polynomial = DensePoly::<F>::from_field_evals(num_vars, &evaluations)
        .map_err(|error| error.to_string())?;
    let point = opening_point(num_vars);
    let ext_evals: Vec<E> = evaluations.iter().copied().map(E::lift_base).collect();
    let opening = multilinear_eval(&ext_evals, &point).map_err(|error| error.to_string())?;

    let t0 = Instant::now();
    let setup = AkitaCommitmentScheme::<Cfg>::setup_prover(num_vars, 1)
        .map_err(|error| error.to_string())?;
    let setup_ns = elapsed_ns(t0);

    let prepared = CpuBackend::DEFAULT
        .prepare_setup(&setup)
        .map_err(|error| error.to_string())?;
    let stack = akita_prover::UniformProverStack::uniform(
        &CpuBackend::DEFAULT,
        &prepared,
        setup.expanded.as_ref(),
    )
    .map_err(|error| error.to_string())?;

    let t0 = Instant::now();
    let commit_output = AkitaCommitmentScheme::<Cfg>::commit::<_, _>(
        &setup,
        std::slice::from_ref(&polynomial),
        &stack,
        akita_prover::GroupContext::scheduler_without_precommitted_groups(),
    )
    .map_err(|error| error.to_string())?;
    let commit_ns = elapsed_ns(t0);

    let polynomial_refs = [&polynomial];
    let selection = Cfg::resolve_catalog_row_for_profiles(&CommittedGroupBatchProfile {
        final_group: *commit_output.committed_group.profile(),
        precommitteds: Vec::new(),
    })
    .map_err(|error| error.to_string())?
    .selection();
    let verifier_setup =
        AkitaCommitmentScheme::<Cfg>::setup_verifier(&setup).map_err(|error| error.to_string())?;

    let t0 = Instant::now();
    let mut prover_transcript = AkitaTranscript::<F>::new(TRANSCRIPT_DOMAIN);
    let proof = AkitaCommitmentScheme::<Cfg>::batched_prove::<_, _, _>(
        &setup,
        prover_claims(
            &point,
            &polynomial_refs,
            &commit_output.committed_group,
            commit_output.hint.clone(),
        )?,
        &stack,
        &mut prover_transcript,
        BasisMode::Lagrange,
    )
    .map_err(|error| error.to_string())?;
    let open_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    let mut verifier_transcript = AkitaTranscript::<F>::new(TRANSCRIPT_DOMAIN);
    AkitaCommitmentScheme::<Cfg>::batched_verify(
        &proof,
        &verifier_setup,
        &mut verifier_transcript,
        verifier_claims(
            selection,
            &point,
            &[opening],
            &commit_output.committed_group,
        )?,
        BasisMode::Lagrange,
    )
    .map_err(|error| error.to_string())?;
    let verify_ns = elapsed_ns(t0);

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: None,
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof.serialized_size(Compress::No) as u64),
        commitment_bytes: Some(commit_output.committed_group.serialized_size(Compress::No) as u64),
        state_bytes: Some(setup.expanded.serialized_size(Compress::No) as u64),
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn prover_claims<'a, P>(
    point: &'a [E],
    polynomials: &'a [&'a P],
    commitment: &'a CommittedGroup<F>,
    hint: AkitaCommitmentHint<F>,
) -> Result<SelectedProverOpeningData<'a, E, akita_prover::PreparedProverGroup<'a, P>, F>, String>
where
    P: akita_prover::RootPolyMeta<F>,
{
    let group = PolynomialGroupClaims::new(
        point.to_vec(),
        vec![E::from_u64(0); polynomials.len()],
        commitment.clone(),
    )
    .map_err(|error| error.to_string())?;
    let claims = OpeningClaims::from_groups(vec![group]).map_err(|error| error.to_string())?;
    SelectedProverOpeningData::from_committed_claims::<Cfg>(claims, vec![hint], vec![polynomials])
        .map_err(|error| error.to_string())
}

fn verifier_claims<'a>(
    selection: OpeningScheduleSelection,
    point: &[E],
    openings: &[E],
    commitment: &'a CommittedGroup<F>,
) -> Result<GroupBatchStatement<'a, E, F>, String> {
    let group = PolynomialGroupClaims::new(point.to_vec(), openings.to_vec(), commitment)
        .map_err(|error| error.to_string())?;
    let claims = OpeningClaims::from_groups(vec![group]).map_err(|error| error.to_string())?;
    GroupBatchStatement::new(selection, claims).map_err(|error| error.to_string())
}

fn dense_evaluations(num_vars: usize) -> Vec<F> {
    let mut rng = StdRng::seed_from_u64(INPUT_SEED);
    let half_bound = 1i64 << 30;
    (0..(1usize << num_vars))
        .map(|_| F::from_i64(rng.gen_range(-half_bound..half_bound)))
        .collect()
}

fn opening_point(num_vars: usize) -> Vec<E> {
    let mut rng = StdRng::seed_from_u64(POINT_SEED);
    (0..num_vars)
        .map(|_| E::from_u64(rng.gen::<u64>()))
        .collect()
}

fn init_single_thread_pool() {
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
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
