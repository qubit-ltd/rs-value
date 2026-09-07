# Task T7B Report: Getter and Visitor Coverage

## Scope

Added behavior-focused integration tests for strict `Value` and `MultiValues`
conversion paths and for owned-string deserialization through the two numeric
visitors. No production source or coverage configuration was changed.

## TDD and focused verification

- RED: the existing coverage artifact reported `Value` getter at 4/4 functions,
  28/43 lines, 26/42 regions; `MultiValues` getter at 7/8 functions, 28/79
  lines, 34/77 regions; and both visitor files at 2/3 functions, 8/13 lines,
  9/14 regions.
- FIRST_CHECK: the getter and wide-integer suites passed. The first decimal
  visitor run failed 2 of 4 tests because it incorrectly targeted the
  structured BigDecimal V1 payload. The tests were corrected to exercise the
  BigInteger string adapter that owns `DecimalVisitor`.
- GREEN: focused suites passed with 17/17 `Value` getter tests, 14/14
  `MultiValues` getter tests, 5/5 wide-integer visitor tests, and 4/4 decimal
  visitor tests.
- Coverage sampling: the two visitors now report 3/3 functions, 13/13 lines,
  and 14/14 regions. The `Value` getter reports 4/4 functions, 35/43 lines,
  and 33/42 regions; the `MultiValues` getter reports 8/8 functions, 63/79
  lines, and 65/77 regions.

## Contracts covered

- Borrowed and consuming scalar reads across present, unset, and wrong-type
  storage.
- First-value, vector, borrowed-element, borrowed-slice, consuming-vector, and
  borrowed-string collection reads across present, empty, unset, and
  wrong-type storage.
- Owned JSON string dispatch into both numeric visitors, including wide
  integer limits, an arbitrary-precision magnitude, and exact non-canonical
  error messages.

## Files changed

- `tests/multi_values/multi_values_getter_tests.rs`
- `tests/value/value_getter_tests.rs`
- `tests/wide_integer/internal/integer_visitor_tests.rs`
- `tests/wire/decimal/internal/decimal_visitor_tests.rs`

The repository-wide CI and coverage gate remain assigned to the integration
task; this leaf task ran only focused checks as required by the execution plan.
