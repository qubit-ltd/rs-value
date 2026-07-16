# rs-value Runtime Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace source-oriented conversion bounds, centralize policy-driven numeric comparison, make `Value` a lawful hash key, remove arbitrary defaults, and migrate every affected downstream crate.

**Architecture:** `rs-datatype` owns the target-side conversion trait and a value-crate-independent numeric comparison kernel. `rs-value` maps its owned enum variants into those lower-level abstractions and owns representation equality, hashing, unset state, and comparison errors. `rs-config` and `rs-metadata` retain only their domain-specific behavior and delete duplicated generic bounds and numeric algorithms.

**Tech Stack:** Rust 2024, Rust 1.94, Serde, thiserror, BigInt/BigDecimal, optional num-rational, external integration tests, rustdoc compile-contract tests, repository CI scripts.

## Global Constraints

- Breaking changes are allowed in `rs-datatype`, `rs-value`, `rs-config`, `rs-metadata`, and every affected `rs-*` crate.
- Do not add compatibility aliases or deprecated shims for removed APIs.
- `rs-datatype` must not depend on `rs-value`.
- Keep `rs-datatype`'s default feature set free of arbitrary-precision dependencies.
- Numeric ordering applies only to numeric variants and always requires an explicit policy.
- `Value` equality is same-variant representation identity; it is distinct from cross-variant numeric equivalence.
- Tests remain under each crate's external `tests/` tree and mirror source paths.
- Every struct, enum, and trait has its own snake-case source file; private helper types live under an `internal/` subtree.
- Add complete Rustdoc and the repository copyright header to every new Rust file and item.
- Do not commit unless the user explicitly authorizes commits. If authorization is later granted, commit separately in each nested Git repository.

---

### Task 1: Replace `DataConvertTo<T>` with `DataConversionTarget`

**Files:**
- Delete: `rs-datatype/src/converter/data_convert_to.rs`
- Create: `rs-datatype/src/converter/data_conversion_target.rs`
- Modify: `rs-datatype/src/converter/mod.rs`
- Modify: `rs-datatype/src/converter/data_converter.rs`
- Modify: `rs-datatype/src/converter/data_converter/boolean.rs`
- Modify: `rs-datatype/src/converter/data_converter/duration.rs`
- Modify: `rs-datatype/src/converter/data_converter/numeric.rs`
- Modify: `rs-datatype/src/converter/data_converter/structured.rs`
- Modify: `rs-datatype/src/converter/data_converter/text.rs`
- Modify: `rs-datatype/src/converter/data_converters.rs`
- Modify: `rs-datatype/src/converter/scalar_string_data_converters.rs`
- Modify: `rs-datatype/src/lib.rs`
- Delete: `rs-datatype/tests/converter/data_convert_to_tests.rs`
- Create: `rs-datatype/tests/converter/data_conversion_target_tests.rs`
- Modify: `rs-datatype/tests/converter/mod.rs`
- Test: existing files below `rs-datatype/tests/converter/`

**Interfaces:**
- Consumes: `DataConverter<'_>`, `DataConversionOptions`, `DataConversionError`, `DataTypeOf`.
- Produces:

```rust
pub trait DataConversionTarget: DataTypeOf + Sized {
    fn convert_from(
        source: &DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError>;
}
```

- [ ] **Step 1: Replace the direct-dispatch test with a failing target-side extension test**

Create `tests/converter/data_conversion_target_tests.rs` with a local newtype:

```rust
use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConversionTarget,
    DataConverter,
    DataType,
    DataTypeOf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Port(u16);

impl DataTypeOf for Port {
    const DATA_TYPE: DataType = DataType::UInt16;
}

impl DataConversionTarget for Port {
    fn convert_from(
        source: &DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError> {
        u16::convert_from(source, options).map(Self)
    }
}

#[test]
fn test_data_conversion_target_supports_downstream_newtype() {
    let port = DataConverter::from("8080")
        .to::<Port>()
        .expect("string should convert through local target implementation");
    assert_eq!(port, Port(8080));
}
```

