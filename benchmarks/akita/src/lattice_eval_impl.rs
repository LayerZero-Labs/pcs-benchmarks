use akita_config::proof_optimized::{fp128, fp32, fp64};
use akita_config::{CommitmentConfig, RecursiveCommitmentConfig};
use akita_pcs::AkitaCommitmentScheme;
use akita_prover::{ComputeBackendSetup, CpuBackend, DensePoly, SelectedProverOpeningData};
use akita_serialization::{AkitaDeserialize, AkitaSerialize, Compress, Valid};
use akita_transcript::AkitaTranscript;
use akita_types::{
    AkitaCommitmentHint, BasisMode, CommittedGroup, CommittedGroupBatchProfile, FpExtEncoding,
    GroupBatchStatement, OpeningClaims, OpeningClaimsLayout, OpeningScheduleSelection,
    PolynomialGroupClaims,
};
use jolt_field::{Field, Fold, One, PseudoMersenne, Ring, Unreduced};
use pcs_bench_core::{RunStatus, WorkerOutput};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type DirectCfg = fp32::Dense;
type OffloadCfg = RecursiveCommitmentConfig<fp32::Dense>;
type ProverOpeningData<'a, Cfg, P> = SelectedProverOpeningData<
    'a,
    <Cfg as CommitmentConfig>::ExtField,
    akita_prover::PreparedProverGroup<'a, P>,
    <Cfg as CommitmentConfig>::Field,
>;

const INPUT_SEED: u64 = 0xDEAD_BEEF;
const POINT_SEED: u64 = 0xCAFE_BABE;
const TRANSCRIPT_DOMAIN: &[u8] = b"pcs-benchmark/lattice-eval/v1";

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
    let _payload_log2 = parse_u32_flag("--payload-log2").ok();
    let offload = has_flag("--offload");
    let field = parse_string_flag("--field").unwrap_or_else(|| "fp32".into());
    match (field.as_str(), offload) {
        ("fp32", true) => run_cfg::<OffloadCfg>(log2_n, true, "fp32-dense-recursive"),
        ("fp32", false) => run_cfg::<DirectCfg>(log2_n, false, "fp32 dense"),
        ("fp64", false) => run_cfg::<fp64::Dense>(log2_n, false, "fp64 dense"),
        ("fp128", false) => run_cfg::<fp128::Dense>(log2_n, false, "fp128 dense"),
        ("fp64" | "fp128", true) => {
            Err(format!("--offload is only supported for fp32, not {field}"))
        }
        (other, _) => Err(format!(
            "unknown --field {other} (expected fp32, fp64, or fp128)"
        )),
    }
}

