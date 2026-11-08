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

## Natural JSON object projection (RV-001)

`canonical_object/{flat,nested}/{ordered,reverse,shuffled}/{0,1,16,1024}/natural` measures JSON object projection, and `string_map/{0,1,16,1024}/{natural,collection}` measures scalar and collection StringMap projection. Fixtures are built before Criterion starts timing. The standalone `natural_json_allocation_tests` binary compares both paths with a direct `serde_json::Map` build using the same backend and keys/values; counters cover allocator-requested cumulative bytes and peak live requested bytes, not RSS.

The object frame now inserts each completed child into its final map and calls `Map::sort_keys()` when that frame finishes. StringMap retains its sorted borrowed-entry buffer: the direct-map-then-sort variant was withdrawn after a repeated unified-feature regression. This leaves StringMap allocator overhead measurable and avoids a confirmed throughput cost in `preserve_order` builds.

At 4096 entries, the final object-frame projection used 473,144 cumulative / 472,824 peak-live bytes against a 472,376 / 472,376-byte direct-map reference in default, and 533,040 / 532,912 against 532,496 / 532,496 in unified. The remaining 768 B and 544 B cumulative differences are width-independent in these samples. StringMap retains its sorted-entry scratch: default used 578,872 / 578,872 against 421,056 / 421,056 direct, while unified used 638,992 / 638,992 against 573,456 / 573,456. This evidence does not claim StringMap allocation improvement.

Before/after runs used Criterion's optimized profile, rustc 1.94.0, Linux x86_64, 30 samples, 3-second warm-up, and 5-second measurement. Baselines were captured from revision `0140fdb`; the unified run used the detached baseline worktree. Each command used the same selected object and StringMap fixtures:

```text
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- 'canonical_object/flat/ordered/(0|1|16)/natural|canonical_object/flat/reverse/16/natural|canonical_object/flat/shuffled/(16|1024)/natural|canonical_object/nested/shuffled/(16|1024)/natural|string_map/(0|1|16|1024)/(natural|collection)' --sample-size 30 --warm-up-time 3 --measurement-time 5 --save-baseline rv001-stable-before-default
cargo bench --locked --no-default-features --features natural-json --bench downstream_read_bench -- 'canonical_object/flat/ordered/(0|1|16)/natural|canonical_object/flat/reverse/16/natural|canonical_object/flat/shuffled/(16|1024)/natural|canonical_object/nested/shuffled/(16|1024)/natural|string_map/(0|1|16|1024)/(natural|collection)' --sample-size 30 --warm-up-time 3 --measurement-time 5 --baseline rv001-stable-before-default
cargo bench --locked --no-default-features --features 'natural-json,serde_json/arbitrary_precision,serde_json/preserve_order' --bench downstream_read_bench -- 'canonical_object/flat/ordered/(0|1|16)/natural|canonical_object/flat/reverse/16/natural|canonical_object/flat/shuffled/(16|1024)/natural|canonical_object/nested/shuffled/(16|1024)/natural|string_map/(0|1|16|1024)/(natural|collection)' --sample-size 30 --warm-up-time 3 --measurement-time 5 --save-baseline rv001-stable-before-unified
cargo bench --locked --no-default-features --features 'natural-json,serde_json/arbitrary_precision,serde_json/preserve_order' --bench downstream_read_bench -- 'canonical_object/flat/ordered/(0|1|16)/natural|canonical_object/flat/reverse/16/natural|canonical_object/flat/shuffled/(16|1024)/natural|canonical_object/nested/shuffled/(16|1024)/natural|string_map/(0|1|16|1024)/(natural|collection)' --sample-size 30 --warm-up-time 3 --measurement-time 5 --baseline rv001-stable-before-unified
```

Criterion's reported 95% intervals and change ranges for selected Natural JSON object cases are:

