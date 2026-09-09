//! Scheme-neutral definitions used by every benchmark adapter.

mod hash;
mod hash_report;
mod hash_table;
mod lattice;
mod observation;
mod report;
mod rokoko_log;
mod table;
mod workload;

pub use hash::{
    hash_case, hash_matrix, log2_n_for_payload_bits, plonky3_is_packed, plonky3_log_height,
    plonky3_log_width, whir_first_fold, whir_first_fold_with_rate, whir_folding_schedule,
    whir_folding_schedule_with_rate, whir_round_log_inv_rates, whir_round_log_inv_rates_with_rate,
    HashCase, HashSchemeId, BASEFOLD_FRI_LOG_BLOWUP, BASEFOLD_FRI_POW_BITS, BASEFOLD_FRI_QUERIES,
    BASEFOLD_LOG_STACKING_HEIGHT, BINARY_128, BINIUS64_REVISION, FLOCK_BITS, FLOCK_LOG_PACKING,
    FLOCK_REVISION, GOLDILOCKS, HASH_CELL_COUNT, HASH_SCHEME_COUNT, HASH_SECURITY_BITS,
    HASH_SECURITY_BITS_100, HASH_THREADS, KOALA_BEAR, KOALA_BEAR_TWO_ADICITY, PLONKY2_CAP_HEIGHT,
    PLONKY2_FRI_POW_BITS, PLONKY2_FRI_QUERIES, PLONKY2_FRI_RATE_BITS, PLONKY2_REVISION,
    PLONKY3_FRI_POW_BITS, PLONKY3_FRI_QUERIES, PLONKY3_FRI_STIR_REVISION, PLONKY3_REVISION,
    PLONKY3_UNI_LOG_BLOWUP, PROVEKIT_SECURITY_BITS, PROVEKIT_WHIR_FOLD, PROVEKIT_WHIR_LOG_INV_RATE,
    SP1_REVISION, WHIR_DIRECT_SEND_VARS, WHIR_FOLDING_FACTOR, WHIR_MAX_POW_BITS, WHIR_POW_BITS,
    WHIR_PROVEKIT_WHIR_REVISION, WHIR_STARTING_LOG_INV_RATE,
};
pub use hash_report::{render_latex_hash_eval_report, render_markdown_hash_eval_report};
pub use hash_table::{
    aggregate_hash_resource_rows, aggregate_hash_timing_rows, render_latex_hash_resource_table,
    render_latex_hash_timing_table, render_markdown_hash_resource_table,
    render_markdown_hash_timing_table, HashResourceTableRow, HashTimingTableRow,
};
pub use lattice::{
    greyhound_ring_len, lattice_case, lattice_matrix, log2_n_for_32bit_payload,
    rokoko_native_for_payload, worker_memory_limit_bytes, FieldSpec, LatticeCase, SchemeId,
    AKITA_FP128, AKITA_FP32, AKITA_FP64, AKITA_REVISION, GREYHOUND_Q32, GREYHOUND_REVISION,
    GREYHOUND_SIS_POLICY, LATTICE_CELL_COUNT, LATTICE_SCHEME_COUNT, PAYLOAD_LOG2, ROKOKO_Q50,
    ROKOKO_REVISION, THREADS_LATTICE_EVAL, WORKER_RAM_DENOMINATOR, WORKER_RAM_NUMERATOR,
};
pub use observation::{
    looks_like_greyhound_sis, looks_like_oom, HashRecord, LatticeRecord, Observation, Provenance,
    RunStatus, WorkerOutput,
};
pub use report::{render_latex_eval_report, render_markdown_eval_report};
pub use rokoko_log::{parse_rokoko_stdout, RokokoTimings};
pub use table::{
    aggregate_resource_rows, aggregate_timing_rows, format_millis, format_seconds,
    render_latex_resource_table, render_latex_timing_table, render_markdown_resource_table,
    render_markdown_timing_table, GapNote, ResourceTableRow, TimingTableRow,
};
pub use workload::{PolynomialKind, Workload};
