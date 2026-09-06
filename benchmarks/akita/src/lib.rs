//! Akita benchmark adapter.
//!
//! End-to-end Criterion benchmarks live in `benches/e2e.rs`. Keeping this
//! crate separate prevents future PCS dependencies and feature sets from
//! contaminating one another.

/// Exact Akita source revision measured by this adapter.
pub const IMPLEMENTATION_REVISION: &str = "f9f7de87bcf230436193dbf6ba5a3bdc077b8f53";
