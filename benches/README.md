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

## JSON number DTO validation (2026-11-04)

This comparison measures borrowed payload DTO validation for one JSON array of
ordinary floating-point values, then the same DTO construction plus bounded
encoding. Fixtures contain 256 or 4096 values (`index as f64 / 10.0`), are
constructed outside the timed region, and use a 4 MiB output limit. The test
asserts that the fixture encodes before timing. It uses standard `serde_json`
number storage (`natural-json`, without `arbitrary_precision`).

Each run used the same command before and after the implementation:

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- nested_json_number_validation --sample-size 50 --warm-up-time 3 --measurement-time 5
```

Measurements were made on an Intel Xeon E5-2680 v4, Ubuntu 24.04 x86_64, with
Rust/Cargo 1.94.0. Values are Criterion lower–upper intervals; times below
1 ms are shown in microseconds.

| Fixture | Run | DTO construction | DTO construction + bounded encode |
| --- | ---: | ---: | ---: |
| 256 | baseline 1 | 25.517–31.761 µs | 84.463–100.97 µs |
| 256 | baseline 2 | 22.742–22.868 µs | 74.507–77.384 µs |
| 256 | baseline 3 | 23.242–23.814 µs | 73.632–73.856 µs |
| 4096 | baseline 1 | 477.17–619.85 µs | 1.2429–1.3656 ms |
| 4096 | baseline 2 | 459.10–583.86 µs | 1.5088–1.8669 ms |
| 4096 | baseline 3 | 379.31–394.16 µs | 1.2042–1.3087 ms |

| Fixture | Run | DTO construction | DTO construction + bounded encode |
| --- | ---: | ---: | ---: |
| 256 | optimized 1 | 21.737–21.931 µs | 97.947–129.18 µs |
| 256 | optimized 2 | 21.787–21.926 µs | 71.713–71.995 µs |
| 256 | optimized 3 | 22.312–27.521 µs | 74.369–79.914 µs |
| 4096 | optimized 1 | 385.14–442.47 µs | 1.2025–1.2493 ms |
| 4096 | optimized 2 | 370.02–455.47 µs | 1.1559–1.1571 ms |
| 4096 | optimized 3 | 374.87–402.23 µs | 1.1684–1.1777 ms |

Intervals overlap or improve in the DTO measurements. The combined measurements
vary across runs, but do not show a greater-than-5% regression with separated
intervals in two runs. This supports retaining the stack-buffer change under
the design's decision gate; the timing data is noisy and does not establish an
end-to-end downstream speedup.

The thread-local allocation test measured payload DTO construction after
fixture creation and warm-up. Before the change, integer arrays allocated 288 B
at all three sizes; floating arrays allocated 296 B, 2,336 B, and 33,056 B at
sizes 1, 256, and 4096. After the change, both integer and floating fixtures
allocated 288 B at all three sizes. These numbers include the fixed iterator
frame allocation and show that the per-number temporary string allocation is
gone for ordinary values. `arbitrary_precision` behavior is checked separately;
its number representation may still allocate internally, and long number text
continues to use the allocating fallback. This is not a claim that Wire V1
encoding as a whole is allocation-free.

## Canonical JSON objects (RV-002)

`canonical_object/{flat,nested}/{ordered,reverse,shuffled}/{0,1,16,1024}/{wire,natural}` measures Wire V1 JSON serialization and Natural JSON materialization for objects with distinct values. The deterministic fixture and Wire DTO are built before timing. `shuffled` uses a fixed permutation; with default `serde_json` maps, insertion order is canonicalized by the map backend, while `preserve_order` exercises noncanonical traversal. `nested` places each value in a two-level object.

Capture before/after results separately for default and unified `preserve_order` plus `arbitrary_precision`:

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- canonical_object --sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline rv002-default-before
cargo bench --locked --no-default-features --features 'natural-json,serde_json/preserve_order,serde_json/arbitrary_precision' --bench downstream_read_bench -- canonical_object --sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline rv002-unified-before
# After the implementation, use --baseline rv002-default-before and --baseline rv002-unified-before respectively.
```

