# Benchmarks

`downstream_read_bench` measures the read path that downstream crates use when
checking and encoding a value for Wire V1. The `preflight/`,
`borrowed_encode/`, and `preflight_and_encode/` groups each cover 1, 32, and
4096 string values. The fixture budget is deliberately larger than the 4096
item output so a benchmark measures the operation rather than a rejected
limit.

The groups measure these steps:

| Group | Timed work |
| --- | --- |
| `preflight/` | Explicit payload preflight only. |
| `borrowed_encode/` | DTO construction happens before timing; the timed helper runs its internal preflight and encodes the borrowed DTO. |
| `preflight_and_encode/` | Explicit preflight, borrowed DTO construction, the helper's internal preflight, and encoding all happen during timing. |

The difference between `preflight_and_encode/` and `borrowed_encode/` includes
borrowed DTO construction as well as the explicit preflight. It is not a
measurement of pure preflight cost.

Run the focused benchmark with:

```text
cargo bench --all-features --bench downstream_read_bench
```

Criterion output is the performance evidence for this change. A benchmark
run that is not available in the current environment must be reported as
unavailable; it must not be treated as evidence of an improvement.
