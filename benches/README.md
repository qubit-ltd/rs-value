# Benchmarks

`downstream_read_bench` measures the read path that downstream crates use when
checking and encoding a value for Wire V1. The `preflight/`,
`borrowed_encode/`, and `preflight_and_encode/` groups each cover 1, 32, and
4096 string values. The fixture budget is deliberately larger than the 4096
item output so a benchmark measures the operation rather than a rejected
limit.

Run the focused benchmark with:

```text
cargo bench --all-features --bench downstream_read_bench
```

Criterion output is the performance evidence for this change. A benchmark
run that is not available in the current environment must be reported as
unavailable; it must not be treated as evidence of an improvement.