Use the same toolchain, machine, profile, input, and Criterion parameters. Criterion timing is noisy and only covers these synthetic JSON-tree operations; treat a small isolated change as inconclusive and investigate repeated stable regressions above 5%. Allocation evidence is reported separately from throughput.

## RV-002 outcome and limits (2026-11-06)

The before/after runs used Rust 1.94.0, Criterion's optimized bench profile, and an Intel Xeon E5-2680 v4 (56 logical CPUs, Ubuntu 24.04 x86_64). The host also ran other Cargo builds during some samples, so short-run outliers and cross-run variance were substantial. Both configurations used 10 samples, 1-second warm-up, and 1-second measurement. A suspected default flat/ordered/1024 Wire regression was repeated with 30 samples, 3-second warm-up, and 5-second measurement.

| Configuration and case | Before interval | After interval | Interpretation |
| --- | ---: | ---: | --- |
| Default, flat ordered 1024 Wire (initial) | 97.6–101.7 µs | 102.8–111.2 µs | Apparent 6.5% median regression; longer repeat was 100.7–105.8 µs, Criterion change interval −1.5% to +4.9%, p=0.40, so not confirmed. |
| Unified features, flat ordered 1024 Wire | 92.2–94.9 µs | 88.4–89.1 µs | About 5% faster in the short comparison; evidence is suggestive, not a broad end-to-end claim. |
| Unified features, nested ordered 1024 Natural JSON (temporary experiment, withdrawn) | 1.25–1.35 ms | 1.25–1.42 ms | No confirmed change (p=0.98). |
| Unified features, flat shuffled 1024 Natural JSON (temporary experiment, withdrawn) | 434–455 µs | 447–513 µs | Confirmed >5% regression signal (Criterion change interval +20% to +44%, p<0.001) on this unordered case. |

The allocation observation isolates 1024-entry Wire object serialization, with fixture construction, DTO construction, and canonical-output validation outside measurement. It uses `serde_json::to_writer(sink(), &wire)`, so output-buffer allocation is excluded. Under default `serde_json`, both map traversals are sorted and the sink measurements are 0 bytes for both inputs. Under unified `preserve_order` + `arbitrary_precision`, sorted input allocates 0 bytes while reverse input allocates 16,384 bytes for its sorting-index vector. Both inputs were validated outside timing to produce the same 16,319-byte canonical output. Earlier `to_json_vec()` totals included the returned output Vec and are not comparable to this isolated measurement, so they are omitted here.

The Wire fast path is retained: it removes the measured sort-index allocation for sorted objects, preserves the existing budget rejection tests, and showed no confirmed throughput regression. The Natural JSON fast path was a temporary local experiment in `canonicalize_json_value`; the benchmark rows above measure that prototype, which was withdrawn before the final commit after the unordered-object regression. The committed code does not contain that path. These are synthetic object measurements; they do not predict full downstream application performance. Raw Criterion reports and samples remain under `target/criterion` on the measurement host.

## Natural JSON wide-array allocation evidence

The dedicated `natural_json_allocation_tests` integration binary measures the public `Value::to_json_value` path with its fixture constructed before the measurement window and the projected result retained through the readout. Its thread-local global allocator records cumulative requested allocation bytes (successful `alloc` plus the new size of successful `realloc`) and peak live requested bytes (alloc/realloc growth less dealloc/realloc old size). A direct `serde_json::Value::Array(vec![Bool(true); n])` build is the required-output reference. These are allocator-requested bytes in the test profile, not RSS or Criterion throughput.

Captured on Linux x86_64 with rustc 1.94.0, `serde_json::Value` size 32 bytes in the default profile and 72 bytes with `arbitrary_precision,preserve_order`. The fixture is an array of `n` booleans; measured sizes are 16, 1024, and 4096. Default profile command: `cargo test --locked --features natural-json --test natural_json_allocation_tests -- --nocapture`. Unified profile command: `cargo test --locked --features 'natural-json,serde_json/arbitrary_precision,serde_json/preserve_order' --test natural_json_allocation_tests -- --nocapture`.