| Profile and case | Before | After | Criterion change |
| --- | ---: | ---: | ---: |
| Default, flat ordered 16 | 3.5890–3.5956 µs | 3.3801–3.3841 µs | −6.22% to −5.95%, p<0.001 |
| Default, flat reverse 16 | 3.5953–3.5982 µs | 3.3924–3.3960 µs | −5.89% to −5.73%, p<0.001 |
| Default, flat shuffled 16 | 3.6215–3.6273 µs | 3.4175–3.4236 µs | −6.03% to −5.79%, p<0.001 |
| Default, nested shuffled 16 | 14.297–14.409 µs | 12.395–12.450 µs | −14.38% to −13.34%, p<0.001 |
| Default, flat shuffled 1024 | 359.39–359.49 µs | 328.15–328.87 µs | −8.84% to −8.58%, p<0.001 |
| Default, nested shuffled 1024 | 1.2930–1.3010 ms | 1.1163–1.1192 ms | −14.60% to −13.87%, p<0.001 |
| Unified, flat ordered 16 | 4.6715–4.6763 µs | 4.7283–4.7359 µs | +1.49% to +1.80%, p<0.001 |
| Unified, flat shuffled 16 | 5.0780–5.1212 µs | 5.1923–5.2475 µs | +2.57% to +3.91%; 10% outliers |
| Unified, nested shuffled 16 | 21.019–21.059 µs | 20.969–21.343 µs | +0.19% to +1.79%; 10% outliers |
| Unified, flat shuffled 1024 | 389.51–393.75 µs | 395.55–399.43 µs | +1.43% to +3.03%, p<0.001 |
| Unified, nested shuffled 1024 | 1.4144–1.4161 ms | 1.3749–1.3931 ms | −2.80% to −1.98%; 10% outliers |

The unified flat shuffled 1024 case has a small measured slowdown below the 5% investigation threshold; no tested object case shows a confirmed slowdown above that threshold. The initial 10-sample experiment that directly inserted StringMap entries and then sorted the final map was repeated at 30 samples / 3 seconds / 5 seconds: unified StringMap 1024 natural regressed +25–29%, and collection +28–31% (p<0.001). That variant was removed. The retained StringMap source path is unchanged from the baseline, and its between-run timing varies with the randomized `HashMap` fixture layout; those StringMap deltas are not attributed to this change and do not establish a StringMap gain or regression. Criterion artifacts are under the detached baseline and implementation worktrees' `target/criterion` directories. These synthetic timings do not predict downstream application performance.

## JSON root-leaf identity hashing (RV-001)

The root-leaf optimization bypasses the iterative traversal vectors for JSON
null, booleans, numbers, and strings. Arrays and objects still use the existing
iterative engine. The counting test constructs fixtures and hashers before the
measurement window and measures only `Value::hash`; it warms hashing and the
thread-local allocator first. It compares JSON integer hashing with direct
`serde_json::Number::hash` to distinguish traversal state from number hashing.

The allocation baseline was measured before the change with
`cargo test --locked --features json --test json_identity_allocation_tests -- --nocapture`
and with
`cargo test --locked --features 'all,serde_json/arbitrary_precision,serde_json/preserve_order' --test json_identity_allocation_tests -- --nocapture`.
Both baseline runs intentionally fail the new zero-allocation assertion and
print the measurements. The matching post-change commands pass.

The test also compares hashing the lexical number
`12345678901234567890.123456789` directly with hashing it as `Value::Json`.
In the unified baseline, `Value::Json` allocates 2 times / 120 bytes while
direct Number hashing allocates 0 / 0; after the change, both allocate 0 / 0.
This checks the unified `arbitrary_precision` representation without claiming
that every Number representation is allocation-free.

| Profile | Value hashed | Before: allocations / bytes | After: allocations / bytes | Direct Number: allocations / bytes |
| --- | --- | ---: | ---: | ---: |
| default | null, bool, integer, float, empty/nonempty string (each) | 2 / 160 | 0 / 0 | integer 0 / 0 |
| unified `arbitrary_precision` + `preserve_order` | null, bool, integer, float, empty/nonempty string (each) | 2 / 120 | 0 / 0 | integer 0 / 0 |

