//! Hash-based PCS comparison matching the dense-payload evaluation table.

use crate::lattice::{
    log2_n_for_32bit_payload, FieldSpec, AKITA_FP128, AKITA_FP32, AKITA_FP64, PAYLOAD_LOG2,
};
use serde::{Deserialize, Serialize};

/// Thread counts in the hash timing table.
pub const HASH_THREADS: [u32; 2] = [1, 8];

/// Schemes in roster order (Akita fp32/fp64/fp128 through BaseFold SP1).
pub const HASH_SCHEME_COUNT: usize = 11;

/// Headline hash matrix size: 5 payloads × 11 schemes × 2 thread counts.
pub const HASH_CELL_COUNT: usize = 110;

/// KoalaBear prime `2^31 - 2^24 + 1`.
pub const KOALA_BEAR: FieldSpec = FieldSpec {
    name: "2^{31}-2^{24}+1",
    modulus: 2_130_706_433,
    log2_bits: 31,
};

/// Goldilocks prime `2^64 - 2^32 + 1`.
pub const GOLDILOCKS: FieldSpec = FieldSpec {
    name: "2^{64}-2^{32}+1",
    modulus: 0xFFFF_FFFF_0000_0001,
    log2_bits: 64,
};

/// Binary extension \(\mathbb F_{2^{128}}\) used by Binius64 BaseFold.
pub const BINARY_128: FieldSpec = FieldSpec {
    name: "F_{2^{128}}",
    modulus: 0,
    log2_bits: 128,
};

/// Bit-valued multilinear packed into \(\mathbb F_{2^{128}}\) (Flock Ligerito).
pub const FLOCK_BITS: FieldSpec = FieldSpec {
    name: "F_2",
    modulus: 0,
    log2_bits: 1,
};

/// KoalaBear two-adicity; univariate FRI/STIR packing and WHIR first-round fold.
pub const KOALA_BEAR_TWO_ADICITY: u32 = 24;

/// Pinned [Plonky3](https://github.com/Plonky3/Plonky3) revision (`p3-whir`).
pub const PLONKY3_REVISION: &str = "9d496524560f3c699473906c6f50fca7cf343730";

/// Pinned Plonky3 revision for univariate FRI and STIR.
pub const PLONKY3_FRI_STIR_REVISION: &str = "3da160d09d1c6a878adaa5b339939fcdccda5d36";

/// Pinned [SP1 / SLOP](https://github.com/succinctlabs/sp1) revision (`slop-basefold`).
pub const SP1_REVISION: &str = "0f2a1e1389747ac0dbee1c4d40243eed20baba86";

/// Pinned [elliottech/plonky2](https://github.com/elliottech/plonky2) revision.
pub const PLONKY2_REVISION: &str = "e1c2d35450948b88fca6a7e69e2643c3ecad3caa";

/// Pinned [Binius64](https://github.com/binius-zk/binius64) revision.
pub const BINIUS64_REVISION: &str = "6e75a2d1d2e716578ae3ccb62806413fb1615176";

/// Pinned [Flock](https://github.com/succinctlabs/flock) revision.
pub const FLOCK_REVISION: &str = "43f0eee06d887d87ad25d72614cbc2b17fe91430";

/// Pinned [worldfnd/whir](https://github.com/worldfnd/whir) revision used by ProveKit WHIR.
pub const WHIR_PROVEKIT_WHIR_REVISION: &str = "8804e80e8e890d01bb585f2bd5e5b564ac0fd80d";

/// Common transcript-error target for Akita, Plonky3 WHIR, and SP1 BaseFold, in bits.
pub const HASH_SECURITY_BITS: u32 = 128;

/// Native 100-bit target used by Plonky2 FRI, Plonky3 FRI/STIR, and Binius64.
pub const HASH_SECURITY_BITS_100: u32 = 100;

/// ProveKit WHIR internal target (Johnson bound).
pub const PROVEKIT_SECURITY_BITS: u32 = 133;

