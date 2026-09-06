//! Lattice PCS comparison matrix matching the dense-payload evaluation table.

use serde::{Deserialize, Serialize};

/// Fraction of host RAM each worker may use (`ulimit -v`).
pub const WORKER_RAM_NUMERATOR: u64 = 9;

/// Denominator of [`WORKER_RAM_NUMERATOR`].
pub const WORKER_RAM_DENOMINATOR: u64 = 10;

/// Worker virtual-memory ceiling: 90% of host RAM, leaving the rest for the OS.
#[must_use]
pub const fn worker_memory_limit_bytes(host_ram_bytes: u64) -> u64 {
    host_ram_bytes.saturating_mul(WORKER_RAM_NUMERATOR) / WORKER_RAM_DENOMINATOR
}

/// Lattice-eval is exclusively single-threaded. RoKoKo has no native
/// multithreaded prover. Greyhound can parallelize extension products; the
/// worker pins `LATTICE_DOGS_THREADS=1` so the ratio is not a scaling artifact.
pub const THREADS_LATTICE_EVAL: u32 = 1;

/// Target payload exponents `N log_2 |F|` in the headline table.
pub const PAYLOAD_LOG2: [u32; 5] = [27, 29, 31, 33, 35];

/// A named prime field used by a lattice PCS implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldSpec {
    /// Display name used in tables, for example `2^{32}-99`.
    pub name: &'static str,
    /// Prime modulus.
    pub modulus: u64,
    /// `round(log2(modulus))` used to convert payload bits into `log2 N`.
    pub log2_bits: u32,
}

/// Akita and Greyhound dense comparison field `q = 2^32 - 99`.
pub const AKITA_FP32: FieldSpec = FieldSpec {
    name: "2^{32}-99",
    modulus: 4_294_967_197,
    log2_bits: 32,
};

/// Greyhound Labrador default `LOGQ=32`, `QOFF=99` — the same prime as Akita fp32.
pub const GREYHOUND_Q32: FieldSpec = AKITA_FP32;

/// RoKoKo native field `q = 2^50 - 2687`.
pub const ROKOKO_Q50: FieldSpec = FieldSpec {
    name: "2^{50}-2687",
    modulus: 1_125_899_906_839_937,
    log2_bits: 50,
};

/// Pinned Akita revision on `main` (immutable git SHA).
pub const AKITA_REVISION: &str = "f9f7de87bcf230436193dbf6ba5a3bdc077b8f53";

/// Last commit on [PR #466](https://github.com/LayerZero-Labs/akita/pull/466).
pub const AKITA_PR466_REVISION: &str = "bb68275e90ea280c19ad572b1653724a04656740";

/// GitHub pull request that `AKITA_PR466_REVISION` belongs to.
pub const AKITA_PR466_URL: &str = "https://github.com/LayerZero-Labs/akita/pull/466";

/// Pinned Greyhound reference revision (`LayerZero-Labs/greyhound-reference`).
pub const GREYHOUND_REVISION: &str = "687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397";

/// Euclidean SIS policy used by the Greyhound lattice-eval worker.
pub const GREYHOUND_SIS_POLICY: &str = "l2-quantum128-adps16";

/// Pinned RoKoKo revision.
pub const ROKOKO_REVISION: &str = "1baa91e901fc37b5fa59e65c26a630cb93849b3e";

/// Identifies a lattice PCS implementation in the comparison harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemeId {
    /// Akita at the pinned `main` commit.
    Akita,
    /// Akita at the tip of PR #466 (quotient-free ring relations).
    AkitaPr466,
    /// Greyhound Pack (`LayerZero-Labs/greyhound-reference`).
    Greyhound,
    /// RoKoKo PCS chain (`lattice-arguments/rokoko`).
    Rokoko,
}

