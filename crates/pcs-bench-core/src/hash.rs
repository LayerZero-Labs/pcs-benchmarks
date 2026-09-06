//! Hash-based PCS comparison matching the dense-payload evaluation table.

use crate::lattice::{log2_n_for_32bit_payload, FieldSpec, AKITA_FP32, PAYLOAD_LOG2};
use serde::{Deserialize, Serialize};

/// Common transcript-error target for WHIR and BaseFold, in bits.
pub const HASH_SECURITY_BITS: u32 = 128;

/// Thread counts in the hash timing table.
pub const HASH_THREADS: [u32; 2] = [1, 8];

/// KoalaBear prime `2^31 - 2^24 + 1`, used by the WHIR and BaseFold workers.
pub const KOALA_BEAR: FieldSpec = FieldSpec {
    name: "2^{31}-2^{24}+1",
    modulus: 2_130_706_433,
    log2_bits: 31,
};

/// KoalaBear two-adicity; WHIR's first-round fold must keep the FFT in range.
pub const KOALA_BEAR_TWO_ADICITY: u32 = 24;

/// Pinned [Plonky3](https://github.com/Plonky3/Plonky3) revision (`p3-whir`).
pub const PLONKY3_REVISION: &str = "9d496524560f3c699473906c6f50fca7cf343730";

/// Pinned [SP1 / SLOP](https://github.com/succinctlabs/sp1) revision (`slop-basefold`).
pub const SP1_REVISION: &str = "0f2a1e1389747ac0dbee1c4d40243eed20baba86";

/// BaseFold interleaved height. Domain `2^{height+1}` fits KoalaBear two-adicity 24.
pub const BASEFOLD_LOG_STACKING_HEIGHT: u32 = 20;

/// FRI log-inverse rate for BaseFold (`rho = 1/2`).
pub const BASEFOLD_FRI_LOG_BLOWUP: usize = 1;

/// FRI queries for 128-bit conjectured soundness: `log_blowup * queries + pow = 128`.
pub const BASEFOLD_FRI_QUERIES: usize = 112;

/// FRI query proof-of-work bits for BaseFold.
pub const BASEFOLD_FRI_POW_BITS: usize = 16;

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

/// Identifies a hash-eval implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashSchemeId {
    /// Akita at the pinned `main` commit, same fp32-dense catalog as lattice-eval.
    Akita,
    /// Plonky3 `p3-whir` multilinear PCS.
    Whir,
    /// SP1 SLOP stacked BaseFold (`slop-basefold`).
    Basefold,
}

impl HashSchemeId {
    /// Stable table label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Akita => "Akita",
            Self::Whir => "WHIR",
            Self::Basefold => "BaseFold",
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
            "whir" => Some(Self::Whir),
            "basefold" | "base-fold" => Some(Self::Basefold),
            _ => None,
        }
    }

    /// CLI token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Akita => "akita",
            Self::Whir => "whir",
            Self::Basefold => "basefold",
        }
    }

    /// Every scheme in table order.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Akita, Self::Whir, Self::Basefold]
    }

    /// GitHub repository URL without a trailing slash.
    #[must_use]
    pub const fn source_repo(self) -> &'static str {
        match self {
            Self::Akita => "https://github.com/LayerZero-Labs/akita",
            Self::Whir => "https://github.com/Plonky3/Plonky3",
            Self::Basefold => "https://github.com/succinctlabs/sp1",
        }
    }

    /// Pinned git SHA measured for this scheme.
    #[must_use]
    pub const fn revision(self) -> &'static str {
        match self {
            Self::Akita => crate::lattice::AKITA_REVISION,
            Self::Whir => PLONKY3_REVISION,
            Self::Basefold => SP1_REVISION,
        }
    }

    /// Canonical GitHub commit URL for the pinned revision.
    #[must_use]
    pub fn commit_url(self) -> String {
        format!("{}/commit/{}", self.source_repo(), self.revision())
    }

    /// Abbreviated SHA used in tables.
    #[must_use]
    pub fn short_sha(self) -> &'static str {
        let sha = self.revision();
        sha.get(..8).unwrap_or(sha)
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
    /// Native `log2 N` (coefficient count matches the lattice Akita rows).
    pub log2_n: u32,
    /// Worker thread count.
    pub threads: u32,
    /// Compile-time / protocol parameter name.
    pub native_param: &'static str,
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