/// BaseFold interleaved height. Domain `2^{height+1}` fits KoalaBear two-adicity 24.
pub const BASEFOLD_LOG_STACKING_HEIGHT: u32 = 20;

/// FRI log-inverse rate for BaseFold (`rho = 1/2`).
pub const BASEFOLD_FRI_LOG_BLOWUP: usize = 1;

/// FRI queries for 128-bit conjectured soundness: `log_blowup * queries + pow = 128`.
pub const BASEFOLD_FRI_QUERIES: usize = 112;

/// FRI query proof-of-work bits for BaseFold.
pub const BASEFOLD_FRI_POW_BITS: usize = 16;

/// Plonky3 FRI/STIR log-inverse rate (`rho = 1/2`).
pub const PLONKY3_UNI_LOG_BLOWUP: u32 = 1;

/// Plonky3 FRI grinding budget.
pub const PLONKY3_FRI_POW_BITS: usize = 20;

/// Plonky3 FRI query count: `(100 - 20) / 1 = 80`.
pub const PLONKY3_FRI_QUERIES: usize = 80;

/// Plonky2 FRI log-inverse rate (`rho = 1/8`).
pub const PLONKY2_FRI_RATE_BITS: usize = 3;

/// Plonky2 FRI queries at the native 100-bit conjectural target.
pub const PLONKY2_FRI_QUERIES: usize = 28;

/// Plonky2 FRI grinding bits.
pub const PLONKY2_FRI_POW_BITS: usize = 16;

/// Plonky2 Merkle cap height (standard recursion config).
pub const PLONKY2_CAP_HEIGHT: usize = 4;

/// WHIR starting log-inverse rate (`rho = 1/2`), matching `p3-whir` benches.
pub const WHIR_STARTING_LOG_INV_RATE: usize = 1;

/// WHIR folding factor after the first round.
pub const WHIR_FOLDING_FACTOR: usize = 4;

/// Starting WHIR grinding budget. The worker searches `[WHIR_POW_BITS, WHIR_MAX_POW_BITS]`
/// independently (raising the budget also lowers the algebraic query target).
pub const WHIR_POW_BITS: usize = 20;

/// KoalaBear grinding limit: Fiat-Shamir grind requires `2^bits < q`.
pub const WHIR_MAX_POW_BITS: usize = 30;

/// WHIR direct-send threshold (matches `p3-whir` `MAX_NUM_VARIABLES_TO_SEND_COEFFS`).
pub const WHIR_DIRECT_SEND_VARS: usize = 6;

/// ProveKit WHIR starting log-inverse rate (`rho = 1/4`).
pub const PROVEKIT_WHIR_LOG_INV_RATE: usize = 2;

/// ProveKit WHIR folding factor.
pub const PROVEKIT_WHIR_FOLD: usize = 8;

/// Flock packing: `m` bit-variables become `m - 7` packed \(\mathbb F_{2^{128}}\) variables.
pub const FLOCK_LOG_PACKING: u32 = 7;

/// Identifies a hash-eval implementation, in roster order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashSchemeId {
    /// Akita at the pinned `main` commit, same direct fp32-dense catalog as lattice-eval.
    Akita,
    /// Same pin as [`Self::Akita`], fp64-dense catalog (`q = 2^{64}-59`).
    AkitaFp64,
    /// Same pin as [`Self::Akita`], fp128-dense catalog (`q = 2^{128}-2^{32}+22537`).
    AkitaFp128,
    /// Plonky2 univariate FRI over Goldilocks (`elliottech/plonky2`).
    Plonky2Fri,
    /// Plonky3 univariate FRI over KoalaBear.
    Plonky3Fri,
    /// Plonky3 univariate STIR over KoalaBear.
    Plonky3Stir,
    /// Plonky3 `p3-whir` multilinear PCS.
    Whir,
    /// Binius64 BaseFold over \(\mathbb F_{2^{128}}\).
    Binius64,
    /// Flock Ligerito bit-multilinear PCS (Fast profile).
    FlockLigerito,
    /// worldfnd/whir via ProveKit, Goldilocks degree-3 challenges, base-field coeffs.
    WhirProvekit,
    /// SP1 SLOP stacked BaseFold (`slop-basefold`).
    Basefold,
}