| Feature profile | n | Projection cumulative / peak live (B) | Direct output cumulative / peak live (B) | Projection scratch over output (B) |
| --- | ---: | ---: | ---: | ---: |
| default, before | 16 | 2,016 / 1,296 | 512 / 512 | 1,504 / 784 |
| default, before | 1,024 | 114,912 / 81,936 | 32,768 / 32,768 | 82,144 / 49,168 |
| default, before | 4,096 | 458,976 / 327,696 | 131,072 / 131,072 | 327,904 / 196,624 |
| default, after | 16 | 1,280 / 960 | 512 / 512 | 768 / 448 |
| default, after | 1,024 | 33,536 / 33,216 | 32,768 / 32,768 | 768 / 448 |
| default, after | 4,096 | 131,840 / 131,520 | 131,072 / 131,072 | 768 / 448 |
| unified, before | 16 | 3,584 / 2,576 | 1,152 / 1,152 | 2,432 / 1,424 |
| unified, before | 1,024 | 237,440 / 163,856 | 73,728 / 73,728 | 163,712 / 90,128 |
| unified, before | 4,096 | 950,144 / 655,376 | 294,912 / 294,912 | 655,232 / 360,464 |
| unified, after | 16 | 1,504 / 1,376 | 1,152 / 1,152 | 352 / 224 |
| unified, after | 1,024 | 74,080 / 73,952 | 73,728 / 73,728 | 352 / 224 |
| unified, after | 4,096 | 295,264 / 295,136 | 294,912 / 294,912 | 352 / 224 |

At 4096 items, cumulative / peak-live bytes fell by 327,136 / 196,176 B in the default profile and 654,880 / 360,240 B in the unified profile. Scratch beyond the direct output reference is width-independent in the measured samples after the change. The regression test checks that the 4096-element scratch stays within two `Value` slots of the 16-element scratch; this bound fails on the unchanged implementation. The figures establish allocator traffic for this synthetic boolean-array projection only. They do not establish process RSS or general JSON workloads.

The array Criterion comparison used the same baseline source revision (`6f5ae6e`) and the same benchmark fixture in a detached temporary worktree for the before run, then the implementation worktree for the after run. Commands used `--sample-size 10 --warm-up-time 1 --measurement-time 1 --save-baseline`; features were `natural-json` and, separately, `natural-json,serde_json/arbitrary_precision,serde_json/preserve_order`. The fixture is created outside the timed iteration and returns the projected value through `black_box`. Times are Criterion 95% intervals:

| Profile | Items | Before | After | Median change |
| --- | ---: | ---: | ---: | ---: |
| Default | 16 | 884.33–887.40 ns | 729.38–734.05 ns | −17.3% |
| Default | 1,024 | 37.103–37.520 µs | 34.546–34.631 µs | −7.2% |
| Default | 4,096 | 146.58–146.68 µs | 137.90–137.96 µs | −6.0% |
| Unified | 16 | 1.0146–1.0178 µs | 797.15–797.92 ns | −21.5% |
| Unified | 1,024 | 47.128–47.212 µs | 41.559–41.594 µs | −11.8% |
| Unified | 4,096 | 190.95–191.32 µs | 170.19–172.06 µs | −10.6% |

The object comparison re-ran the natural projection cases most relevant to canonical sorting. The before and after runs each used the same 10-sample, 1-second warm-up and measurement settings. The table reports Criterion 95% slope intervals in milliseconds; object fixture creation remains outside timing.

| Profile | Object case, 1,024 entries | Before | After |
| --- | --- | ---: | ---: |
| Default | Flat, ordered | 376.3–389.4 µs | 378.9–379.8 µs |
| Default | Nested, ordered | 1.311–1.364 ms | 1.272–1.273 ms |
| Default | Flat, shuffled | 414.5–419.6 µs | 373.1–373.4 µs |
| Unified | Flat, ordered | 309.9–352.1 µs | 294.5–295.2 µs |
| Unified | Nested, ordered | 1.363–1.434 ms | 1.208–1.224 ms |
| Unified | Flat, shuffled | 431.9–450.5 µs | 403.9–405.3 µs |

These synthetic runs showed no repeatable object slowdown; the default flat/ordered intervals overlap, while the other listed after intervals are lower. Natural-array timing also improved in these runs, but neither result predicts process RSS or end-to-end application performance. Criterion measurements are host- and load-sensitive.