fn run_cfg<Cfg>(log2_n: u32, offload: bool, catalog: &'static str) -> Result<(), String>
where
    Cfg: CommitmentConfig,
    Cfg::Field:
        Field + Unreduced + PseudoMersenne + Valid + AkitaSerialize + AkitaDeserialize<Context = ()>,
    Cfg::ExtField: Field + Unreduced + Fold + AkitaSerialize + FpExtEncoding<Cfg::Field>,
{
    let resolved = OpeningClaimsLayout::new(log2_n as usize, 1)
        .ok()
        .and_then(|layout| Cfg::resolve_catalog_row_for_opening(&layout).ok());
    let Some(resolved) = resolved else {
        emit(&WorkerOutput {
            status: RunStatus::Unsupported,
            status_detail: Some(format!(
                "pinned Akita {catalog} catalog has no row for nv={log2_n}"
            )),
            log2_n: Some(log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            evaluation_bytes: None,
            public_context_bytes: None,
            state_bytes: None,
            peak_rss_bytes: peak_rss_bytes(),
        })?;
        return Ok(());
    };
    if offload {
        let offload_edges = resolved
            .schedule()
            .recursive_folds
            .iter()
            .filter(|fold| fold.params.setup_prefix().is_some())
            .count();
        if offload_edges == 0 {
            emit(&WorkerOutput {
                status: RunStatus::Unsupported,
                status_detail: Some(format!(
                    "pinned Akita {catalog} catalog row for nv={log2_n} has no setup-prefix edges"
                )),
                log2_n: Some(log2_n),
                timings_ns: BTreeMap::new(),
                proof_bytes: None,
                commitment_bytes: None,
                evaluation_bytes: None,
                public_context_bytes: None,
                state_bytes: None,
                peak_rss_bytes: peak_rss_bytes(),
            })?;
            return Ok(());
        }
    }

    let output = std::thread::Builder::new()
        .name("akita-lattice-eval".into())
        .stack_size(256 * 1024 * 1024)
        .spawn(move || timed_dense::<Cfg>(log2_n, catalog))
        .map_err(|error| error.to_string())?
        .join()
        .map_err(|_| "Akita worker thread panicked".to_owned())??;

    emit(&output)
}

#[allow(clippy::too_many_lines)]
fn timed_dense<Cfg>(log2_n: u32, catalog: &'static str) -> Result<WorkerOutput, String>
where
    Cfg: CommitmentConfig,
    Cfg::Field:
        Field + Unreduced + PseudoMersenne + Valid + AkitaSerialize + AkitaDeserialize<Context = ()>,
    Cfg::ExtField: Field + Unreduced + Fold + AkitaSerialize + FpExtEncoding<Cfg::Field>,
{
    let num_vars = log2_n as usize;
    let evaluations = dense_evaluations::<Cfg::Field>(num_vars);
    let point = opening_point::<Cfg::ExtField>(num_vars);
    // Same as Akita's dense profile workload: transfer the eval buffer, then
    // derive the extension opening from those base-field coefficients.
    let polynomial = DensePoly::<Cfg::Field>::from_field_evals(num_vars, evaluations)
        .map_err(|error| error.to_string())?;
    let live = 1usize << num_vars;
    let opening = akita_types::derive_tensor_extension_opening_claim::<Cfg::Field, Cfg::ExtField>(
        num_vars,
        &polynomial.field_coeffs()[..live],
        &point,
    )
    .map_err(|error| error.to_string())?
    .0;

    let t0 = Instant::now();
    let setup = AkitaCommitmentScheme::<Cfg>::setup_prover(num_vars, 1)
        .map_err(|error| error.to_string())?;
    let prepared = CpuBackend::DEFAULT
        .prepare_setup(&setup)
        .map_err(|error| error.to_string())?;
    let stack = akita_prover::UniformProverStack::uniform(
        &CpuBackend::DEFAULT,
        &prepared,
        setup.expanded.as_ref(),
    )
    .map_err(|error| error.to_string())?;
    let verifier_setup =
        AkitaCommitmentScheme::<Cfg>::setup_verifier(&setup).map_err(|error| error.to_string())?;
    let setup_ns = elapsed_ns(t0);

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
    let resolved = Cfg::resolve_catalog_row_for_profiles(&CommittedGroupBatchProfile {
        final_group: *commit_output.committed_group.profile(),
        precommitteds: Vec::new(),
    })
    .map_err(|error| error.to_string())?;
    let offload_edges = resolved
        .schedule()
        .recursive_folds
        .iter()
        .filter(|fold| fold.params.setup_prefix().is_some())
        .count();
    let selection = resolved.selection();

    let t0 = Instant::now();
    let mut prover_transcript = AkitaTranscript::<Cfg::Field>::new(TRANSCRIPT_DOMAIN);
    let proof = AkitaCommitmentScheme::<Cfg>::batched_prove::<_, _, _>(
        &setup,
        prover_claims::<Cfg, _>(
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
    let mut verifier_transcript = AkitaTranscript::<Cfg::Field>::new(TRANSCRIPT_DOMAIN);
    AkitaCommitmentScheme::<Cfg>::batched_verify(
        &proof,
        &verifier_setup,
        &mut verifier_transcript,
        verifier_claims::<Cfg>(
            selection,
            &point,
            &[opening],
            &commit_output.committed_group,
        )?,
        BasisMode::Lagrange,
    )
    .map_err(|error| error.to_string())?;
    let verify_ns = elapsed_ns(t0);

    if negative_check_enabled() {
        let altered_opening = opening + Cfg::ExtField::one();
        let mut negative_transcript = AkitaTranscript::<Cfg::Field>::new(TRANSCRIPT_DOMAIN);
        if AkitaCommitmentScheme::<Cfg>::batched_verify(
            &proof,
            &verifier_setup,
            &mut negative_transcript,
            verifier_claims::<Cfg>(
                selection,
                &point,
                &[altered_opening],
                &commit_output.committed_group,
            )?,
            BasisMode::Lagrange,
        )
        .is_ok()
        {
            return Err("Akita verifier accepted an altered opening claim".into());
        }
    }

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    // The group profile is verifier context selected by the shared catalog.
    // Charge only the cryptographic commitment payload, not the self-describing
    // `CommittedGroup` envelope used to reconstruct that context.
    let commitment_bytes = commit_output
        .committed_group
        .commitment()
        .serialized_size(Compress::No) as u64;
    if commitment_bytes != 128 {
        return Err(format!(
            "unexpected Akita commitment payload size: {commitment_bytes} bytes"
        ));
    }
    let public_context_bytes = (commit_output
        .committed_group
        .serialized_size(Compress::No) as u64)
        .saturating_sub(commitment_bytes);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(format!(
            "catalog={catalog},distribution=full-field-uniform,point=full-extension-uniform,setup_offload_edges={offload_edges}"
        )),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof.serialized_size(Compress::No) as u64),
        commitment_bytes: Some(commitment_bytes),
        evaluation_bytes: Some(opening.serialized_size(Compress::No) as u64),
        public_context_bytes: Some(public_context_bytes),
        state_bytes: Some(setup.expanded.serialized_size(Compress::No) as u64),
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn prover_claims<'a, Cfg, P>(
    point: &'a [Cfg::ExtField],
    polynomials: &'a [&'a P],
    commitment: &'a CommittedGroup<Cfg::Field>,
    hint: AkitaCommitmentHint<Cfg::Field>,
) -> Result<ProverOpeningData<'a, Cfg, P>, String>
where
    Cfg: CommitmentConfig,
    Cfg::ExtField: Ring,
    P: akita_prover::RootPolyMeta<Cfg::Field>,
{
    let group = PolynomialGroupClaims::new(
        point.to_vec(),
        vec![Cfg::ExtField::from_u64(0); polynomials.len()],
        commitment.clone(),
    )
    .map_err(|error| error.to_string())?;
    let claims = OpeningClaims::from_groups(vec![group]).map_err(|error| error.to_string())?;
    SelectedProverOpeningData::from_committed_claims::<Cfg>(claims, vec![hint], vec![polynomials])
        .map_err(|error| error.to_string())
}

fn verifier_claims<'a, Cfg>(
    selection: OpeningScheduleSelection,
    point: &[Cfg::ExtField],
    openings: &[Cfg::ExtField],
    commitment: &'a CommittedGroup<Cfg::Field>,
) -> Result<GroupBatchStatement<'a, Cfg::ExtField, Cfg::Field>, String>
where
    Cfg: CommitmentConfig,
{
    let group = PolynomialGroupClaims::new(point.to_vec(), openings.to_vec(), commitment)
        .map_err(|error| error.to_string())?;
    let claims = OpeningClaims::from_groups(vec![group]).map_err(|error| error.to_string())?;
    GroupBatchStatement::new(selection, claims).map_err(|error| error.to_string())
}

fn dense_evaluations<F: Field>(num_vars: usize) -> Vec<F> {
    let mut rng = StdRng::seed_from_u64(configured_seed(INPUT_SEED));
    (0..(1usize << num_vars))
        .map(|_| F::random(&mut rng))
        .collect()
}

fn opening_point<E: Field>(num_vars: usize) -> Vec<E> {
    let mut rng = StdRng::seed_from_u64(configured_seed(POINT_SEED));
    (0..num_vars).map(|_| E::random(&mut rng)).collect()
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

fn has_flag(name: &str) -> bool {
    std::env::args().skip(1).any(|arg| arg == name)
}

fn parse_u32_flag(name: &str) -> Result<u32, String> {
    parse_string_flag(name)
        .ok_or_else(|| format!("missing {name}"))?
        .parse()
        .map_err(|_| format!("invalid {name}"))
}

fn parse_string_flag(name: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == name {
            return args.next();
        }
        if let Some(value) = arg.strip_prefix(&format!("{name}=")) {
            return Some(value.to_string());
        }
    }
    None
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