impl HashSchemeId {
    /// Stable table label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Akita | Self::AkitaFp64 | Self::AkitaFp128 => "Akita",
            Self::Plonky2Fri => "Plonky2 FRI",
            Self::Plonky3Fri => "Plonky3 FRI",
            Self::Plonky3Stir => "Plonky3 STIR",
            Self::Whir => "WHIR (Plonky3)",
            Self::Binius64 => "Binius64 BaseFold",
            Self::FlockLigerito => "Flock Ligerito",
            Self::WhirProvekit => "WHIR (ProveKit)",
            Self::Basefold => "BaseFold (SP1)",
        }
    }

    /// Table label with LaTeX escaping.
    #[must_use]
    pub const fn latex_name(self) -> &'static str {
        self.display_name()
    }

    /// Parse a CLI scheme token.
    #[must_use]
    pub fn parse_token(token: &str) -> Option<Self> {
        match token {
            "akita" => Some(Self::Akita),
            "akita-fp64" | "akita_fp64" => Some(Self::AkitaFp64),
            "akita-fp128" | "akita_fp128" => Some(Self::AkitaFp128),
            "plonky2" | "plonky2-fri" => Some(Self::Plonky2Fri),
            "plonky3-fri" | "p3-fri" => Some(Self::Plonky3Fri),
            "plonky3-stir" | "p3-stir" | "stir" => Some(Self::Plonky3Stir),
            "whir" => Some(Self::Whir),
            "binius64" | "binius" => Some(Self::Binius64),
            "flock" | "ligerito" | "flock-ligerito" => Some(Self::FlockLigerito),
            "whir-provekit" | "provekit" | "whir-goldilocks" => Some(Self::WhirProvekit),
            "basefold" | "base-fold" => Some(Self::Basefold),
            _ => None,
        }
    }

    /// CLI token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Akita => "akita",
            Self::AkitaFp64 => "akita-fp64",
            Self::AkitaFp128 => "akita-fp128",
            Self::Plonky2Fri => "plonky2-fri",
            Self::Plonky3Fri => "plonky3-fri",
            Self::Plonky3Stir => "plonky3-stir",
            Self::Whir => "whir",
            Self::Binius64 => "binius64",
            Self::FlockLigerito => "flock",
            Self::WhirProvekit => "whir-provekit",
            Self::Basefold => "basefold",
        }
    }

    /// Every scheme in table order.
    #[must_use]
    pub const fn all() -> [Self; HASH_SCHEME_COUNT] {
        [
            Self::Akita,
            Self::AkitaFp64,
            Self::AkitaFp128,
            Self::Plonky2Fri,
            Self::Plonky3Fri,
            Self::Plonky3Stir,
            Self::Whir,
            Self::Binius64,
            Self::FlockLigerito,
            Self::WhirProvekit,
            Self::Basefold,
        ]
    }

    /// GitHub repository URL without a trailing slash.
    #[must_use]
    pub const fn source_repo(self) -> &'static str {
        match self {
            Self::Akita | Self::AkitaFp64 | Self::AkitaFp128 => {
                "https://github.com/LayerZero-Labs/akita"
            }
            Self::Plonky2Fri => "https://github.com/elliottech/plonky2",
            Self::Plonky3Fri | Self::Plonky3Stir | Self::Whir => {
                "https://github.com/Plonky3/Plonky3"
            }
            Self::Binius64 => "https://github.com/binius-zk/binius64",
            Self::FlockLigerito => "https://github.com/succinctlabs/flock",
            Self::WhirProvekit => "https://github.com/worldfnd/whir",
            Self::Basefold => "https://github.com/succinctlabs/sp1",
        }
    }

    /// Pinned git SHA measured for this scheme.
    #[must_use]
    pub const fn revision(self) -> &'static str {
        match self {
            Self::Akita | Self::AkitaFp64 | Self::AkitaFp128 => crate::lattice::AKITA_REVISION,
            Self::Plonky2Fri => PLONKY2_REVISION,
            Self::Plonky3Fri | Self::Plonky3Stir => PLONKY3_FRI_STIR_REVISION,
            Self::Whir => PLONKY3_REVISION,
            Self::Binius64 => BINIUS64_REVISION,
            Self::FlockLigerito => FLOCK_REVISION,
            Self::WhirProvekit => WHIR_PROVEKIT_WHIR_REVISION,
            Self::Basefold => SP1_REVISION,
        }
    }

    /// Canonical GitHub commit URL for the pinned revision.
    #[must_use]
    pub fn commit_url(self) -> String {
        format!("{}/commit/{}", self.source_repo(), self.revision())
    }

    /// `--field` value for the shared Akita worker, when this scheme is Akita.
    #[must_use]
    pub const fn akita_field_arg(self) -> Option<&'static str> {
        match self {
            Self::Akita => Some("fp32"),
            Self::AkitaFp64 => Some("fp64"),
            Self::AkitaFp128 => Some("fp128"),
            _ => None,
        }
    }

    /// Abbreviated SHA used in tables.
    #[must_use]
    pub fn short_sha(self) -> &'static str {
        let sha = self.revision();
        sha.get(..8).unwrap_or(sha)
    }

    /// Native transcript-error target used by the benchmark configuration.
    #[must_use]
    pub const fn security_bits(self) -> u32 {
        match self {
            Self::Plonky2Fri
            | Self::Plonky3Fri
            | Self::Plonky3Stir
            | Self::Binius64
            | Self::FlockLigerito => HASH_SECURITY_BITS_100,
            Self::WhirProvekit => PROVEKIT_SECURITY_BITS,
            Self::Akita | Self::AkitaFp64 | Self::AkitaFp128 | Self::Whir | Self::Basefold => {
                HASH_SECURITY_BITS
            }
        }
    }

    /// Polynomial statement measured by this adapter.
    #[must_use]
    pub const fn statement(self) -> &'static str {
        match self {
            Self::Plonky2Fri | Self::Plonky3Fri | Self::Plonky3Stir => "univariate",
            Self::FlockLigerito => "packed F128 MLE",
            Self::Akita
            | Self::AkitaFp64
            | Self::AkitaFp128
            | Self::Whir
            | Self::Binius64
            | Self::WhirProvekit
            | Self::Basefold => "multilinear",
        }
    }
}

