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
    hash_case, hash_matrix, whir_first_fold, whir_first_fold_with_rate, whir_folding_schedule,
    whir_folding_schedule_with_rate, whir_round_log_inv_rates, whir_round_log_inv_rates_with_rate,
    HashCase, HashSchemeId, BASEFOLD_FRI_LOG_BLOWUP, BASEFOLD_FRI_POW_BITS, BASEFOLD_FRI_QUERIES,
    BASEFOLD_LOG_STACKING_HEIGHT, HASH_SECURITY_BITS, HASH_THREADS, KOALA_BEAR,
    KOALA_BEAR_TWO_ADICITY, PLONKY3_REVISION, SP1_REVISION, WHIR_DIRECT_SEND_VARS,
    WHIR_FOLDING_FACTOR, WHIR_MAX_POW_BITS, WHIR_POW_BITS, WHIR_STARTING_LOG_INV_RATE,
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
    AKITA_FP32, AKITA_PR466_REVISION, AKITA_PR466_URL, AKITA_REVISION, GREYHOUND_Q32,
    GREYHOUND_REVISION, PAYLOAD_LOG2, ROKOKO_Q50, ROKOKO_REVISION, THREADS_LATTICE_EVAL,
    WORKER_RAM_DENOMINATOR, WORKER_RAM_NUMERATOR,
};
pub use observation::{
    looks_like_greyhound_sis, looks_like_oom, HashRecord, LatticeRecord, Observation, Provenance,
    RunStatus, WorkerOutput, RESULT_SCHEMA_VERSION,
};
pub use report::{render_latex_eval_report, render_markdown_eval_report};
pub use rokoko_log::{parse_rokoko_stdout, RokokoTimings};
pub use table::{
    aggregate_resource_rows, aggregate_timing_rows, format_millis, format_seconds,
    render_latex_resource_table, render_latex_timing_table, render_markdown_resource_table,
    render_markdown_timing_table, GapNote, ResourceTableRow, TimingTableRow,
};
pub use workload::{PolynomialKind, Workload};
