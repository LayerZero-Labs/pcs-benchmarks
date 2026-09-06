//! Logical PCS workloads shared across adapters.

use serde::{Deserialize, Serialize};

/// Polynomial representation supplied to a PCS.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolynomialKind {
    /// Arbitrary evaluations over the full multilinear hypercube.
    Dense,
    /// Exactly one non-zero entry in each fixed-size chunk.
    OneHot,
}

/// A logical workload that adapters must implement equivalently.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Workload {
    /// Stable cross-scheme identifier.
    pub id: String,
    /// Base-two logarithm of the polynomial length.
    pub log2_size: u32,
    /// Number of polynomials committed in the batch.
    pub polynomial_count: u32,
    /// Number of opening points.
    pub opening_count: u32,
    /// Input representation.
    pub polynomial_kind: PolynomialKind,
    /// Required claimed security level.
    pub security_bits: u32,
    /// Seed used for deterministic input generation.
    pub seed: u64,
}

impl Workload {
    /// Returns the number of field elements in each polynomial.
    #[must_use]
    pub fn polynomial_len(&self) -> Option<usize> {
        1usize.checked_shl(self.log2_size)
    }
}

#[cfg(test)]
mod tests {
    use super::{PolynomialKind, Workload};

    #[test]
    fn computes_polynomial_length() {
        let workload = Workload {
            id: "dense-16".into(),
            log2_size: 16,
            polynomial_count: 1,
            opening_count: 1,
            polynomial_kind: PolynomialKind::Dense,
            security_bits: 128,
            seed: 42,
        };
        assert_eq!(workload.polynomial_len(), Some(65_536));
    }

    #[test]
    fn rejects_unrepresentable_polynomial_length() {
        let workload = Workload {
            id: "too-large".into(),
            log2_size: usize::BITS,
            polynomial_count: 1,
            opening_count: 1,
            polynomial_kind: PolynomialKind::Dense,
            security_bits: 128,
            seed: 42,
        };
        assert_eq!(workload.polynomial_len(), None);
    }
}