/// One cell in the dense hash comparison (payload × scheme × threads).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashCase {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: HashSchemeId,
    /// Implementation field.
    pub field: FieldSpec,
    /// Native `log2 N` (coefficient or bit count; see scheme).
    pub log2_n: u32,
    /// Worker thread count.
    pub threads: u32,
    /// Compile-time / protocol parameter name.
    pub native_param: &'static str,
}

/// `log2 N` so that `N * coeff_bits` matches the payload bit volume.
///
/// `coeff_bits` must be a power of two (1, 32, 64, 128, 256).
#[must_use]
pub const fn log2_n_for_payload_bits(payload_log2: u32, coeff_bits: u32) -> u32 {
    payload_log2 - coeff_bits.trailing_zeros()
}

/// Plonky3 univariate FRI/STIR log-height after packing into KoalaBear two-adicity.
#[must_use]
pub const fn plonky3_log_height(log2_n: u32) -> u32 {
    let max_h = KOALA_BEAR_TWO_ADICITY.saturating_sub(PLONKY3_UNI_LOG_BLOWUP);
    if log2_n <= max_h {
        log2_n
    } else {
        max_h
    }
}

/// Log-width (number of columns) for packed Plonky3 FRI/STIR.
#[must_use]
pub const fn plonky3_log_width(log2_n: u32) -> u32 {
    log2_n.saturating_sub(plonky3_log_height(log2_n))
}