Change `tests/converter/mod.rs` to declare `mod data_conversion_target_tests;`.

- [ ] **Step 2: Run the focused test and verify the new API is absent**

Run:

```bash
cargo test --all-features --test lib_tests converter::data_conversion_target_tests
```

Expected: compilation fails because `DataConversionTarget` does not exist.

- [ ] **Step 3: Introduce the target-side trait and exports**

Create `src/converter/data_conversion_target.rs` with the exact interface above, complete Rustdoc, and an example implementing a local newtype. Replace module declarations and crate-root exports so `qubit_datatype::DataConversionTarget` is available whenever `converter` is enabled. Remove all exports of `DataConvertTo`.

- [ ] **Step 4: Invert built-in conversion implementations**

For every existing block shaped like:

```rust
impl DataConvertTo<u16> for DataConverter<'_> {
    fn convert(&self, options: &DataConversionOptions) -> Result<u16, DataConversionError> {
        // existing body
    }
}
```

rewrite it without changing the body semantics:

```rust
impl DataConversionTarget for u16 {
    fn convert_from(
        source: &DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError> {
        // existing body, with `self` references changed to `source`
    }
}
```

Apply this to every built-in target family, including feature-gated targets. Keep implementations in the existing responsibility-specific modules.

- [ ] **Step 5: Simplify converter facade bounds**

Change `DataConverter` dispatch to:

```rust
pub fn to<T: DataConversionTarget>(&self) -> Result<T, DataConversionError> {
    self.to_with(DataConversionOptions::default_ref())
}

pub fn to_with<T: DataConversionTarget>(
    &self,
    options: &DataConversionOptions,
) -> Result<T, DataConversionError> {
    T::convert_from(self, options)
}
```

Change batch and scalar-string converter bounds from the old HRTB to `T: DataConversionTarget`. Preserve source-index wrapping and missing/empty behavior.

- [ ] **Step 6: Run converter tests**

Run:

```bash
cargo test --all-features --test lib_tests converter
```

Expected: all converter tests pass, including the local `Port` implementation.

- [ ] **Step 7: Audit removed names**

Run:

```bash
rg -n 'DataConvertTo|data_convert_to' src tests
```

Expected: no matches.

- [ ] **Step 8: Commit only if separately authorized**

In `rs-datatype`, commit only after explicit user authorization; otherwise leave the tested working-tree changes uncommitted.

---

### Task 2: Add the shared numeric comparison kernel

**Files:**
- Modify: `rs-datatype/Cargo.toml`
- Modify: `rs-datatype/src/lib.rs`
- Create: `rs-datatype/src/numeric/mod.rs`
- Create: `rs-datatype/src/numeric/numeric_comparison_policy.rs`
- Create: `rs-datatype/src/numeric/numeric_value_ref.rs`
- Create: `rs-datatype/src/numeric/compare_numeric.rs`
- Create: `rs-datatype/src/numeric/internal/mod.rs`
- Create: `rs-datatype/src/numeric/internal/fixed_numeric.rs`
- Create: `rs-datatype/src/numeric/internal/exact_rational.rs` (feature `big-number`)
- Create: `rs-datatype/tests/numeric/mod.rs`
- Create: `rs-datatype/tests/numeric/numeric_comparison_policy_tests.rs`
- Create: `rs-datatype/tests/numeric/numeric_value_ref_tests.rs`
- Create: `rs-datatype/tests/numeric/compare_numeric_tests.rs`
- Modify: `rs-datatype/tests/lib_tests.rs`

