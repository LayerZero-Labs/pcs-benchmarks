//! Shared Plonky3 univariate FRI/STIR worker.

use p3_challenger::{CanObserve, DuplexChallenger, FieldChallenger};
use p3_commit::{ExtensionMmcs, Pcs};
use p3_dft::Radix2DFTSmallBatch;
use p3_field::coset::TwoAdicMultiplicativeCoset;
use p3_field::extension::QuinticTrinomialExtensionField;
use p3_field::{Field, PrimeCharacteristicRing};
use p3_fri::{FriParameters, TwoAdicFriPcs};
use p3_koala_bear::{KoalaBear, Poseidon2KoalaBear};
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_stir::{StirConfig, StirParameters, TwoAdicStirPcs};
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_whir::parameters::SecurityAssumption;
use pcs_bench_core::{
    plonky3_log_height, plonky3_log_width, RunStatus, WorkerOutput, HASH_SECURITY_BITS_100,
    PLONKY3_FRI_POW_BITS, PLONKY3_FRI_QUERIES, PLONKY3_UNI_LOG_BLOWUP,
};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type F = KoalaBear;
type EF = QuinticTrinomialExtensionField<F>;
type Dft = Radix2DFTSmallBatch<F>;
type Poseidon16 = Poseidon2KoalaBear<16>;
type Poseidon24 = Poseidon2KoalaBear<24>;
type MerkleHash = PaddingFreeSponge<Poseidon24, 24, 16, 8>;
type MerkleCompress = TruncatedPermutation<Poseidon16, 2, 8, 16>;
type PackedF = <F as Field>::Packing;
type ValMmcs = MerkleTreeMmcs<PackedF, PackedF, MerkleHash, MerkleCompress, 2, 8>;
type ChallengeMmcs = ExtensionMmcs<F, EF, ValMmcs>;
type Challenger = DuplexChallenger<F, Poseidon16, 16, 8>;
type FriPcsTy = TwoAdicFriPcs<F, Dft, ValMmcs, ChallengeMmcs>;
type StirPcsTy = TwoAdicStirPcs<F, Dft, ValMmcs, ChallengeMmcs, EF, Challenger>;

/// Which univariate protocol to run.
#[derive(Clone, Copy)]
pub enum UniKind {
    /// Plonky3 `TwoAdicFriPcs`.
    Fri,
    /// Plonky3 `TwoAdicStirPcs`.
    Stir,
}