/// True when the univariate must be packed into more than one column.
#[must_use]
pub const fn plonky3_is_packed(log2_n: u32) -> bool {
    plonky3_log_width(log2_n) > 0
}

/// First-round WHIR fold so `log2_n + rate - fold <=` KoalaBear two-adicity.
#[must_use]
pub const fn whir_first_fold(log2_n: u32) -> usize {
    whir_first_fold_with_rate(log2_n, WHIR_STARTING_LOG_INV_RATE)
}

/// First-round fold for an explicit starting log-inverse rate.
#[must_use]
pub const fn whir_first_fold_with_rate(log2_n: u32, starting_log_inv_rate: usize) -> usize {
    let min_fold = log2_n
        .saturating_add(starting_log_inv_rate as u32)
        .saturating_sub(KOALA_BEAR_TWO_ADICITY);
    if min_fold > WHIR_FOLDING_FACTOR as u32 {
        min_fold as usize
    } else {
        WHIR_FOLDING_FACTOR
    }
}

/// Concrete WHIR folding schedule, including the initial fold, matching `p3-whir`.
#[must_use]
pub fn whir_folding_schedule(log2_n: u32) -> Vec<usize> {
    whir_folding_schedule_with_rate(log2_n, WHIR_STARTING_LOG_INV_RATE)
}

/// Folding schedule for an explicit starting log-inverse rate.
#[must_use]
pub fn whir_folding_schedule_with_rate(log2_n: u32, starting_log_inv_rate: usize) -> Vec<usize> {
    let first = whir_first_fold_with_rate(log2_n, starting_log_inv_rate);
    let mut remaining = log2_n as usize;
    let mut schedule = vec![first];
    remaining -= first;
    while remaining > WHIR_DIRECT_SEND_VARS {
        let round_factor = WHIR_FOLDING_FACTOR.min(remaining);
        schedule.push(round_factor);
        remaining -= round_factor;
    }
    schedule
}

/// Per-round log-inverse rates at the default starting rate.
#[must_use]
pub fn whir_round_log_inv_rates(log2_n: u32) -> Vec<usize> {
    whir_round_log_inv_rates_with_rate(log2_n, WHIR_STARTING_LOG_INV_RATE)
}

/// Per-round log-inverse rates. Defaults to WHIR's `rate += fold - 1` schedule,
/// then lowers a round's rate just enough that the next `two_adic_generator`
/// stays inside KoalaBear two-adicity 24.
#[must_use]
pub fn whir_round_log_inv_rates_with_rate(log2_n: u32, starting_log_inv_rate: usize) -> Vec<usize> {
    let schedule = whir_folding_schedule_with_rate(log2_n, starting_log_inv_rate);
    if schedule.len() <= 1 {
        return Vec::new();
    }
    let num_rounds = schedule.len() - 1;
    let mut rates = Vec::with_capacity(num_rounds);
    let mut log_inv_rate = starting_log_inv_rate;
    let mut domain_log = log2_n as usize + starting_log_inv_rate;
    for round in 0..num_rounds {
        let fold = schedule[round];
        let default_next = log_inv_rate + fold - 1;
        let min_rs = if round + 1 < num_rounds {
            domain_log
                .saturating_sub(KOALA_BEAR_TWO_ADICITY as usize)
                .saturating_sub(schedule[round + 1])
                .max(1)
        } else {
            1
        };
        let max_next = log_inv_rate + fold - min_rs;
        let next_rate = default_next.min(max_next).max(1);
        rates.push(next_rate);
        let rs_reduction = log_inv_rate + fold - next_rate;
        domain_log -= rs_reduction;
        log_inv_rate = next_rate;
    }
    rates
}

/// The headline hash matrix (5 payloads × 11 schemes × 2 thread counts).
#[must_use]
pub fn hash_matrix() -> Vec<HashCase> {
    let mut cases = Vec::with_capacity(HASH_CELL_COUNT);
    for payload in PAYLOAD_LOG2 {
        for scheme in HashSchemeId::all() {
            for threads in HASH_THREADS {
                cases.push(hash_case_inner(payload, scheme, threads));
            }
        }
    }
    cases
}

