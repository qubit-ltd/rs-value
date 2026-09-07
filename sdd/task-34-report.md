# T3D Report: Identity and Numeric Parsing Coverage

## Scope

Added behavior-focused tests under `tests/identity/`, `tests/wide_integer/`,
and `tests/wire/decimal/` only. The tests cover canonical hashing, signed-zero
identity, map order independence, extreme 128-bit integer values, malformed and
non-canonical integer text, collection parsing, and large `BigInt` decimal wire
values.

## Verification

- `rustfmt --edition 2024 --check` passed for all T3D-modified Rust files.
- The pre-change focused wide-integer test command passed: 6 tests passed.
- The post-change integration test command was blocked by an unrelated compile
  error in `tests/value/value_contract_edge_tests.rs:87` (`no rules expected
  keyword if` in a macro invocation introduced by another task).
- Coverage could not run because this worktree's `.rs-ci` submodule is not
  initialized and does not contain `.rs-ci/coverage.sh`.

## Files changed

- `tests/identity/big_decimal_hash_tests.rs`
- `tests/identity/float_identity_tests.rs`
- `tests/identity/string_map_hash_tests.rs`
- `tests/wide_integer/internal/display_integer_tests.rs`
- `tests/wide_integer/internal/integer_visitor_tests.rs`
- `tests/wide_integer/internal/parsed_integer_tests.rs`
- `tests/wire/decimal/internal/decimal_visitor_tests.rs`
- `tests/wire/decimal/internal/display_decimal_tests.rs`
- `tests/wire/decimal/internal/parsed_decimal_tests.rs`

## Integration follow-up

- Corrected the unsigned `"-1"` assertion to require the actual invalid-digit
  parse error rather than a non-canonical error.
- Removed an unused import from the wide-integer collection test.
- Re-ran the focused and related test commands after the correction; results
  are recorded in the follow-up commit.

## Follow-up verification results

- `cargo test --locked --all-features --test integration_tests wide_integer`:
  10 passed.
- `cargo test --locked --all-features --test integration_tests identity`:
  52 passed, 2 ignored.
- `cargo test --locked --all-features --test integration_tests 'wire::decimal'`:
  8 passed.
- `cargo test --locked --all-features --test integration_tests`:
  679 passed, 2 ignored.