The before/after Criterion runs used Rust/Cargo 1.94.0, Linux x86_64, an
Intel Xeon E5-2680 v4, `--all-features`, and 30 samples with 1-second warm-up
and 2-second measurement for every case. The command was:

```text
cargo bench --locked --all-features --bench value_identity_bench -- 'identity/hash_json_' --sample-size 30 --warm-up-time 1 --measurement-time 2
```

Each cell gives Criterion's median estimate and its 95% interval. These are
three complete runs before and after; intervals vary widely, especially on
container cases, so the timing evidence does not support a stable speedup or
regression claim. The measured allocation removal is the reproducible result.

| Case | Before run 1 | Before run 2 | Before run 3 |
| --- | ---: | ---: | ---: |
| null | 58.182 (56.909–60.607 ns) | 56.947 (56.781–57.204 ns) | 57.009 (56.551–57.864 ns) |
| bool | 67.126 (64.025–71.544 ns) | 63.523 (61.534–66.862 ns) | 61.193 (61.068–61.326 ns) |
| integer | 67.340 (67.080–67.765 ns) | 67.037 (66.803–67.308 ns) | 68.572 (66.988–70.955 ns) |
| float | 67.365 (66.895–68.054 ns) | 68.941 (67.731–70.253 ns) | 66.854 (66.737–67.028 ns) |
| empty string | 64.893 (64.674–65.142 ns) | 65.723 (65.305–66.447 ns) | 64.598 (64.577–64.622 ns) |
| nonempty string | 72.538 (71.037–74.532 ns) | 73.784 (71.249–79.023 ns) | 71.456 (70.711–72.525 ns) |
| shallow array | 250.00 (249.61–250.59 ns) | 552.88 (496.13–628.74 ns) | 257.21 (256.68–257.91 ns) |
| nested JSON | 1.4277 (1.4122–1.4510 µs) | 1.9655 (1.6847–2.3480 µs) | 1.4172 (1.4087–1.4279 µs) |

| Case | After run 1 | After run 2 | After run 3 |
| --- | ---: | ---: | ---: |
| null | 21.869 (21.435–22.547 ns) | 32.846 (28.383–37.580 ns) | 31.174 (27.720–35.723 ns) |
| bool | 25.805 (25.705–25.946 ns) | 26.584 (26.527–26.644 ns) | 26.661 (25.900–27.511 ns) |
| integer | 52.518 (45.575–57.627 ns) | 61.728 (58.521–64.699 ns) | 32.866 (31.586–34.936 ns) |
| float | 122.70 (97.936–147.23 ns) | 56.865 (55.713–58.594 ns) | 32.458 (32.015–33.241 ns) |
| empty string | 63.352 (57.731–69.965 ns) | 33.591 (31.852–36.194 ns) | 34.399 (30.902–38.325 ns) |
| nonempty string | 73.966 (67.075–80.775 ns) | 46.505 (41.263–51.240 ns) | 44.252 (40.016–49.302 ns) |
| shallow array | 581.88 (496.70–675.79 ns) | 257.66 (256.72–259.22 ns) | 514.28 (449.95–575.22 ns) |
| nested JSON | 3.8822 (3.3566–4.5037 µs) | 1.4277 (1.4075–1.4654 µs) | 2.8551 (2.6866–3.0510 µs) |

The host had substantial scheduling variation during these measurements. For
example, shallow-array medians span 250–553 ns before and 258–582 ns after,
while nested JSON spans 1.42–1.97 µs before and 1.43–3.88 µs after. Therefore
the fast path is retained for its directly measured removal of two traversal
allocations per root leaf, not on a throughput claim. These synthetic root
fixtures do not establish end-to-end application performance.