/// Look up one cell.
#[must_use]
pub fn hash_case(payload_log2: u32, scheme: HashSchemeId, threads: u32) -> Option<HashCase> {
    hash_matrix().into_iter().find(|case| {
        case.payload_log2 == payload_log2 && case.scheme == scheme && case.threads == threads
    })
}

fn hash_case_inner(payload_log2: u32, scheme: HashSchemeId, threads: u32) -> HashCase {
    let log2_n_32 = log2_n_for_32bit_payload(payload_log2).unwrap_or(0);
    match scheme {
        HashSchemeId::Akita => HashCase {
            payload_log2,
            scheme,
            field: AKITA_FP32,
            log2_n: log2_n_32,
            threads,
            native_param: "fp32-dense",
        },
        HashSchemeId::AkitaFp64 => HashCase {
            payload_log2,
            scheme,
            field: AKITA_FP64,
            log2_n: log2_n_for_payload_bits(payload_log2, 64),
            threads,
            native_param: "fp64-dense",
        },
        HashSchemeId::AkitaFp128 => HashCase {
            payload_log2,
            scheme,
            field: AKITA_FP128,
            log2_n: log2_n_for_payload_bits(payload_log2, 128),
            threads,
            native_param: "fp128-dense",
        },
        HashSchemeId::Plonky2Fri => HashCase {
            payload_log2,
            scheme,
            field: GOLDILOCKS,
            log2_n: log2_n_for_payload_bits(payload_log2, 64),
            threads,
            native_param: "plonky2-fri-100",
        },
        HashSchemeId::Plonky3Fri => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n: log2_n_32,
            threads,
            native_param: "plonky3-fri-100",
        },
        HashSchemeId::Plonky3Stir => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n: log2_n_32,
            threads,
            native_param: "plonky3-stir-100",
        },
        HashSchemeId::Whir => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n: log2_n_32,
            threads,
            native_param: "whir-128",
        },
        HashSchemeId::Binius64 => HashCase {
            payload_log2,
            scheme,
            field: BINARY_128,
            log2_n: log2_n_for_payload_bits(payload_log2, 128),
            threads,
            native_param: "binius64-basefold-100",
        },
        HashSchemeId::FlockLigerito => HashCase {
            payload_log2,
            scheme,
            field: FLOCK_BITS,
            log2_n: payload_log2,
            threads,
            native_param: "flock-ligerito-fast",
        },
        HashSchemeId::WhirProvekit => HashCase {
            payload_log2,
            scheme,
            field: GOLDILOCKS,
            log2_n: log2_n_for_payload_bits(payload_log2, 64),
            threads,
            native_param: "whir-provekit-goldilocks3-133",
        },
        HashSchemeId::Basefold => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n: log2_n_32,
            threads,
            native_param: "basefold-fri-128",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        hash_matrix, log2_n_for_payload_bits, plonky3_is_packed, plonky3_log_height,
        plonky3_log_width, whir_first_fold, HashSchemeId, BASEFOLD_FRI_LOG_BLOWUP,
        BASEFOLD_FRI_POW_BITS, BASEFOLD_FRI_QUERIES, HASH_CELL_COUNT, HASH_SCHEME_COUNT,
        HASH_SECURITY_BITS, HASH_THREADS, KOALA_BEAR, KOALA_BEAR_TWO_ADICITY, PLONKY3_FRI_QUERIES,
        PLONKY3_UNI_LOG_BLOWUP,
    };
    use crate::lattice::{log2_n_for_32bit_payload, PAYLOAD_LOG2};

    #[test]
    fn security_targets_match_the_roster() {
        assert_eq!(HASH_SECURITY_BITS, 128);
        assert_eq!(
            BASEFOLD_FRI_LOG_BLOWUP * BASEFOLD_FRI_QUERIES + BASEFOLD_FRI_POW_BITS,
            HASH_SECURITY_BITS as usize
        );
        assert_eq!(PLONKY3_FRI_QUERIES, 80);
    }

    #[test]
    fn koala_bear_modulus_matches_the_prime() {
        assert_eq!(KOALA_BEAR.modulus, (1u64 << 31) - (1u64 << 24) + 1);
        assert_eq!(KOALA_BEAR_TWO_ADICITY, 24);
    }

    #[test]
    fn payload_ladders_match_the_roster() {
        assert_eq!(log2_n_for_payload_bits(27, 32), 22);
        assert_eq!(log2_n_for_payload_bits(27, 64), 21);
        assert_eq!(log2_n_for_payload_bits(27, 128), 20);
        assert_eq!(log2_n_for_payload_bits(27, 1), 27);
        for payload in PAYLOAD_LOG2 {
            let matrix = hash_matrix();
            let akita = matrix
                .iter()
                .find(|case| case.payload_log2 == payload && case.scheme == HashSchemeId::Akita)
                .expect("akita");
            assert_eq!(Some(akita.log2_n), log2_n_for_32bit_payload(payload));
            let fp64 = matrix
                .iter()
                .find(|case| case.payload_log2 == payload && case.scheme == HashSchemeId::AkitaFp64)
                .expect("akita fp64");
            assert_eq!(fp64.log2_n, log2_n_for_payload_bits(payload, 64));
            assert_eq!(fp64.field.name, "2^{64}-59");
            let fp128 = matrix
                .iter()
                .find(|case| {
                    case.payload_log2 == payload && case.scheme == HashSchemeId::AkitaFp128
                })
                .expect("akita fp128");
            assert_eq!(fp128.log2_n, log2_n_for_payload_bits(payload, 128));
            assert_eq!(fp128.field.name, "2^{128}-2^{32}+22537");
            let p2 = matrix
                .iter()
                .find(|case| {
                    case.payload_log2 == payload && case.scheme == HashSchemeId::Plonky2Fri
                })
                .expect("plonky2");
            assert_eq!(p2.log2_n, log2_n_for_payload_bits(payload, 64));
            let flock = matrix
                .iter()
                .find(|case| {
                    case.payload_log2 == payload && case.scheme == HashSchemeId::FlockLigerito
                })
                .expect("flock");
            assert_eq!(flock.log2_n, payload);
        }
    }

    #[test]
    fn plonky3_univariate_packs_above_two_adicity() {
        assert!(!plonky3_is_packed(22));
        assert_eq!(plonky3_log_height(22), 22);
        assert_eq!(plonky3_log_width(22), 0);
        assert!(plonky3_is_packed(24));
        assert_eq!(
            plonky3_log_height(24),
            KOALA_BEAR_TWO_ADICITY - PLONKY3_UNI_LOG_BLOWUP
        );
        assert_eq!(plonky3_log_height(30) + plonky3_log_width(30), 30);
        assert!(plonky3_log_height(30) + PLONKY3_UNI_LOG_BLOWUP <= KOALA_BEAR_TWO_ADICITY);
    }

    #[test]
    fn whir_first_fold_keeps_the_fft_inside_koala_bear() {
        assert_eq!(whir_first_fold(22), 4);
        assert_eq!(whir_first_fold(24), 4);
        assert_eq!(whir_first_fold(26), 4);
        assert_eq!(whir_first_fold(28), 5);
        assert_eq!(whir_first_fold(30), 7);
        for log2_n in [22u32, 24, 26, 28, 30] {
            let fft = log2_n + 1 - whir_first_fold(log2_n) as u32;
            assert!(fft <= KOALA_BEAR_TWO_ADICITY);
        }
    }

    #[test]
    fn whir_round_rates_keep_later_generators_inside_koala_bear() {
        use super::{
            whir_first_fold_with_rate, whir_folding_schedule_with_rate, whir_round_log_inv_rates,
            whir_round_log_inv_rates_with_rate,
        };
        for log2_n in [22u32, 24, 26, 28, 30] {
            for starting_rate in [1usize, 2] {
                let schedule = whir_folding_schedule_with_rate(log2_n, starting_rate);
                let rates = whir_round_log_inv_rates_with_rate(log2_n, starting_rate);
                assert_eq!(rates.len(), schedule.len().saturating_sub(1));
                let fft = log2_n as usize + starting_rate
                    - whir_first_fold_with_rate(log2_n, starting_rate);
                assert!(fft <= KOALA_BEAR_TWO_ADICITY as usize);
                let mut log_inv_rate = starting_rate;
                let mut domain_log = log2_n as usize + starting_rate;
                for (round, &next_rate) in rates.iter().enumerate() {
                    let fold = schedule[round];
                    assert!(
                        domain_log - fold <= KOALA_BEAR_TWO_ADICITY as usize,
                        "nv={log2_n} rate={starting_rate} round={round}: generator {} > two-adicity",
                        domain_log - fold
                    );
                    assert!(next_rate >= 1);
                    assert!(next_rate <= log_inv_rate + fold);
                    let rs = log_inv_rate + fold - next_rate;
                    domain_log -= rs;
                    log_inv_rate = next_rate;
                }
            }
        }
        assert_eq!(whir_round_log_inv_rates(22), vec![4, 7, 10]);
        assert_eq!(whir_round_log_inv_rates(30)[0], 5);
    }

    #[test]
    fn matrix_is_five_payloads_times_eleven_schemes_times_two_threads() {
        let matrix = hash_matrix();
        assert_eq!(matrix.len(), HASH_CELL_COUNT);
        assert_eq!(HashSchemeId::all().len(), HASH_SCHEME_COUNT);
        assert_eq!(HASH_THREADS, [1, 8]);
        assert_eq!(HashSchemeId::Akita.akita_field_arg(), Some("fp32"));
        assert_eq!(HashSchemeId::AkitaFp64.akita_field_arg(), Some("fp64"));
        assert_eq!(HashSchemeId::AkitaFp128.akita_field_arg(), Some("fp128"));
        assert_eq!(
            HashSchemeId::parse_token("akita-fp64"),
            Some(HashSchemeId::AkitaFp64)
        );
        assert_eq!(
            HashSchemeId::parse_token("akita-fp128"),
            Some(HashSchemeId::AkitaFp128)
        );
        for (payload_index, payload) in PAYLOAD_LOG2.iter().enumerate() {
            let base = payload_index * HASH_SCHEME_COUNT * HASH_THREADS.len();
            assert_eq!(matrix[base].scheme, HashSchemeId::Akita);
            assert_eq!(matrix[base].threads, 1);
            assert_eq!(matrix[base + 2].scheme, HashSchemeId::AkitaFp64);
            assert_eq!(matrix[base + 4].scheme, HashSchemeId::AkitaFp128);
            assert_eq!(matrix[base + 6].scheme, HashSchemeId::Plonky2Fri);
            assert_eq!(matrix[base + 12].scheme, HashSchemeId::Whir);
            assert_eq!(matrix[base + 20].scheme, HashSchemeId::Basefold);
            assert!(matrix[base..base + HASH_SCHEME_COUNT * HASH_THREADS.len()]
                .iter()
                .all(|case| case.payload_log2 == *payload));
        }
        assert_eq!(
            HashSchemeId::Whir.commit_url(),
            "https://github.com/Plonky3/Plonky3/commit/9d496524560f3c699473906c6f50fca7cf343730"
        );
        assert_eq!(
            HashSchemeId::Basefold.commit_url(),
            "https://github.com/succinctlabs/sp1/commit/0f2a1e1389747ac0dbee1c4d40243eed20baba86"
        );
        assert_eq!(
            HashSchemeId::Plonky3Fri.revision(),
            HashSchemeId::Plonky3Stir.revision()
        );
    }
}