impl SchemeId {
    /// Stable table label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Akita => "Akita",
            Self::AkitaPr466 => "Akita (#466)",
            Self::Greyhound => "Greyhound",
            Self::Rokoko => "RoKoKo",
        }
    }

    /// Table label with LaTeX escaping for `#`.
    #[must_use]
    pub const fn latex_name(self) -> &'static str {
        match self {
            Self::AkitaPr466 => r"Akita (\#466)",
            other => other.display_name(),
        }
    }

    /// Parse a CLI scheme token.
    #[must_use]
    pub fn parse_token(token: &str) -> Option<Self> {
        match token {
            "akita" => Some(Self::Akita),
            "akita-pr466" | "akita_pr466" => Some(Self::AkitaPr466),
            "greyhound" => Some(Self::Greyhound),
            "rokoko" | "ro-koko" => Some(Self::Rokoko),
            _ => None,
        }
    }

    /// CLI token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Akita => "akita",
            Self::AkitaPr466 => "akita-pr466",
            Self::Greyhound => "greyhound",
            Self::Rokoko => "rokoko",
        }
    }

    /// Every scheme in table order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [Self::Akita, Self::AkitaPr466, Self::Greyhound, Self::Rokoko]
    }

    /// GitHub repository URL without a trailing slash.
    #[must_use]
    pub const fn source_repo(self) -> &'static str {
        match self {
            Self::Akita | Self::AkitaPr466 => "https://github.com/LayerZero-Labs/akita",
            Self::Greyhound => "https://github.com/LayerZero-Labs/greyhound-reference",
            Self::Rokoko => "https://github.com/lattice-arguments/rokoko",
        }
    }

    /// Pinned git SHA measured for this scheme.
    #[must_use]
    pub const fn revision(self) -> &'static str {
        match self {
            Self::Akita => AKITA_REVISION,
            Self::AkitaPr466 => AKITA_PR466_REVISION,
            Self::Greyhound => GREYHOUND_REVISION,
            Self::Rokoko => ROKOKO_REVISION,
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

/// One cell in the dense lattice comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LatticeCase {
    /// Target payload exponent.
    pub payload_log2: u32,
    /// Scheme.
    pub scheme: SchemeId,
    /// Implementation field.
    pub field: FieldSpec,
    /// Native `log2 N` when the scheme has a matching instance.
    pub log2_n: Option<u32>,
    /// Compile-time parameter name (`fp32-dense`, `p-26`, ...).
    pub native_param: Option<&'static str>,
    /// Why `log2_n` is `None`.
    pub unsupported_reason: Option<&'static str>,
}

/// Greyhound / Akita `log2 N` for a 32-bit field at the given payload.
#[must_use]
pub const fn log2_n_for_32bit_payload(payload_log2: u32) -> Option<u32> {
    payload_log2.checked_sub(AKITA_FP32.log2_bits.trailing_zeros())
}

/// Closest RoKoKo native degree feature for a target payload, if any.
#[must_use]
pub const fn rokoko_native_for_payload(payload_log2: u32) -> Option<(u32, &'static str)> {
    match payload_log2 {
        31 => Some((26, "p-26")),
        33 => Some((28, "p-28")),
        35 => Some((30, "p-30")),
        _ => None,
    }
}

/// Ring-element count Greyhound `polcom_commit` expects for `log2 N` coefficients.
///
/// Labrador stores the polynomial as `len` elements of `Z_q[X]/(X^{64}+1)`, so
/// `N_coeff = len * 64`.
#[must_use]
pub const fn greyhound_ring_len(log2_n: u32) -> Option<usize> {
    if log2_n < 6 {
        return None;
    }
    1usize.checked_shl(log2_n - 6)
}

/// The headline matrix (5 payloads × 4 implementations).
#[must_use]
pub fn lattice_matrix() -> [LatticeCase; 20] {
    let mut cases = [LatticeCase {
        payload_log2: 0,
        scheme: SchemeId::Akita,
        field: AKITA_FP32,
        log2_n: None,
        native_param: None,
        unsupported_reason: None,
    }; 20];
    for (payload_index, payload) in PAYLOAD_LOG2.iter().enumerate() {
        let index = payload_index * 4;
        cases[index] = akita_or_greyhound_case(*payload, SchemeId::Akita);
        cases[index + 1] = akita_or_greyhound_case(*payload, SchemeId::AkitaPr466);
        cases[index + 2] = akita_or_greyhound_case(*payload, SchemeId::Greyhound);
        cases[index + 3] = rokoko_case(*payload);
    }
    cases
}

/// Look up one cell.
#[must_use]
pub fn lattice_case(payload_log2: u32, scheme: SchemeId) -> Option<LatticeCase> {
    lattice_matrix()
        .into_iter()
        .find(|case| case.payload_log2 == payload_log2 && case.scheme == scheme)
}

fn akita_or_greyhound_case(payload_log2: u32, scheme: SchemeId) -> LatticeCase {
    let native_param = match scheme {
        SchemeId::Akita | SchemeId::AkitaPr466 => Some("fp32-dense"),
        SchemeId::Greyhound => Some("pack-q32"),
        SchemeId::Rokoko => None,
    };
    LatticeCase {
        payload_log2,
        scheme,
        field: AKITA_FP32,
        log2_n: log2_n_for_32bit_payload(payload_log2),
        native_param,
        unsupported_reason: None,
    }
}

