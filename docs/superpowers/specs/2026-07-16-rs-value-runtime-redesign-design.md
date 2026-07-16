# rs-value Runtime Conversion, Comparison, Equality, and State Redesign

## Status

Approved for implementation on 2026-07-16.

This design intentionally permits breaking changes in `rs-datatype`,
`rs-value`, and every affected `rs-*` downstream crate. Compatibility aliases
and deprecation shims are out of scope.

## Context

`rs-value` provides owned runtime scalar, homogeneous collection, and explicit
scalar-or-collection containers. Its conversion operations delegate to
`rs-datatype::DataConverter`.

The current conversion trait is implemented on the source wrapper and is
generic over the target:

```rust
for<'a> DataConverter<'a>: DataConvertTo<T>
```

That bound leaks into downstream public APIs. `rs-config` repeats it across
reader interfaces, while `rs-metadata` avoids exposing it by maintaining a
separate `FromMetadataValue` trait and a manual list of every supported target
type.

`rs-metadata` also implements its own cross-representation numeric comparison
engine and a second compatibility approximation in schema validation. This
duplicates runtime type knowledge and risks disagreement between validation and
execution.

Finally, `Value` cannot currently be used as a hash key because floating-point
values do not implement `Eq` or `Hash`, unordered maps require order-independent
hashing, and JSON object ordering must not affect structural identity.

## Goals

1. Replace the source-oriented conversion bound with a simple, extensible
   target-oriented trait.
2. Put numeric representation and comparison rules in `rs-datatype` and expose
   an ergonomic `Value` comparison API from `rs-value`.
3. Give `Value` a lawful, documented `PartialEq + Eq + Hash` contract suitable
   for `HashSet`, `HashMap` keys, and caches.
4. Remove arbitrary `String` semantics from `Value::default()` and
   `MultiValues::default()`.
5. Add semantic `must_use` diagnostics to value types and pure queries.
6. Remove duplicated conversion and numeric-comparison code from downstream
   crates.

## Non-goals

- Do not merge `DataConverter` and `Value`. The former is a borrowed-or-owned
  conversion source; the latter is an owned persistent runtime value with a
  stable wire contract.
- Do not implement `PartialOrd` or `Ord` for `Value`; numeric ordering always
  requires an explicit policy.
- Do not define ordering for strings, dates, times, durations, URLs, maps, or
  JSON.
- Do not redesign, remove, or feature-gate `NamedValue`, `NamedMultiValues`, or
  `ValueWireV1`.
- Do not add an untyped null state.
- Do not preserve compatibility with `DataConvertTo`, the old metadata
  comparison policy, or the old `Default` implementations.

## Architecture

Responsibilities are divided as follows:

| Crate | Responsibility |
| --- | --- |
| `rs-datatype` | Runtime data-type vocabulary, conversion targets, conversion policies, numeric references, and numeric comparison kernel |
| `rs-value` | Owned scalar and collection containers, strict reads, conversion delegation, value identity, hashing, and wire representation |
| `rs-config` | Configuration-specific lookup, substitution, defaults, and keyed error context |
| `rs-metadata` | Metadata storage, filter expression semantics, and schema validation built on shared value operations |

`rs-datatype` must not depend on `rs-value`. `rs-value` maps its numeric enum
variants into the borrowed numeric representation exported by `rs-datatype`.

## Target-oriented data conversion

### Public trait

Remove `DataConvertTo<T>` and introduce:

```rust
pub trait DataConversionTarget: DataTypeOf + Sized {
    fn convert_from(
        source: &DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError>;
}
```

The trait is deliberately not sealed. A downstream crate may implement both
`DataTypeOf` and `DataConversionTarget` for its own local type. Its
`DataTypeOf::DATA_TYPE` identifies the underlying runtime data family used in
structured errors.

All built-in target types supported today receive implementations in
`rs-datatype`. Existing conversion logic remains organized by target family;
only the public dispatch direction changes.

### Source API

`DataConverter` uses the target-side bound:

```rust
impl DataConverter<'_> {
    pub fn to<T: DataConversionTarget>(&self)
        -> Result<T, DataConversionError>;

    pub fn to_with<T: DataConversionTarget>(
        &self,
        options: &DataConversionOptions,
    ) -> Result<T, DataConversionError>;
}
```

`DataConverters`, `ScalarStringDataConverters`, and other generic conversion
facades use the same target-side bound. Batch conversion continues to wrap the
first failing scalar error with its original source index.

### Value API

The following `rs-value` methods replace their HRTB bounds with
`T: DataConversionTarget`:

- `Value::{to,to_with,to_or,to_or_with}`
- `MultiValues::{to,to_with,to_or,to_or_with}`
- `MultiValues::{to_list,to_list_with,to_list_or,to_list_or_with}`
- `ValueContainer::{to,to_with,to_list,to_list_with}`

Strict APIs continue to use `StrictValueRead` and `StrictValueListRead`, because
strict reads require exact stored Rust types and are not data conversion.

### Downstream migration

`rs-config` replaces every `DataConverter<'a>: DataConvertTo<T>` bound with
`T: DataConversionTarget`. Configuration-specific `FromConfig` and
`IntoConfigDefault` remain because they own substitution and missing-key
semantics.

`rs-metadata` deletes `FromMetadataValue` and its manual implementation table.
Typed metadata reads accept `T: DataConversionTarget` and delegate to
`Value::to`.

No compatibility export for `DataConvertTo` is retained.

## Numeric comparison

### Scope

Only numeric runtime types participate:

- signed integers: `i8`, `i16`, `i32`, `i64`, `i128`;
- unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`;
- floating point: `f32`, `f64`;
- feature-gated arbitrary precision: `BigInt`, `BigDecimal`.

Non-numeric `Value` variants are rejected rather than ordered.

### Borrowed numeric representation

`rs-datatype` exposes a `NumericValueRef<'a>` enum. Fixed-width primitives are
stored by value; arbitrary-precision values are borrowed. It contains no
string parsing or general conversion behavior.

The type provides enough information to distinguish floating-point operands
from integer and decimal operands so comparison policy can be applied without
inspecting `rs-value`.

### Policy

```rust
pub enum NumericComparisonPolicy {
    Exact,
    Approximate,
}
```

`Exact` is the default and compares mathematical values without integer-to-float
rounding, decimal truncation, or signedness loss.

The exact implementation must not convert an operand through `f64`. For the
always-available fixed-width family it compares sign, integer magnitude, and
the decoded IEEE-754 significand/exponent directly. Magnitude/bit-length checks
avoid overflowing `u128` when a finite float is outside the fixed-width integer
range. Float-to-float comparison uses the represented IEEE values after NaN
handling.

When the `big-number` feature is enabled, comparisons involving `BigInt` or
`BigDecimal` use an internal exact rational form: integers have denominator
one, finite floats are decoded as a binary significand times a power of two,
and decimals are represented by their integer coefficient and decimal scale.
`num-rational` is added only to the existing `big-number` feature. The default
feature set does not gain an arbitrary-precision dependency.

`Approximate` applies an `f64` projection only when at least one operand is a
floating-point variant and both operands have finite `f64` projections. Two
non-floating operands are always compared exactly. If a finite projection is
not available, comparison falls back to the exact algorithm rather than
collapsing large values to infinity.

This policy is not an epsilon comparison. For example, an `f32` value promoted
to `f64` retains the exact value represented by that `f32`.

### Special floating-point values

- Any NaN makes numeric ordering unordered.
- Negative infinity is less than every finite value.
- Positive infinity is greater than every finite value.
- Equal infinities compare equal.
- Negative zero and positive zero compare equal.

### APIs

The lower-level kernel returns `Option<Ordering>`; `None` means NaN made the
comparison unordered:

```rust
pub fn compare_numeric(
    left: NumericValueRef<'_>,
    right: NumericValueRef<'_>,
    policy: NumericComparisonPolicy,
) -> Option<Ordering>;
```

`rs-value` maps non-numeric operands and unordered numeric operands to a
structured public error:

```rust
pub fn numeric_cmp(
    &self,
    other: &Self,
    policy: NumericComparisonPolicy,
) -> Result<Ordering, NumericComparisonError>;
```

`NumericComparisonError` distinguishes a non-numeric left operand, a
non-numeric right operand, and unordered NaN. It contains data types and operand
positions but never source values.

`Value` does not implement `PartialOrd` or `Ord`.

### Metadata migration

`rs-metadata::NumberComparisonPolicy` is removed. Filter options directly use
`qubit_datatype::NumericComparisonPolicy`.

Filter equality, membership, and range predicates call `Value::numeric_cmp` for
numeric pairs. Same-variant non-numeric equality continues to use `Value::eq`;
string range predicates continue to compare strings directly.

Schema validation accepts a non-NaN numeric filter value for every numeric
field type, because exact comparison now supports every numeric representation
pair. The duplicated safe-integer constants, normalized number enum, big-number
conversion helpers, and conservative compatibility functions are deleted.

## Value equality and hashing

### Contract

`Value` manually implements `PartialEq`, `Eq`, and `Hash`. Equality represents
runtime representation identity, not cross-variant numeric equivalence.

Different enum variants are always unequal and hash through different variant
discriminants. Consequently, `Value::Int32(1)` is not equal to
`Value::Int64(1)`, even though `numeric_cmp` may report `Ordering::Equal`.

### Ordinary values

Boolean, character, integer, string, big-number, date/time, duration, URL, and
other ordinary variants delegate equality and hashing to their contained
types. `BigDecimal`, URL, and temporal types retain their own `Eq + Hash`
contracts.

`Unset` compares and hashes its declared `DataType`.

### Floating point

Equality and hashing use canonical bits separately for `f32` and `f64`:

1. Any zero normalizes to positive-zero bits.
2. Any NaN normalizes to one fixed quiet-NaN bit pattern for its width.
3. Every other value uses `to_bits()` unchanged.

This produces the following lawful behavior:

- `-0.0 == +0.0` and both hash identically;
- every `f32` NaN equals every other `f32` NaN and hashes identically;
- every `f64` NaN equals every other `f64` NaN and hashes identically;
- `Float32` and `Float64` remain unequal because their variants differ;
- equality is reflexive, symmetric, and transitive.

Numeric ordering deliberately remains unordered for NaN even though identity
equality canonicalizes NaN for hash-key safety.

### String maps

String-map equality uses map equality. Hashing sorts entries by key, then
hashes the entry count and every key/value pair in sorted order. It is
independent of `HashMap` insertion and iteration order.

### JSON

JSON identity is recursive and structural:

- null, Boolean, number, and string values delegate to `serde_json` identity;
- arrays are order-sensitive;
- objects compare by key lookup rather than iteration order;
- object hashing sorts keys and recursively hashes values;
- nesting preserves the same rules at every depth.

The manual recursion guarantees object-order independence even if the
`serde_json` map backend or feature set changes.

### Scope of derived traits

This change guarantees `Eq + Hash` for `Value`. It does not automatically add
`Hash` to `MultiValues`, `ValueContainer`, or the named wrappers; those are not
required for the approved use case.

## Explicit unset construction

Remove `Default` from `Value` and `MultiValues`. Neither type has a neutral
default because every unset value carries a declared data type.

Add explicit constructors:

```rust
Value::new_unset(data_type: DataType) -> Value
MultiValues::new_unset(data_type: DataType) -> MultiValues
```

Retain the mutating `unset(&mut self)` methods. They remove concrete storage
while preserving the current data type. `MultiValues::clear` continues to
produce a concrete empty collection, while `MultiValues::unset` produces an
unset collection.

All downstream uses must choose an explicit declared type. No untyped null
variant or compatibility `Default` implementation is added.

## Must-use diagnostics

Add type-level `#[must_use]` to:

- `Value`;
- `MultiValues`;
- `ValueContainer`;
- `ValueWireV1`;
- `NamedValue`;
- `NamedMultiValues`.