/// Entry point for both bins.
pub fn main_for(kind: UniKind) -> ExitCode {
    let threads = parse_u32_flag("--threads").unwrap_or(1).max(1);
    init_thread_pool(threads);
    match run(kind) {
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

fn run(kind: UniKind) -> Result<(), String> {
    let log2_n = parse_u32_flag("--log2-n")?;
    emit(&timed_univariate(kind, log2_n)?)
}

fn timed_univariate(kind: UniKind, log2_n: u32) -> Result<WorkerOutput, String> {
    let log_height = plonky3_log_height(log2_n) as usize;
    let log_width = plonky3_log_width(log2_n) as usize;
    let width = 1usize << log_width;
    let log_blowup = PLONKY3_UNI_LOG_BLOWUP as usize;
    let packed = log_width > 0;

    let setup_start = Instant::now();
    let mut perm_rng = SmallRng::seed_from_u64(1);
    let poseidon16 = Poseidon16::new_from_rng_128(&mut perm_rng);
    let poseidon24 = Poseidon24::new_from_rng_128(&mut perm_rng);
    let val_mmcs = ValMmcs::new(
        MerkleHash::new(poseidon24),
        MerkleCompress::new(poseidon16.clone()),
        0,
    );
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let base_challenger = Challenger::new(poseidon16);

    match kind {
        UniKind::Fri => {
            let num_queries = PLONKY3_FRI_QUERIES;
            let fri_params = FriParameters {
                log_blowup,
                log_final_poly_len: 0,
                max_log_arity: 1,
                num_queries,
                commit_proof_of_work_bits: 0,
                query_proof_of_work_bits: PLONKY3_FRI_POW_BITS,
                mmcs: challenge_mmcs,
            };
            let dft = Dft::new(1 << (log_height + log_blowup));
            let pcs = FriPcsTy::new(dft, val_mmcs, fri_params);
            let setup_ns = elapsed_ns(setup_start);
            let domain = Pcs::<EF, Challenger>::natural_domain_for_degree(&pcs, 1 << log_height);
            let mut rng = SmallRng::seed_from_u64(configured_seed(0xF12));
            let message = RowMajorMatrix::<F>::rand(&mut rng, 1 << log_height, width);
            timed_pcs(
                "fri",
                &pcs,
                domain,
                message,
                &base_challenger,
                setup_ns,
                log2_n,
                packed,
                |ch, commit| ch.observe(commit.clone()),
            )
        }
        UniKind::Stir => {
            let stir_params = StirParameters {
                log_blowup,
                log_folding_factor: 4,
                log_starting_folding_factor: 2,
                soundness_type: SecurityAssumption::CapacityBound,
                security_level: HASH_SECURITY_BITS_100 as usize,
                max_pow_bits: PLONKY3_FRI_POW_BITS,
                mmcs: challenge_mmcs,
            };
            StirConfig::<F, EF, ChallengeMmcs, Challenger>::try_new(
                log_height,
                stir_params.clone(),
            )
            .map_err(|error| error.to_string())?;
            let dft = Dft::new(1 << (log_height + log_blowup));
            let pcs = StirPcsTy::new(dft, val_mmcs, stir_params);
            let setup_ns = elapsed_ns(setup_start);
            let domain = Pcs::<EF, Challenger>::natural_domain_for_degree(&pcs, 1 << log_height);
            let mut rng = SmallRng::seed_from_u64(configured_seed(0x57113));
            let message = RowMajorMatrix::<F>::rand(&mut rng, 1 << log_height, width);
            timed_pcs(
                "stir",
                &pcs,
                domain,
                message,
                &base_challenger,
                setup_ns,
                log2_n,
                packed,
                |ch, commit| commit.iter().for_each(|root| ch.observe(root.clone())),
            )
        }
    }
}

fn timed_pcs<P>(
    label: &str,
    pcs: &P,
    domain: TwoAdicMultiplicativeCoset<F>,
    message: RowMajorMatrix<F>,
    base_challenger: &Challenger,
    setup_ns: u64,
    log2_n: u32,
    packed: bool,
    observe: impl Fn(&mut Challenger, &P::Commitment),
) -> Result<WorkerOutput, String>
where
    P: Pcs<EF, Challenger, Domain = TwoAdicMultiplicativeCoset<F>>,
    P::Commitment: Clone,
    P::Proof: serde::Serialize,
{
    let mut prover_challenger = base_challenger.clone();
    let t0 = Instant::now();
    let (commit, prover_data) = pcs.commit(vec![(domain, message)]);
    let commit_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    observe(&mut prover_challenger, &commit);
    let zeta: EF = FieldChallenger::<F>::sample_algebra_element(&mut prover_challenger);

    let opening_points = vec![vec![zeta]];
    let (openings, proof) = pcs.open(vec![(&prover_data, opening_points)], &mut prover_challenger);
    let open_ns = elapsed_ns(t0);
    let values = openings[0][0][0].clone();

    let t0 = Instant::now();
    let mut verifier_challenger = base_challenger.clone();
    observe(&mut verifier_challenger, &commit);
    let derived: EF = FieldChallenger::<F>::sample_algebra_element(&mut verifier_challenger);
    if derived != zeta {
        return Err(format!("{label} verifier challenger drifted from prover"));
    }

    pcs.verify(
        vec![(
            commit.clone(),
            vec![(domain.clone(), vec![(zeta, values.clone())])],
        )],
        &proof,
        &mut verifier_challenger,
    )
    .map_err(|error| format!("{label} verify failed: {error:?}"))?;
    let verify_ns = elapsed_ns(t0);
    let evaluation_bytes = postcard::to_allocvec(&values)
        .map_err(|error| error.to_string())?
        .len() as u64;

    if negative_check_enabled() {
        let mut negative_challenger = base_challenger.clone();
        observe(&mut negative_challenger, &commit);
        let altered_zeta: EF =
            FieldChallenger::<F>::sample_algebra_element(&mut negative_challenger);
        let mut altered_values = values;
        let first = altered_values
            .first_mut()
            .ok_or_else(|| format!("{label} returned an empty opening"))?;
        *first += EF::ONE;
        if pcs
            .verify(
                vec![(
                    commit.clone(),
                    vec![(domain, vec![(altered_zeta, altered_values)])],
                )],
                &proof,
                &mut negative_challenger,
            )
            .is_ok()
        {
            return Err(format!(
                "{label} verifier accepted an altered opening claim"
            ));
        }
    }

    let proof_bytes = postcard::to_allocvec(&proof)
        .map_err(|error| error.to_string())?
        .len() as u64;
    let commitment_bytes = postcard::to_allocvec(&commit)
        .ok()
        .map(|bytes| bytes.len() as u64);

    let mut timings_ns = BTreeMap::new();
    timings_ns.insert("setup".into(), setup_ns);
    timings_ns.insert("commit".into(), commit_ns);
    timings_ns.insert("open".into(), open_ns);
    timings_ns.insert("verify".into(), verify_ns);

    Ok(WorkerOutput {
        status: RunStatus::Ok,
        status_detail: Some(if packed {
            format!(
                "statement=univariate-batch,distribution=full-field-uniform,point=transcript-extension,height={},width={}",
                plonky3_log_height(log2_n),
                1u32 << plonky3_log_width(log2_n)
            )
        } else {
            "statement=univariate,distribution=full-field-uniform,point=transcript-extension".into()
        }),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes,
        evaluation_bytes: Some(evaluation_bytes),
        public_context_bytes: Some(0),
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

fn configured_seed(domain: u64) -> u64 {
    std::env::var("PCS_BENCH_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(domain, |seed| seed ^ domain)
}

fn negative_check_enabled() -> bool {
    std::env::var("PCS_BENCH_NEGATIVE_CHECK").as_deref() == Ok("1")
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