/// The headline hash matrix (5 payloads × 3 schemes × 2 thread counts).
#[must_use]
pub fn hash_matrix() -> [HashCase; 30] {
    let mut cases = [HashCase {
        payload_log2: 0,
        scheme: HashSchemeId::Akita,
        field: AKITA_FP32,
        log2_n: 0,
        threads: 1,
        native_param: "fp32-dense",
    }; 30];
    let mut index = 0;
    for payload in PAYLOAD_LOG2 {
        let log2_n = log2_n_for_32bit_payload(payload).expect("hash payloads match lattice sizes");
        for scheme in HashSchemeId::all() {
            for threads in HASH_THREADS {
                cases[index] = hash_case_inner(payload, log2_n, scheme, threads);
                index += 1;
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

fn hash_case_inner(payload_log2: u32, log2_n: u32, scheme: HashSchemeId, threads: u32) -> HashCase {
    match scheme {
        HashSchemeId::Akita => HashCase {
            payload_log2,
            scheme,
            field: AKITA_FP32,
            log2_n,
            threads,
            native_param: "fp32-dense",
        },
        HashSchemeId::Whir => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n,
            threads,
            native_param: "whir-128",
        },
        HashSchemeId::Basefold => HashCase {
            payload_log2,
            scheme,
            field: KOALA_BEAR,
            log2_n,
            threads,
            native_param: "basefold-fri-128",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        hash_matrix, whir_first_fold, HashSchemeId, BASEFOLD_FRI_LOG_BLOWUP, BASEFOLD_FRI_POW_BITS,
        BASEFOLD_FRI_QUERIES, HASH_SECURITY_BITS, HASH_THREADS, KOALA_BEAR, KOALA_BEAR_TWO_ADICITY,
    };
    use crate::lattice::{log2_n_for_32bit_payload, PAYLOAD_LOG2};

    #[test]
    fn security_target_is_128_bits() {
        assert_eq!(HASH_SECURITY_BITS, 128);
        assert_eq!(
            BASEFOLD_FRI_LOG_BLOWUP * BASEFOLD_FRI_QUERIES + BASEFOLD_FRI_POW_BITS,
            HASH_SECURITY_BITS as usize
        );
    }

    #[test]
    fn koala_bear_modulus_matches_the_prime() {
        assert_eq!(KOALA_BEAR.modulus, (1u64 << 31) - (1u64 << 24) + 1);
        assert_eq!(KOALA_BEAR_TWO_ADICITY, 24);
    }

    #[test]
    fn hash_log2_n_matches_akita_coefficient_counts() {
        for payload in PAYLOAD_LOG2 {
            assert_eq!(
                hash_matrix()
                    .iter()
                    .find(|case| case.payload_log2 == payload)
                    .map(|case| case.log2_n),
                log2_n_for_32bit_payload(payload)
            );
        }
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
    fn matrix_is_five_payloads_times_three_schemes_times_two_threads() {
        let matrix = hash_matrix();
        assert_eq!(matrix.len(), 30);
        assert_eq!(HASH_THREADS, [1, 8]);
        for (payload_index, payload) in PAYLOAD_LOG2.iter().enumerate() {
            let base = payload_index * 6;
            assert_eq!(matrix[base].scheme, HashSchemeId::Akita);
            assert_eq!(matrix[base].threads, 1);
            assert_eq!(matrix[base + 1].scheme, HashSchemeId::Akita);
            assert_eq!(matrix[base + 1].threads, 8);
            assert_eq!(matrix[base + 2].scheme, HashSchemeId::Whir);
            assert_eq!(matrix[base + 4].scheme, HashSchemeId::Basefold);
            assert!(matrix[base..base + 6]
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
    }
}
