# Task T3C Report: Wire Coverage

## Scope

- Added public-boundary tests for V1 payload and envelope Serde seeds.
- Added exact owned/borrowed payload and envelope JSON golden assertions.
- Covered default and explicit encode limits, writer failures, and output-budget failures.
- Covered bounded decoding of empty input, invalid UTF-8, an invalid top-level shape, and trailing data.
- Covered direct conversion of measured budget and quantity failures into `ValueWireEncodeError`.
- Changed no production code and preserved the existing Wire V1 representation.

## Initial Coverage Gaps

The pre-change `target/llvm-cov/coverage.json` identified these primary Wire gaps:

| Source | Functions | Lines | Regions | Missing-line focus |
| --- | ---: | ---: | ---: | --- |
| `value_wire_encode_error.rs` | 1/2 | 8/15 | 13/24 | measured budget conversion |
| `value_wire_payload_v1.rs` | 11/17 | 50/79 | 61/95 | limits, writer, conversion entry points |
| `value_wire_payload_v1_seed.rs` | 1/2 | 6/9 | 6/10 | seed construction |
| `value_wire_ref_v1.rs` | 9/12 | 35/55 | 45/66 | vec/writer entry points |
| `value_wire_v1.rs` | 8/16 | 47/71 | 51/84 | owned DTO entry points |
| `strict_string_map.rs` | 3/4 | 18/21 | 24/30 | visitor diagnostic path |

The coverage artifact also showed that existing all-type golden tests already exercise every
V1 data-type tag, scalar/collection shape, unset representation, finite-float policy, and
canonical map/JSON serialization. The new tests therefore target uncovered entry points and
error classifications instead of duplicating that matrix.

## RED / GREEN Evidence

- RED: `cargo test --locked --all-features --test integration_tests value_wire_v1_seed_preserves_the_golden_envelope -- --nocapture`
  failed with actual `Scalar(String("ready"))` versus the deliberately incorrect
  characterization golden `Scalar(String("not-ready"))`.
- FIRST_CHECK: the focused Wire suite initially reported 87 passed and 1 failed, revealing the
  decoder's exact boundary categories: empty input is `Syntax`, while a valid top-level array is
  `InvalidJson`.
- GREEN: `cargo test --locked --all-features --test integration_tests value_wire -- --nocapture`
  passed 88 tests.
- GREEN: `cargo test --locked --all-features --test integration_tests wire -- --nocapture`
  passed 116 tests.
- GREEN: `cargo test --locked --all-features --test integration_tests` passed 700 tests with 2
  ignored and no failures.

## Self-review

- Every changed Rust path is within the T3C Wire-test write set.
- All new test functions use the required `test_` prefix and test one observable contract.
- Error assertions inspect concrete variants, resources, and writer details where observable.
- Exact JSON byte assertions retain the existing V1 field order, tags, and shape.
- No production source, feature definition, coverage exemption, or documentation was changed.
