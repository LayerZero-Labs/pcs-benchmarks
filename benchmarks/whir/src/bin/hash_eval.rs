//! Single-shot dense WHIR worker (Plonky3 `p3-whir`).

use p3_challenger::DuplexChallenger;
use p3_commit::MultilinearPcs;
use p3_dft::Radix2DFTSmallBatch;
use p3_field::extension::QuinticTrinomialExtensionField;
use p3_field::Field;
use p3_koala_bear::{KoalaBear, Poseidon2KoalaBear};
use p3_merkle_tree::MerkleTreeMmcs;
use p3_sumcheck::layout::{Layout as _, SuffixProver, Table};
use p3_sumcheck::{OpeningBatch, OpeningProtocol, PointSchedule, TableShape, TableSpec};
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_whir::fiat_shamir::domain_separator::DomainSeparator;
use p3_whir::parameters::{
    FoldingFactor, ProtocolParameters, SecurityAssumption, WhirConfig, WhirConfigError,
};
use p3_whir::pcs::prover::WhirProver;
use pcs_bench_core::{
    whir_first_fold_with_rate, whir_round_log_inv_rates_with_rate, RunStatus, WorkerOutput,
    HASH_SECURITY_BITS, WHIR_FOLDING_FACTOR, WHIR_MAX_POW_BITS, WHIR_POW_BITS,
    WHIR_STARTING_LOG_INV_RATE,
};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

type F = KoalaBear;
type EF = QuinticTrinomialExtensionField<F>;
type Poseidon16 = Poseidon2KoalaBear<16>;
type Poseidon24 = Poseidon2KoalaBear<24>;
type MerkleHash = PaddingFreeSponge<Poseidon24, 24, 16, 8>;
type MerkleCompress = TruncatedPermutation<Poseidon16, 2, 8, 16>;
type MyChallenger = DuplexChallenger<F, Poseidon16, 16, 8>;
type PackedF = <F as Field>::Packing;
type MyMmcs = MerkleTreeMmcs<PackedF, PackedF, MerkleHash, MerkleCompress, 2, 8>;
type MyDft = Radix2DFTSmallBatch<F>;
type Layout = SuffixProver<F, EF>;
type MyPcs = WhirProver<EF, F, MyDft, MyMmcs, MyChallenger, Layout>;

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
                state_bytes: None,
                peak_rss_bytes: peak_rss_bytes(),
            });
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let log2_n = parse_u32_flag("--log2-n")?;
    if has_flag("--config-only") {
        let derived = derive_whir_config(log2_n)?;
        eprintln!(
            "WHIR nv={log2_n} soundness={:?} rate=1/{} pow_bits={} max_pow_bits={} max_fft={}",
            derived.soundness,
            1usize << derived.starting_log_inv_rate,
            derived.pow_bits,
            derived.config.max_pow_bits(),
            derived.config.max_fft_size()
        );
        return emit(&WorkerOutput {
            status: RunStatus::Ok,
            status_detail: Some(format!(
                "soundness={:?},rate=1/{},pow_bits={}",
                derived.soundness,
                1usize << derived.starting_log_inv_rate,
                derived.pow_bits
            )),
            log2_n: Some(log2_n),
            timings_ns: BTreeMap::new(),
            proof_bytes: None,
            commitment_bytes: None,
            state_bytes: Some(0),
            peak_rss_bytes: peak_rss_bytes(),
        });
    }
    let output = timed_whir(log2_n)?;
    emit(&output)
}

