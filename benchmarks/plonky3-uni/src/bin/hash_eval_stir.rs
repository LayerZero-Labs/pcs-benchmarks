//! Single-shot Plonky3 univariate STIR worker.

fn main() -> std::process::ExitCode {
    pcs_bench_plonky3_uni::main_for(pcs_bench_plonky3_uni::UniKind::Stir)
}
