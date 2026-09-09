//! Parse RoKoKo's native executor stdout into phase timings.

use std::collections::BTreeMap;

/// Timings extracted from a RoKoKo PCS-chain run.
#[derive(Clone, Debug, PartialEq)]
pub struct RokokoTimings {
    /// Phase timings in nanoseconds.
    pub timings_ns: BTreeMap<String, u64>,
    /// Wire or encoded proof size when the executor printed one.
    pub proof_bytes: Option<u64>,
    /// Inner recursive commitment size when the executor printed one.
    pub commitment_bytes: Option<u64>,
    /// Opening evaluations sent separately from the proof.
    pub evaluation_bytes: Option<u64>,
    /// Expanded prover CRS resident size when the executor printed one.
    pub state_bytes: Option<u64>,
    /// Process peak RSS when the executor printed one.
    pub peak_rss_bytes: Option<u64>,
}

/// Parse `TOTAL … time` and size lines from RoKoKo's executor.
#[must_use]
pub fn parse_rokoko_stdout(stdout: &str) -> Option<RokokoTimings> {
    let mut timings_ns = BTreeMap::new();
    let mut proof_bytes = None;
    let mut commitment_bytes = None;
    let mut evaluation_bytes = None;
    let mut state_bytes = None;
    let mut peak_rss_bytes = None;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(ns) = parse_total_ns(line, "TOTAL CRS gen time:") {
            timings_ns.insert("setup".into(), ns);
        } else if let Some(ns) = parse_total_ns(line, "TOTAL Commit time:") {
            timings_ns.insert("commit".into(), ns);
        } else if let Some(ns) = parse_total_ns(line, "TOTAL Prover time:") {
            timings_ns.insert("open".into(), ns);
        } else if let Some(ns) = parse_total_ns(line, "TOTAL Verifier time:") {
            timings_ns.insert("verify".into(), ns);
        } else if let Some(bytes) = parse_bytes_line(line, "TOTAL Commitment size:") {
            commitment_bytes = Some(bytes);
        } else if let Some(bytes) = parse_bytes_line(line, "TOTAL Evaluation size:") {
            evaluation_bytes = Some(bytes);
        } else if let Some(bytes) = parse_bytes_line(line, "TOTAL CRS size:") {
            state_bytes = Some(bytes);
        } else if let Some(bytes) = parse_bytes_line(line, "Peak RSS:") {
            peak_rss_bytes = Some(bytes);
        } else if let Some(bytes) = parse_kb_line(line, "Wire proof size:") {
            proof_bytes = Some(bytes);
        } else if proof_bytes.is_none() {
            if let Some(bytes) = parse_kb_line(line, "Total proof size:") {
                proof_bytes = Some(bytes);
            }
        }
    }

    if timings_ns.contains_key("commit")
        && timings_ns.contains_key("open")
        && timings_ns.contains_key("verify")
    {
        Some(RokokoTimings {
            timings_ns,
            proof_bytes,
            commitment_bytes,
            evaluation_bytes,
            state_bytes,
            peak_rss_bytes,
        })
    } else {
        None
    }
}

fn parse_total_ns(line: &str, prefix: &str) -> Option<u64> {
    let rest = line.strip_prefix(prefix)?.trim();
    let number = rest.strip_suffix("ns")?.trim();
    number.parse().ok()
}

fn parse_kb_line(line: &str, prefix: &str) -> Option<u64> {
    let rest = line.strip_prefix(prefix)?;
    let token = rest.split_whitespace().next()?;
    let kib: f64 = token.parse().ok()?;
    Some((kib * 1024.0).round() as u64)
}

fn parse_bytes_line(line: &str, prefix: &str) -> Option<u64> {
    let rest = line.strip_prefix(prefix)?.trim();
    let token = rest.split_whitespace().next()?;
    token.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::parse_rokoko_stdout;

    #[test]
    fn parses_executor_summary_lines() {
        let stdout = "\
Using p26...
TOTAL CRS gen time: 111 ns
TOTAL CRS size: 1048576 bytes
TOTAL Commit time: 2560000000 ns
TOTAL Commitment size: 3072 bytes
TOTAL Prover time: 1820000000 ns
TOTAL Evaluation size: 512 bytes
Total proof size: 157 KB
Wire proof size: 157.0 KB (serialise 1.200 ms, deserialise 0.800 ms)
TOTAL Verifier time: 11600000 ns
Peak RSS: 2147483648 bytes
";
        let parsed = parse_rokoko_stdout(stdout).expect("timings");
        assert_eq!(parsed.timings_ns.get("setup"), Some(&111));
        assert_eq!(parsed.timings_ns.get("commit"), Some(&2_560_000_000));
        assert_eq!(parsed.timings_ns.get("open"), Some(&1_820_000_000));
        assert_eq!(parsed.timings_ns.get("verify"), Some(&11_600_000));
        assert_eq!(parsed.proof_bytes, Some(160_768));
        assert_eq!(parsed.commitment_bytes, Some(3072));
        assert_eq!(parsed.evaluation_bytes, Some(512));
        assert_eq!(parsed.state_bytes, Some(1_048_576));
        assert_eq!(parsed.peak_rss_bytes, Some(2_147_483_648));
    }
}