#[allow(clippy::too_many_lines)]
fn timed_whir(log2_n: u32) -> Result<WorkerOutput, String> {
    let derived = derive_whir_config(log2_n)?;
    let num_variables = log2_n as usize;
    let folding_factor = whir_folding_factor(log2_n, derived.starting_log_inv_rate);

    let mut rng = StdRng::seed_from_u64(1);
    let poseidon16 = Poseidon16::new_from_rng_128(&mut rng);
    let poseidon24 = Poseidon24::new_from_rng_128(&mut rng);
    let merkle_hash = MerkleHash::new(poseidon24);
    let merkle_compress = MerkleCompress::new(poseidon16.clone());
    let mmcs = MyMmcs::new(merkle_hash, merkle_compress, 0);

    let mut rng = StdRng::seed_from_u64(0);
    let table = Table::rand(&mut rng, 1, num_variables);
    let witness = Layout::new_witness(vec![table], folding_factor.at_round(0));
    let point_schedule: PointSchedule = vec![OpeningBatch::new(vec![0], Vec::new())];
    let protocol = OpeningProtocol::new(vec![TableSpec::new(
        TableShape::new(num_variables, 1),
        point_schedule,
    )])
    .pad_to_min_num_variables(folding_factor.at_round(0));

    eprintln!(
        "WHIR nv={log2_n} soundness={:?} rate=1/{} pow_bits={} max_pow_bits={}",
        derived.soundness,
        1usize << derived.starting_log_inv_rate,
        derived.pow_bits,
        derived.config.max_pow_bits()
    );
    let t0 = Instant::now();
    let config = derived.config;
    let challenger = MyChallenger::new(poseidon16);
    let dft = Radix2DFTSmallBatch::<F>::new(1 << config.max_fft_size());
    let pcs = MyPcs::new(config, dft, mmcs);
    let setup_ns = elapsed_ns(t0);

    let mut prover_challenger = challenger.clone();
    let mut domainsep = DomainSeparator::new(vec![]);
    pcs.add_domain_separator::<8>(&mut domainsep);
    domainsep.observe_domain_separator(&mut prover_challenger);
    let t0 = Instant::now();
    let (commitment, prover_data) =
        <MyPcs as MultilinearPcs<EF, MyChallenger>>::commit(&pcs, witness, &mut prover_challenger);
    let commit_ns = elapsed_ns(t0);

    let t0 = Instant::now();
    let proof = <MyPcs as MultilinearPcs<EF, MyChallenger>>::open(
        &pcs,
        prover_data,
        protocol.clone(),
        &mut prover_challenger,
    );
    let open_ns = elapsed_ns(t0);

    let proof_bytes = postcard::to_allocvec(&proof)
        .map_err(|error| error.to_string())?
        .len() as u64;
    let commitment_bytes = postcard_len(&commitment);

    let mut verifier_challenger = challenger;
    let mut domainsep = DomainSeparator::new(vec![]);
    pcs.add_domain_separator::<8>(&mut domainsep);
    domainsep.observe_domain_separator(&mut verifier_challenger);
    let t0 = Instant::now();
    <MyPcs as MultilinearPcs<EF, MyChallenger>>::verify(
        &pcs,
        &commitment,
        &proof,
        &mut verifier_challenger,
        protocol,
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
        status_detail: Some(format!(
            "soundness={:?},rate=1/{},pow_bits={}",
            derived.soundness,
            1usize << derived.starting_log_inv_rate,
            derived.pow_bits
        )),
        log2_n: Some(log2_n),
        timings_ns,
        proof_bytes: Some(proof_bytes),
        commitment_bytes,
        state_bytes: Some(0),
        peak_rss_bytes: peak_rss_bytes(),
    })
}

fn postcard_len<T: serde::Serialize>(value: &T) -> Option<u64> {
    postcard::to_allocvec(value)
        .ok()
        .map(|bytes| bytes.len() as u64)
}

struct DerivedWhir {
    config: WhirConfig<EF, F, MyChallenger>,
    pow_bits: usize,
    soundness: SecurityAssumption,
    starting_log_inv_rate: usize,
}

fn whir_folding_factor(log2_n: u32, starting_log_inv_rate: usize) -> FoldingFactor {
    let first_fold = whir_first_fold_with_rate(log2_n, starting_log_inv_rate);
    if first_fold == WHIR_FOLDING_FACTOR {
        FoldingFactor::Constant(first_fold)
    } else {
        FoldingFactor::ConstantFromSecondRound(first_fold, WHIR_FOLDING_FACTOR)
    }
}

fn derive_whir_config(log2_n: u32) -> Result<DerivedWhir, String> {
    let num_variables = log2_n as usize;
    let mut last_pow_error = None;
    for starting_log_inv_rate in [WHIR_STARTING_LOG_INV_RATE, 2] {
        let folding_factor = whir_folding_factor(log2_n, starting_log_inv_rate);
        let round_log_inv_rates = whir_round_log_inv_rates_with_rate(log2_n, starting_log_inv_rate);
        for soundness in [
            SecurityAssumption::CapacityBound,
            SecurityAssumption::JohnsonBound,
            SecurityAssumption::UniqueDecoding,
        ] {
            for pow_bits in WHIR_POW_BITS..=WHIR_MAX_POW_BITS {
                let params = ProtocolParameters {
                    security_level: HASH_SECURITY_BITS as usize,
                    pow_bits,
                    folding_factor: folding_factor.clone(),
                    soundness_type: soundness,
                    starting_log_inv_rate,
                    round_log_inv_rates: round_log_inv_rates.clone(),
                };
                match WhirConfig::<EF, F, MyChallenger>::new(num_variables, params) {
                    Ok(config) => {
                        return Ok(DerivedWhir {
                            config,
                            pow_bits,
                            soundness,
                            starting_log_inv_rate,
                        });
                    }
                    Err(WhirConfigError::PowBitsExceedBudget { required, budget }) => {
                        last_pow_error = Some(format!(
                            "{soundness:?} rate=1/{} budget {budget}: derived {required}-bit PoW",
                            1usize << starting_log_inv_rate
                        ));
                    }
                    Err(error) => return Err(error.to_string()),
                }
            }
        }
    }
    Err(last_pow_error.unwrap_or_else(|| {
        "WHIR cannot reach 128 bits within the 30-bit KoalaBear grinding limit".into()
    }))
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

fn has_flag(name: &str) -> bool {
    std::env::args().skip(1).any(|arg| arg == name)
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
