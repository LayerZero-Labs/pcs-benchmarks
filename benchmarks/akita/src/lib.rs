//! Akita benchmark adapter.
//!
//! End-to-end Criterion benchmarks live in `benches/e2e.rs`. Keeping this
//! crate separate prevents future PCS dependencies and feature sets from
//! contaminating one another.

/// Exact Akita source revision measured by this adapter.
pub const IMPLEMENTATION_REVISION: &str = "d1b224d809c7edc357b0dbab0f607e19b475910b";
