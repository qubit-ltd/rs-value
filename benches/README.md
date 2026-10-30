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

## Repeated borrowed DTO validation (2026-10-30)

This focused comparison runs on an Intel Core i5-9600K (6 cores), Ubuntu
24.04 x86_64, with Rust/Cargo 1.94.0. It uses the optimized Criterion bench
profile and the same command for all three runs:

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- repeated_validation --sample-size 50 --warm-up-time 3 --measurement-time 5
```

Each fixture contains 4096 values. The encoded output lengths were 28,721
bytes for strings, 23,526 bytes for finite floats, and 133,005 bytes for JSON
objects with nested numbers. `single` times preflight, one borrowed DTO
construction, and bounded encoding. `repeated` adds a constructed and
discarded borrowed DTO between the same preflight and final DTO construction;
both outputs are asserted equal before timing. Fixture construction is outside
the timed region and both paths use a 4 MiB output limit.

Criterion intervals from each run (lower–upper; values below 1 ms are shown
in microseconds):

| Fixture | Run | Single | Repeated |
| --- | ---: | ---: | ---: |
| strings | 1 | 1.5054–1.9183 ms | 699.94 µs–1.1108 ms |
| strings | 2 | 567.78–793.42 µs | 556.51–846.03 µs |
| strings | 3 | 645.50–854.52 µs | 936.18 µs–1.2867 ms |
| floats | 1 | 1.8496–2.4945 ms | 2.4075–3.2211 ms |
| floats | 2 | 910.16 µs–1.2461 ms | 1.6974–2.3325 ms |
| floats | 3 | 1.3558–1.7379 ms | 1.2526–1.8670 ms |
| json | 1 | 30.692–41.681 ms | 15.335–21.009 ms |
| json | 2 | 39.920–54.924 ms | 26.554–37.858 ms |
| json | 3 | 19.641–25.933 ms | 15.910–19.552 ms |

The comparison must use each run's single and repeated intervals, not Criterion's
`change` estimate against the previous saved baseline. Same-run median ratios
(repeated versus single) were: strings −48.0%, +2.8%, +47.5%; floats +28.5%,
+83.8%, +0.9%; JSON −49.7%, −31.3%, −22.1%. Only the second float run has
non-overlapping intervals and a median extra cost above 5%; the first float
run's intervals overlap slightly, and the third is near parity. JSON's third-run
intervals do not overlap, but repeated construction is faster there. Thus fewer
than two runs support a >5% extra cost with separated intervals, so the design's
screening condition for a real-path follow-up was not met. Absolute times also
drift substantially between runs. These results do not justify a production
change; the repeated-validation performance concern is not confirmed by this
proxy benchmark.

This isolates the cost of an extra value-representation validation. It is not
an end-to-end `rs-config` benchmark: it omits configuration traversal,
property serialization, and the surrounding JSON encoder session and budget
accounting. Raw Criterion output for the final benchmark source was retained at
`/tmp/rv-003-final-bench-1.txt`, `/tmp/rv-003-final-bench-2.txt`, and
`/tmp/rv-003-final-bench-3.txt` on the measurement host.