fn rokoko_case(payload_log2: u32) -> LatticeCase {
    match rokoko_native_for_payload(payload_log2) {
        Some((log2_n, feature)) => LatticeCase {
            payload_log2,
            scheme: SchemeId::Rokoko,
            field: ROKOKO_Q50,
            log2_n: Some(log2_n),
            native_param: Some(feature),
            unsupported_reason: None,
        },
        None => LatticeCase {
            payload_log2,
            scheme: SchemeId::Rokoko,
            field: ROKOKO_Q50,
            log2_n: None,
            native_param: None,
            unsupported_reason: Some(
                "RoKoKo ships fixed native sets p-26, p-28, and p-30 only; no instance matches this payload",
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        greyhound_ring_len, lattice_matrix, log2_n_for_32bit_payload, rokoko_native_for_payload,
        worker_memory_limit_bytes, SchemeId, PAYLOAD_LOG2, ROKOKO_Q50, WORKER_RAM_DENOMINATOR,
        WORKER_RAM_NUMERATOR,
    };

    #[test]
    fn worker_memory_limit_is_ninety_percent_of_host_ram() {
        assert_eq!(WORKER_RAM_NUMERATOR, 9);
        assert_eq!(WORKER_RAM_DENOMINATOR, 10);
        let host = 121 * 1024 * 1024 * 1024;
        assert_eq!(worker_memory_limit_bytes(host), host * 9 / 10);
        assert_eq!(worker_memory_limit_bytes(0), 0);
    }

    #[test]
    fn payload_to_akita_sizes_match_the_paper_table() {
        assert_eq!(log2_n_for_32bit_payload(27), Some(22));
        assert_eq!(log2_n_for_32bit_payload(29), Some(24));
        assert_eq!(log2_n_for_32bit_payload(31), Some(26));
        assert_eq!(log2_n_for_32bit_payload(33), Some(28));
        assert_eq!(log2_n_for_32bit_payload(35), Some(30));
    }

    #[test]
    fn rokoko_native_sets_are_the_closest_supported_payloads() {
        assert_eq!(rokoko_native_for_payload(27), None);
        assert_eq!(rokoko_native_for_payload(29), None);
        assert_eq!(rokoko_native_for_payload(31), Some((26, "p-26")));
        assert_eq!(rokoko_native_for_payload(33), Some((28, "p-28")));
        assert_eq!(rokoko_native_for_payload(35), Some((30, "p-30")));
    }

    #[test]
    fn rokoko_native_instances_carry_about_25_16_times_the_32bit_payload() {
        // 50 / 32 = 25/16. A RoKoKo row at log2 N = k reports k+50 logical bits
        // versus the 32-bit target of k+32.
        let ratio_times_16 = (ROKOKO_Q50.log2_bits * 16) / 32;
        assert_eq!(ratio_times_16, 25);
    }

    #[test]
    fn greyhound_len_is_coefficients_over_ring_degree_64() {
        assert_eq!(greyhound_ring_len(22), Some(1 << 16));
        assert_eq!(greyhound_ring_len(26), Some(1 << 20));
        assert_eq!(greyhound_ring_len(5), None);
    }

    #[test]
    fn matrix_is_five_payloads_times_four_implementations() {
        let matrix = lattice_matrix();
        assert_eq!(matrix.len(), 20);
        for (index, payload) in PAYLOAD_LOG2.iter().enumerate() {
            let base = index * 4;
            assert_eq!(matrix[base].scheme, SchemeId::Akita);
            assert_eq!(matrix[base + 1].scheme, SchemeId::AkitaPr466);
            assert_eq!(matrix[base + 2].scheme, SchemeId::Greyhound);
            assert_eq!(matrix[base + 3].scheme, SchemeId::Rokoko);
            assert!(matrix[base..base + 4]
                .iter()
                .all(|case| case.payload_log2 == *payload));
        }
        let unsupported = matrix.iter().filter(|case| case.log2_n.is_none()).count();
        assert_eq!(unsupported, 2);
        assert_eq!(
            SchemeId::AkitaPr466.commit_url(),
            "https://github.com/LayerZero-Labs/akita/commit/bb68275e90ea280c19ad572b1653724a04656740"
        );
        assert_eq!(
            SchemeId::Greyhound.commit_url(),
            "https://github.com/LayerZero-Labs/greyhound-reference/commit/687a6f8be1dbc5bf1fa3927bb4a0a8d1e84d8397"
        );
    }
}