**Interfaces:**
- Consumes: fixed numeric primitives and feature-gated `BigInt`/`BigDecimal`.
- Produces:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericComparisonPolicy {
    #[default]
    Exact,
    Approximate,
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum NumericValueRef<'a> {
    Int8(i8), Int16(i16), Int32(i32), Int64(i64), Int128(i128),
    UInt8(u8), UInt16(u16), UInt32(u32), UInt64(u64), UInt128(u128),
    Float32(f32), Float64(f64),
    #[cfg(feature = "big-number")]
    BigInteger(&'a num_bigint::BigInt),
    #[cfg(feature = "big-number")]
    BigDecimal(&'a bigdecimal::BigDecimal),
}

pub fn compare_numeric(
    left: NumericValueRef<'_>,
    right: NumericValueRef<'_>,
    policy: NumericComparisonPolicy,
) -> Option<std::cmp::Ordering>;
```

- [ ] **Step 1: Write table-driven failing fixed-number tests**

Cover signed/unsigned reversal, `i128::MIN`, `u128::MAX`, `2^24`, `2^53`, fractional floats, signed zero, infinities, and distinct NaN payloads. Each case must also assert reversal symmetry:

```rust
fn assert_order(
    left: NumericValueRef<'_>,
    right: NumericValueRef<'_>,
    expected: Option<Ordering>,
) {
    assert_eq!(compare_numeric(left, right, NumericComparisonPolicy::Exact), expected);
    assert_eq!(
        compare_numeric(right, left, NumericComparisonPolicy::Exact),
        expected.map(Ordering::reverse),
    );
}
```

Include explicit assertions that every comparison containing NaN returns `None` and `-0.0` compares equal to `+0.0`.

- [ ] **Step 2: Write failing approximate-policy tests**

Assert that approximate projection is used only when a floating variant participates:

```rust
assert_eq!(
    compare_numeric(
        NumericValueRef::Float64(0.1),
        NumericValueRef::BigDecimal(&BigDecimal::from_str("0.1").unwrap()),
        NumericComparisonPolicy::Approximate,
    ),
    Some(Ordering::Equal),
);
```

Also assert that two large integer operands remain exactly ordered under `Approximate` and that an out-of-range finite projection falls back to exact ordering.

- [ ] **Step 3: Run numeric tests and verify the module is absent**

Run:

```bash
cargo test --all-features --test lib_tests numeric
```

Expected: compilation fails because the numeric module and APIs do not exist.

- [ ] **Step 4: Add policy and borrowed numeric types**

Create one public type per file, export them through `numeric/mod.rs` and `lib.rs`, and add `#[must_use]` to both public value types. Add `num-rational = { version = "0.4", optional = true }` and include it only in the `big-number` feature.

- [ ] **Step 5: Implement exact fixed-width comparison**

In `internal/fixed_numeric.rs`, normalize fixed integers into sign plus `u128` magnitude and decode finite IEEE values into sign, significand, and base-two exponent. Compare signs first, then bit lengths, and shift only after proving the shifted magnitude fits. Handle infinity before finite normalization and NaN before all ordering. Do not cast integers to floats in the exact path.

Expose only narrow crate-private helpers consumed by `compare_numeric.rs`:

```rust
pub(super) fn compare_fixed(
    left: NumericValueRef<'_>,
    right: NumericValueRef<'_>,
) -> Option<Ordering>;
```

- [ ] **Step 6: Implement exact arbitrary-precision comparison**

Under `big-number`, convert comparisons involving `BigInt` or `BigDecimal` into `BigRational`:

```rust
pub(super) fn to_exact_rational(value: NumericValueRef<'_>) -> Option<BigRational>;
```

Map integers to denominator one, decode floats from IEEE significand/exponent without decimal formatting, and derive decimals from `BigDecimal::as_bigint_and_exponent()`. Return `None` only for NaN; handle infinities before rational conversion.

- [ ] **Step 7: Implement policy dispatch**

`compare_numeric` must follow this order:

```rust
// 1. Reject NaN with None.
// 2. Order infinities exactly.
// 3. For Approximate with at least one floating variant, compare finite f64
//    projections when both projections are finite.
// 4. Otherwise use the exact fixed or arbitrary-precision path.
```

Do not add epsilon comparison or string conversion.

- [ ] **Step 8: Run numeric tests in both feature profiles**

Run:

```bash
cargo test --test lib_tests numeric
cargo test --all-features --test lib_tests numeric
```

Expected: fixed-family tests pass without features; big-number tests pass with all features.

- [ ] **Step 9: Commit only if separately authorized**

In `rs-datatype`, commit the numeric kernel separately only after explicit user authorization.

---

### Task 3: Migrate `rs-value` conversion bounds

**Files:**
- Modify: `rs-value/src/value/value.rs`
- Modify: `rs-value/src/value/value_converters.rs`
- Modify: `rs-value/src/multi_values/multi_values_converters.rs`
- Modify: `rs-value/src/value_container.rs`
- Modify: `rs-value/tests/value/value_converter_tests.rs`
- Modify: `rs-value/tests/multi_values/multi_values_converters_tests.rs`
- Modify: `rs-value/tests/value_container_tests.rs`
- Modify: `rs-value/tests/feature_contract_tests.rs`

**Interfaces:**
- Consumes: `qubit_datatype::DataConversionTarget` from Task 1.
- Produces: all public `to*` methods bounded only by `T: DataConversionTarget`.

- [ ] **Step 1: Add a failing public-bound test**

Define a local `Port` target in `feature_contract_tests.rs`, then call `Value::to`, `MultiValues::to_list`, and `ValueContainer::to` without writing a `DataConverter` HRTB. Assert the converted values.

- [ ] **Step 2: Run the feature-contract test and verify old bounds remain**

Run:

```bash
cargo test --all-features --test feature_contract_tests
```

Expected: compilation fails until `rs-value` imports and uses `DataConversionTarget`.

- [ ] **Step 3: Replace conversion bounds and delegation**

Every scalar target bound becomes:

```rust
T: DataConversionTarget
```

Keep list return types as `Vec<T>`, keep string splitting in `ValueContainer::to_list_with`, and preserve `ValueError::{DataConversion,DataListConversion}` mapping.

- [ ] **Step 4: Run focused conversion tests**

Run:

```bash
cargo test --all-features --test integration_tests value::value_converter_tests
cargo test --all-features --test integration_tests multi_values::multi_values_converters_tests
cargo test --all-features --test integration_tests value_container_tests
cargo test --all-features --test feature_contract_tests
```

Expected: all focused tests pass.

- [ ] **Step 5: Audit old bounds in rs-value**

Run:

```bash
rg -n 'DataConvertTo|for<.a> DataConverter' src tests
```

Expected: no matches.

---

### Task 4: Implement lawful `Value` equality and hashing

**Files:**
- Modify: `rs-value/src/value/value.rs`
- Modify: `rs-value/src/value/mod.rs`
- Create: `rs-value/src/value/value_identity.rs`
- Create: `rs-value/tests/value/value_identity_tests.rs`
- Modify: `rs-value/tests/value/mod.rs`

**Interfaces:**
- Consumes: the existing `Value` variants and shared type-table macro.
- Produces: `impl PartialEq`, `impl Eq`, and `impl Hash` for `Value`.

- [ ] **Step 1: Write failing float identity tests**

Create helper functions using `DefaultHasher`, then assert:

```rust
assert_eq!(Value::Float64(-0.0), Value::Float64(0.0));
assert_eq!(hash(&Value::Float64(-0.0)), hash(&Value::Float64(0.0)));

let left = Value::Float64(f64::from_bits(0x7ff8_0000_0000_0001));
let right = Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff));
assert_eq!(left, left);
assert_eq!(left, right);
assert_eq!(hash(&left), hash(&right));
assert_ne!(Value::Float32(f32::NAN), Value::Float64(f64::NAN));
```

Repeat canonical-zero and multiple-payload assertions for `f32`.

- [ ] **Step 2: Write failing map, JSON, and variant tests**

Construct string maps and nested JSON objects in different insertion orders. Assert equality and equal hashes. Assert array order changes equality. Assert every representative cross-variant numeric pair is unequal and has no equality requirement across variants.

- [ ] **Step 3: Run identity tests and verify derived equality fails the contract**

Run:

```bash
cargo test --all-features --test integration_tests value::value_identity_tests
```

Expected: failures for NaN reflexivity and signed-zero hashing support; `Hash` is not implemented.

- [ ] **Step 4: Remove derived `PartialEq` and implement identity helpers**

In `value_identity.rs`, define documented private helpers:

```rust
fn canonical_f32_bits(value: f32) -> u32;
fn canonical_f64_bits(value: f64) -> u64;
fn json_eq(left: &serde_json::Value, right: &serde_json::Value) -> bool;
fn hash_json<H: Hasher>(value: &serde_json::Value, state: &mut H);
fn hash_string_map<H: Hasher>(value: &HashMap<String, String>, state: &mut H);
```

Canonicalize all zero to positive-zero bits and all NaN payloads to one quiet-NaN constant per width. JSON arrays recurse in order; objects compare by key lookup and hash keys in sorted order. String maps hash their length and sorted key/value pairs.

- [ ] **Step 5: Implement `PartialEq + Eq + Hash` through the type table**

Hash the outer enum discriminant before payload data. Use the type table's materialization/JSON-class tokens to generate same-variant arms while routing floats, `StringMap`, and JSON through their special helpers. The fallback arm for different variants returns `false`. `Unset` compares and hashes its `DataType`.

- [ ] **Step 6: Run identity and existing core tests**

Run:

```bash
cargo test --all-features --test integration_tests value::value_identity_tests
cargo test --all-features --test integration_tests value::value_core_tests
cargo test --all-features --test integration_tests json_tests
```

Expected: all tests pass.

---

### Task 5: Expose policy-driven numeric comparison from `Value`

**Files:**
- Create: `rs-value/src/numeric_comparison_error.rs`
- Modify: `rs-value/src/lib.rs`
- Modify: `rs-value/src/value/mod.rs`
- Create: `rs-value/src/value/value_numeric_comparison.rs`
- Create: `rs-value/tests/numeric_comparison_error_tests.rs`
- Create: `rs-value/tests/value/value_numeric_comparison_tests.rs`
- Modify: `rs-value/tests/mod.rs`
- Modify: `rs-value/tests/value/mod.rs`

**Interfaces:**
- Consumes: `NumericValueRef`, `NumericComparisonPolicy`, and `compare_numeric` from Task 2.
- Produces:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum NumericComparisonError {
    LeftNotNumeric { actual: DataType },
    RightNotNumeric { actual: DataType },
    UnorderedNaN,
}

impl Value {
    pub fn numeric_cmp(
        &self,
        other: &Self,
        policy: NumericComparisonPolicy,
    ) -> Result<Ordering, NumericComparisonError>;
}
```

- [ ] **Step 1: Write failing API and error tests**

Test exact equality across integer variants, exact signed/unsigned ordering, exact decimal/float distinction, approximate decimal/float equality, non-numeric left/right errors, and NaN errors. Also assert that `Value::Int32(1) != Value::Int64(1)` while exact `numeric_cmp` returns `Ordering::Equal`.

- [ ] **Step 2: Run focused tests and verify APIs are absent**

Run:

```bash
cargo test --all-features --test integration_tests value::value_numeric_comparison_tests
cargo test --all-features --test integration_tests numeric_comparison_error_tests
```

Expected: compilation fails because the public error and method do not exist.

- [ ] **Step 3: Implement variant mapping and comparison errors**

Add a private, documented `as_numeric_ref(&self) -> Option<NumericValueRef<'_>>` that explicitly maps every numeric variant and returns `None` for `Unset` and non-numeric variants. `numeric_cmp` checks left, then right, then calls the lower kernel; map `None` from the kernel to `UnorderedNaN`.

- [ ] **Step 4: Export and document the API**

Export `NumericComparisonError` from `lib.rs`. Re-export no duplicate comparison policy; callers import `NumericComparisonPolicy` from `qubit-datatype`. Add examples that distinguish identity equality from numeric equality.

- [ ] **Step 5: Run numeric comparison tests**

Run the two focused commands from Step 2. Expected: all tests pass.

---

### Task 6: Remove arbitrary defaults and add must-use diagnostics

**Files:**
- Modify: `rs-value/src/value/value.rs`
- Modify: `rs-value/src/multi_values/multi_values.rs`
- Modify: `rs-value/src/multi_values/multi_values_core.rs`
- Modify: `rs-value/src/value_container.rs`
- Modify: `rs-value/src/named_value.rs`
- Modify: `rs-value/src/named_multi_values.rs`
- Modify: `rs-value/src/value_wire/value_wire_v1.rs`
- Modify: `rs-value/tests/public_api_boundary_tests.rs`
- Modify: affected rs-value tests that call `Value::default()` or `MultiValues::default()`

**Interfaces:**
- Produces `Value::new_unset(DataType)` and `MultiValues::new_unset(DataType)`.
- Removes `Default` from both types.
- Adds approved type-level and query-level must-use diagnostics.

- [ ] **Step 1: Add compile-contract tests**

Use the existing external-consumer compiler harness to assert that this source fails:

```rust
use qubit_value::Value;
fn main() { let _ = Value::default(); }
```

and that explicit construction compiles:

```rust
use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value};
fn main() {
    let _ = Value::new_unset(DataType::Int32);
    let _ = MultiValues::new_unset(DataType::String);
}
```

Add a `#![deny(unused_must_use)]` consumer proving that a discarded `Value` constructor or approved pure query fails compilation.

- [ ] **Step 2: Run boundary tests and verify old defaults still compile**

Run:

```bash
cargo test --all-features --test integration_tests public_api_boundary_tests
```

Expected: new negative tests fail because `Default` still exists and must-use warnings are absent.

- [ ] **Step 3: Add explicit unset constructors and remove defaults**

Add constructor-first methods:

```rust
#[must_use]
#[inline(always)]
pub const fn new_unset(data_type: DataType) -> Self {
    Self::Unset(data_type)
}
```

Remove both `impl Default` blocks. Replace rs-value's own default calls with explicit declared types.

- [ ] **Step 4: Add semantic must-use attributes**

Add type-level `#[must_use]` to the six approved public value types. Add method-level attributes to unprotected pure queries such as `data_type`, `count`, `is_unset`, `is_numeric`, `is_scalar`, `is_collection`, and `name`. Do not redundantly annotate methods returning `Result` or `Option`.

- [ ] **Step 5: Run boundary and core tests**

Run:

```bash
cargo test --all-features --test integration_tests public_api_boundary_tests
cargo test --all-features --test integration_tests value::value_core_tests
cargo test --all-features --test integration_tests multi_values::multi_values_core_tests
```

Expected: all tests pass.

---

### Task 7: Migrate rs-config to the target-side conversion API

**Files:**
- Modify: `rs-config/src/config.rs`
- Modify: `rs-config/src/config_reader.rs`
- Modify: `rs-config/src/config_section.rs`
- Modify: `rs-config/src/from/from_config.rs`
- Modify: `rs-config/src/config_value_deserializer.rs` where generic bounds require it
- Modify: affected files constructing unset values
- Test: `rs-config/tests/config_tests.rs`
- Test: `rs-config/tests/config_reader_tests.rs`
- Test: `rs-config/tests/config_section_tests.rs`
- Test: `rs-config/tests/from/from_config_tests.rs`
- Test: `rs-config/tests/config_value_deserializer_tests.rs`

**Interfaces:**
- Consumes: `DataConversionTarget`, explicit `Value::new_unset`, and unchanged value conversion behavior.
- Produces: `ConfigReader` and `Config` generic APIs without a `DataConverter` HRTB.

- [ ] **Step 1: Change a compile-time bound test to require only the target trait**

Add a generic helper in `config_reader_tests.rs` against the conversion-backed
list API (the scalar `get` API intentionally remains governed by
`FromConfig`):

```rust
fn read_converted_list<T>(
    reader: &impl ConfigReader,
    key: &str,
) -> ConfigResult<Vec<T>>
where
    T: DataConversionTarget,
{
    reader.get_list(key)
}
```

Exercise it with a supported target and preserve keyed error assertions.

- [ ] **Step 2: Run focused tests before migration**

Run:

```bash
cargo test --all-features --test config_reader_tests
```

Expected: compilation fails until public and internal bounds use the new trait.

- [ ] **Step 3: Replace imports and bounds**

Replace `DataConvertTo` imports and every HRTB with `DataConversionTarget`. Keep `FromConfig`, `StrictValueRead`, `StrictValueListRead`, and configuration-specific substitution/default behavior. Replace direct unset construction only where the explicit constructor improves consistency; preserve declared types.

- [ ] **Step 4: Run focused config tests**

Run:

```bash
cargo test --all-features --test config_reader_tests
cargo test --all-features --test config_tests
cargo test --all-features --test config_section_tests
cargo test --all-features --test from_config_tests
cargo test --all-features --test config_value_deserializer_tests
```

Expected: all tests pass with unchanged keyed error semantics.

- [ ] **Step 5: Audit removed bounds**

Run `rg -n 'DataConvertTo|for<.a> DataConverter' src tests`. Expected: no matches.

---

### Task 8: Replace rs-metadata conversion and numeric comparison duplication

**Files:**
- Delete: `rs-metadata/src/from_metadata_value.rs`
- Delete: `rs-metadata/src/filter/number_comparison_policy.rs`
- Modify: `rs-metadata/src/lib.rs`
- Modify: `rs-metadata/src/metadata.rs`
- Modify: `rs-metadata/src/filter/condition.rs`
- Modify: `rs-metadata/src/filter/filter_match_options.rs`
- Modify: `rs-metadata/src/schema/filter_validation.rs`
- Delete: `rs-metadata/tests/from_metadata_value_tests.rs`
- Delete: `rs-metadata/tests/filter/number_comparison_policy_tests.rs`
- Modify: `rs-metadata/tests/metadata_tests.rs`
- Modify: `rs-metadata/tests/filter/condition_tests.rs`
- Modify: `rs-metadata/tests/filter/filter_match_options_tests.rs`
- Modify: `rs-metadata/tests/schema/filter_validation_tests.rs`
- Modify: affected test entry modules

**Interfaces:**
- Consumes: `DataConversionTarget`, `NumericComparisonPolicy`, and `Value::numeric_cmp`.
- Produces: metadata reads and filters with no local type allowlist or numeric kernel.

- [ ] **Step 1: Update tests to the shared policy and broaden exact numeric compatibility**

Change filter options and builders to accept `qubit_datatype::NumericComparisonPolicy`. Add assertions for exact `BigDecimal`/float ordering, approximate equality, all integer widths, NaN rejection, and schema acceptance of every non-NaN numeric representation for numeric fields.

- [ ] **Step 2: Run focused metadata tests before migration**

Run:

```bash
cargo test --test metadata_tests
cargo test --test filter_tests
cargo test --test schema_tests
```

Expected: compilation failures for shared policy/API references or behavior failures under the old conservative implementation.

- [ ] **Step 3: Remove `FromMetadataValue`**

Change typed metadata APIs to:

```rust
pub fn try_get<T>(&self, key: &str) -> MetadataResult<T>
where
    T: DataConversionTarget,
```

Delegate to `value.to::<T>()` and preserve `MetadataError::conversion_error` context. Remove the trait module, export, tests, and manual target list.

- [ ] **Step 4: Replace filter comparison**

Delete `NumberValue`, safe-range constants, integer/float helpers, big-number helpers, and approximate `f64` fallback from `condition.rs`. Numeric equality and range predicates call `Value::numeric_cmp`; string range comparison stays local. Map comparison errors to predicate non-match rather than panic.

- [ ] **Step 5: Simplify schema validation**

Delete conservative compatibility helpers. A filter value is compatible when its exact data type matches, or both field and value are numeric and the value is not a NaN float. Keep existing rejection for unset values and nonnumeric mismatches.

- [ ] **Step 6: Run focused metadata tests**

Run the three commands from Step 2. Expected: all tests pass.

- [ ] **Step 7: Audit duplicate APIs and algorithms**

Run:

```bash
rg -n 'FromMetadataValue|NumberComparisonPolicy|NumberValue|MAX_SAFE_INTEGER|compare_i128_f64|compare_u128_f64|big_integer_value|big_decimal_value' src tests
```

Expected: no matches, except references in migration notes if any documentation explicitly records removed APIs.

---

### Task 9: Cross-crate audit, documentation, and verification

**Files:**
- Modify: `rs-datatype/README.md`
- Modify: `rs-datatype/README.zh_CN.md`
- Modify: `rs-value/README.md`
- Modify: `rs-value/README.zh_CN.md`
- Modify: `rs-metadata/README.md` and localized guide only where removed policy names appear
- Modify: any additional affected `rs-*` source found by the audit

**Interfaces:**
- Consumes: all APIs produced by Tasks 1–8.
- Produces: a clean workspace with no stale names and verified affected crates.

- [ ] **Step 1: Search every rs-* crate for removed APIs and implicit defaults**

Run from `rust-common`:

```bash
rg -n 'DataConvertTo|FromMetadataValue|NumberComparisonPolicy|Value::default\(\)|MultiValues::default\(\)' rs-*
```

Migrate every production, test, doctest, and README match. Do not modify generated `target/` content.

- [ ] **Step 2: Update public documentation**

Document `DataConversionTarget`, exact versus approximate numeric comparison, representation equality versus numeric equality, canonical NaN/zero hashing, order-independent map/JSON hashing, and explicit typed unset construction. Remove old trait and metadata-policy examples.

- [ ] **Step 3: Run prescribed verification in dependency order**

For each changed crate, run from that crate directory:

```bash
./align-ci.sh
./ci-check.sh
```

Order: `rs-datatype`, `rs-value`, `rs-config`, `rs-metadata`, then any additional affected crate. Stop at the first failure, fix only in-scope causes, and rerun that crate before continuing.

- [ ] **Step 4: Run conditional coverage inspection**

Only when a crate's CI reports coverage below its threshold and `coverage.sh` exists, run exactly:

```bash
./coverage.sh json
```

Add tests for uncovered approved behavior, then rerun alignment and CI in the prescribed order.

- [ ] **Step 5: Re-run the workspace stale-name audit**

Run:

```bash
rg -n 'DataConvertTo|FromMetadataValue|NumberComparisonPolicy|Value::default\(\)|MultiValues::default\(\)' rs-* --glob '!target/**'
```

Expected: no stale code or documentation references.

- [ ] **Step 6: Inspect final changes per repository**

Run `git status --short` and `git --no-pager diff --check` separately in each changed nested repository. Verify no unrelated user changes were modified and no generated artifacts are included.

- [ ] **Step 7: Commit only if separately authorized**

If the user explicitly authorizes commits, create separate, intention-focused commits in each nested repository. Otherwise report the verified uncommitted changes.