Add method-level `#[must_use]` to pure queries whose return types do not already
carry reliable must-use semantics, including `data_type`, `count`, `is_unset`,
`is_numeric`, `is_scalar`, `is_collection`, and `name`.

Do not redundantly annotate methods returning `Result`, `Option`, iterators, or
other already-protected types unless the operation has a distinct obligation.

## Error handling

- Conversion continues to use `DataConversionError` and
  `DataListConversionError`.
- Custom conversion targets return the same structured, value-redacted
  conversion errors and identify their underlying `DataType` through
  `DataTypeOf`.
- Numeric comparison uses `NumericComparisonError`; it is not encoded as a data
  conversion failure.
- Downstream crates add their key, path, or filter context without discarding
  the structured source error.
- Hashing contains no fallible path and must not serialize values as an
  intermediate representation.

## Testing strategy

Implementation follows test-driven development. Tests remain external and
mirror source paths.

### rs-datatype

- Every built-in target implements `DataConversionTarget`.
- Generic conversion requires only `T: DataConversionTarget`.
- A downstream-style local newtype implementation works through
  `DataConverter`, batch converters, and `Value` after integration.
- Exact numeric comparison covers every signed/unsigned width pair and their
  extrema.
- Exact integer/float tests cover `2^24`, `2^53`, `i64`, `u64`, `i128`, and
  `u128` boundaries.
- BigInt and BigDecimal comparison covers values inside and outside `f64`
  range, decimal scale differences, and cross-family equality.
- Approximate comparison is exercised only for pairs with a floating operand.
- NaN, infinities, and signed zero have explicit tests.

### rs-value

- Every `Value` variant has equality and hash-consistency coverage.
- Distinct variants with numerically equal contents remain unequal.
- Positive and negative zero compare and hash equally.
- Multiple NaN payloads are reflexive and hash equally for each float width.
- String maps with different insertion orders compare and hash equally.
- JSON objects with different key orders, including nested objects, compare and
  hash equally; arrays remain order-sensitive.
- `HashSet<Value>` and `HashMap<Value, _>` exercise representative variants.
- `numeric_cmp` maps numeric variants and reports non-numeric and NaN errors.
- Compile-contract tests prove `Default` is unavailable and explicit unset
  constructors are available.
- Must-use compile-fail doctests cover the type-level and query contracts where
  a warning contract is externally observable.

### Downstream crates

- `rs-config` conversion behavior and keyed errors remain unchanged after bound
  simplification.
- `rs-metadata` filter equality, membership, ranges, schema validation, exact
  policy, approximate policy, NaN, and big-number cases use the shared kernel.
- Workspace searches verify that `DataConvertTo`, `FromMetadataValue`, and the
  removed metadata comparison helpers no longer exist.

## Implementation and verification order

1. Change `rs-datatype` conversion dispatch and migrate its own tests.
2. Add the `rs-datatype` numeric comparison kernel and tests.
3. Change `rs-value` conversion bounds.
4. Implement `Value` equality and hashing with tests.
5. Remove defaults, add explicit unset constructors, and add must-use
   diagnostics with compile-contract tests.
6. Migrate `rs-config`.
7. Migrate `rs-metadata` and delete its duplicate comparison implementation.
8. Migrate every other affected `rs-*` crate discovered by graph and textual
   searches.
9. For each changed crate, run its repository-prescribed alignment command,
   then CI-equivalent checks, then conditional coverage inspection when CI
   reports coverage below threshold.

Because the crates are separate Git repositories, changes and any later commits
must remain grouped per repository.

## Success criteria

- No public or internal bound uses `DataConvertTo<T>`.
- Downstream generic conversion APIs use `T: DataConversionTarget` without an
  HRTB over `DataConverter`.
- `rs-metadata` contains no private cross-numeric comparison engine or duplicate
  numeric compatibility model.
- `Value` satisfies the documented lawful `Eq + Hash` contract and works as a
  hash key.
- `Value` and `MultiValues` do not implement `Default`.
- Explicit typed unset construction is available and used downstream.
- Required must-use diagnostics are observable.
- All affected crate alignment, CI, tests, doctests, and conditional coverage
  checks pass in dependency order.
