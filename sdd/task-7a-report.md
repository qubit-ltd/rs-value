# Task T7A report: budget and Wire coverage gaps

## Scope

- Added public-boundary tests for natural JSON projection budgets, Wire
  preflight construction and traversal, Wire encode/decode error variants, the
  payload seed error path, and recursive canonical JSON arrays.
- Added private boundary tests for the three preflight counter-overflow guards;
  those states cannot be constructed through the public API without attempting
  impossible allocations.
- Changed no production behavior and did not modify `.rs-ci-coverage.json`.

## RED

The fresh baseline coverage run executed the full all-feature test suite but
failed the unchanged crate-wide gate:

```text
functions=142/157, lines=875/1048, regions=1190/1536
required: functions >= 95%, lines > 90%, regions > 85%
```

The largest relevant gap was
`src/value_wire/value_wire_encode_preflight.rs` at 21/28 functions, 210/276
lines, and 299/526 regions.

## FIRST_CHECK

The focused public preflight suite passed after adding exact point and
cumulative budget assertions:

```text
cargo test --locked --all-features --test integration_tests \
  value_wire_encode_preflight -- --nocapture
13 passed; 0 failed
```

The first follow-up coverage run improved the gate substantially but exposed a
single remaining function shortfall:

```text
functions=149/157, lines=991/1048, regions=1449/1536
```

The only reachable missing functions in preflight were the three private
`checked_add` overflow closures. The payload seed's zero-sized `const`
`#[inline(always)]` constructor did not emit a countable LLVM coverage region,
even when invoked indirectly, so no artificial constructor-only test was kept.

## GREEN

Three source-local tests directly initialized one counter at `usize::MAX`, then
verified the exact `Nodes`, `PayloadBytes`, or `OutputBytes` error and complete
transaction rollback. Focused verification passed:

```text
cargo test --locked --all-features --lib \
  value_wire_encode_preflight::tests -- --nocapture
3 passed; 0 failed

cargo test --locked --all-features --test integration_tests \
  value_wire -- --nocapture
105 passed; 0 failed

cargo test --locked --all-features --test integration_tests \
  json_tests -- --nocapture
22 passed; 0 failed
```

The fresh full coverage run then passed the unchanged gate:

```text
functions=156/161, lines=1037/1091, regions=1520/1598
```

Relevant file results include:

| Source | Functions | Lines | Regions |
| --- | ---: | ---: | ---: |
| `value_wire_encode_preflight.rs` | 100% | 99.37% | 97.62% |
| `value_wire_encode_error.rs` | 100% | 100% | 100% |
| `wire/json.rs` | 100% | 100% | 100% |

## Self-review

- Every public test asserts a concrete result, error variant, resource, limit,
  or cumulative observation; there are no invocation-only coverage tests.
- Private tests are limited to counters that public callers cannot feasibly
  overflow and do not widen production visibility.
- The tests preserve the existing Wire V1 representation and preflight lower-
  bound semantics.
- No coverage threshold, exemption, feature, or public API was changed.
