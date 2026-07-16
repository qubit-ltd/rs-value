# rs-value Identity Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every rs-value runtime container a lawful in-memory hash key, bound BigDecimal hashing by coefficient size, improve numeric-comparison diagnostics, simplify conversion bounds, and verify every direct downstream crate.

**Architecture:** A private `identity` module owns canonical payload equality and hashing, while `Value` and `MultiValues` retain their own variant dispatch. BigDecimal hashing lives in an isolated feature-gated file so it can be moved to another crate later without importing rs-value types. Wrapper identity derives from the two core containers; numeric comparison and conversion-bound cleanup remain separate reviewable tasks in the same release plan.

**Tech Stack:** Rust 2024, Rust 1.94, `std::hash`, BigDecimal 0.4, BigInt 0.4, Serde JSON, thiserror, external integration tests, rustdoc, repository CI scripts.

## Global Constraints

- Breaking changes are allowed in `rs-value` and every affected `rs-*` downstream crate; do not add compatibility aliases or deprecated error variants.
- Different enum variants remain unequal; scalar and collection shape remains part of `ValueContainer` identity.
- Normalize signed zero and every NaN payload within its float width for both equality and hashing.
- String maps and JSON objects use structural, key-order-independent hashing; collection element order remains significant.
- BigDecimal hashing must be `O(d)` in stored coefficient digits and independent of absolute scale; it must not call `BigDecimal::hash`, expand the scale, or serialize the value.
- Keep `src/identity/big_decimal_hash.rs` independent of `Value`, `MultiValues`, wire types, and conversion types.
- Standard `Hash` output is process-local implementation behavior, not a stable persistent or distributed fingerprint.
- Tests stay in the external `tests/` tree and mirror the corresponding source responsibility.
- Every new Rust file and function receives the repository copyright header and complete English Rustdoc.
- Use Rust 1.94 commands for build and test verification; use the repository-configured nightly only through `ci-check.sh` for linting.
- Group implementation commits by intent and use the English Angular-style commit messages specified in each task.
- Do not push. If execution uses a worktree, merge it back into the original repository's current branch and remove it only after all checks and commits succeed.

---

### Task 1: Add bounded canonical BigDecimal hashing

**Files:**
- Create: `src/identity/mod.rs`
- Create: `src/identity/big_decimal_hash.rs`
- Modify: `src/lib.rs:90-110`
- Modify: `src/value/value_identity.rs:10-202`
- Modify: `tests/value/value_identity_tests.rs:32-157`

**Interfaces:**
- Consumes: `BigDecimal::as_bigint_and_exponent`, `num_bigint::Sign`, the existing value-type table, and standard `Hash`/`Hasher`.
- Produces:

```rust
#[cfg(feature = "big-number")]
pub(crate) fn hash_big_decimal<H: Hasher>(
    value: &BigDecimal,
    state: &mut H,
);

pub(crate) fn canonical_f32_bits(value: f32) -> u32;
pub(crate) fn canonical_f64_bits(value: f64) -> u64;
pub(crate) fn hash_string_map<H: Hasher>(
    value: &HashMap<String, String>,
    state: &mut H,
);
#[cfg(feature = "json")]
pub(crate) fn json_eq(left: &serde_json::Value, right: &serde_json::Value) -> bool;
#[cfg(feature = "json")]
pub(crate) fn hash_json<H: Hasher>(
    value: &serde_json::Value,
    state: &mut H,
);
```

- [ ] **Step 1: Add failing BigDecimal identity regressions**

Append these tests to `tests/value/value_identity_tests.rs`:

```rust
/// Verifies equal decimal encodings use the same canonical hash.
#[test]
fn test_value_big_decimal_identity_normalizes_coefficient_and_scale() {
    let encodings = [
        BigDecimal::new(BigInt::from(1), 0),
        BigDecimal::new(BigInt::from(10), 1),
        BigDecimal::new(BigInt::from(10_000), 4),
    ];

    for value in &encodings {
        assert_eq!(value, &encodings[0]);
        assert_eq!(
            hash(&Value::BigDecimal(value.clone())),
            hash(&Value::BigDecimal(encodings[0].clone())),
        );
    }

    let values: HashSet<_> = encodings
        .into_iter()
        .map(Value::BigDecimal)
        .collect();
    assert_eq!(values.len(), 1);
}

/// Verifies zero and extreme scales never trigger scale-sized hashing work.
#[test]
fn test_value_big_decimal_hash_handles_extreme_scales() {
    let zero_min = Value::BigDecimal(BigDecimal::new(
        BigInt::from(0),
        i64::MIN,
    ));
    let zero_max = Value::BigDecimal(BigDecimal::new(
        BigInt::from(0),
        i64::MAX,
    ));
    assert_eq!(zero_min, zero_max);
    assert_eq!(hash(&zero_min), hash(&zero_max));

    let positive = Value::BigDecimal(BigDecimal::new(
        BigInt::from(1),
        i64::MIN,
    ));
    let negative = Value::BigDecimal(BigDecimal::new(
        BigInt::from(-1),
        i64::MIN,
    ));
    assert_eq!(positive, positive);
    assert_eq!(negative, negative);
    let _ = hash(&positive);
    let _ = hash(&negative);
}
```

- [ ] **Step 2: Run the regression and confirm the upstream hash path fails**

Run:

```bash
cargo +1.94.0 test --all-features --test integration_tests \
  value::value_identity_tests::test_value_big_decimal_hash_handles_extreme_scales
```

Expected: FAIL on the current implementation while hashing the non-zero `i64::MIN` scale, demonstrating that `BigDecimal::hash` still reaches scale-dependent expansion or signed-scale overflow.

- [ ] **Step 3: Create the standalone BigDecimal hash helper**

Create `src/identity/big_decimal_hash.rs` with this implementation, plus the standard repository header:

```rust
//! Canonical hashing for [`bigdecimal::BigDecimal`].

use std::hash::{Hash, Hasher};

use bigdecimal::BigDecimal;
use num_bigint::Sign;

/// Hashes a decimal by its normalized coefficient and effective scale.
///
/// The work and temporary allocation are proportional to the stored
/// coefficient digit count and do not depend on the absolute scale.
///
/// # Arguments
///
/// * `value` - Decimal whose representation is normalized for hashing.
/// * `state` - Destination hasher.
pub(crate) fn hash_big_decimal<H: Hasher>(
    value: &BigDecimal,
    state: &mut H,
) {
    let (coefficient, scale) = value.as_bigint_and_exponent();
    if coefficient.sign() == Sign::NoSign {
        0_u8.hash(state);
        return;
    }

    let coefficient = coefficient.to_str_radix(10);
    let normalized = coefficient.trim_end_matches('0');
    let trailing_zero_count = coefficient.len() - normalized.len();
    let effective_scale =
        i128::from(scale) - trailing_zero_count as i128;

    1_u8.hash(state);
    normalized.hash(state);
    effective_scale.hash(state);
}
```

- [ ] **Step 4: Move shared payload helpers into the private identity module**

Create `src/identity/mod.rs` with the standard header and these definitions. Preserve the current JSON tag values and recursive object-key sorting exactly:

```rust
//! Shared equality and hashing for runtime value payloads.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[cfg(feature = "big-number")]
mod big_decimal_hash;
#[cfg(feature = "big-number")]
pub(crate) use big_decimal_hash::hash_big_decimal;

/// Returns canonical identity bits for an `f32` payload.
#[inline(always)]
pub(crate) fn canonical_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Returns canonical identity bits for an `f64` payload.
#[inline(always)]
pub(crate) fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Compares JSON values structurally; object member order is insignificant.
#[cfg(feature = "json")]
#[inline(always)]
pub(crate) fn json_eq(
    left: &serde_json::Value,
    right: &serde_json::Value,
) -> bool {
    left == right
}

/// Hashes JSON structurally with sorted object keys and ordered arrays.
#[cfg(feature = "json")]
pub(crate) fn hash_json<H: Hasher>(
    value: &serde_json::Value,
    state: &mut H,
) {
    match value {
        serde_json::Value::Null => 0_u8.hash(state),
        serde_json::Value::Bool(value) => {
            1_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Number(value) => {
            2_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::String(value) => {
            3_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Array(values) => {
            4_u8.hash(state);
            values.len().hash(state);
            for value in values {
                hash_json(value, state);
            }
        }
        serde_json::Value::Object(values) => {
            5_u8.hash(state);
            values.len().hash(state);
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            for key in keys {
                key.hash(state);
                hash_json(&values[key], state);
            }
        }
    }
}

/// Hashes a string map independently of its iteration order.
pub(crate) fn hash_string_map<H: Hasher>(
    value: &HashMap<String, String>,
    state: &mut H,
) {
    value.len().hash(state);
    let mut entries: Vec<_> = value.iter().collect();
    entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    for (key, value) in entries {
        key.hash(state);
        value.hash(state);
    }
}
```

Add `mod identity;` in `src/lib.rs` immediately after `mod finite_float;`, before `value_type_table` and the container modules.

- [ ] **Step 5: Refactor Value identity to dispatch through shared helpers**

Delete the helper function bodies from `src/value/value_identity.rs`, import the shared functions explicitly, and replace its two payload macros with:

```rust
use crate::identity::{
    canonical_f32_bits,
    canonical_f64_bits,
    hash_string_map,
};
#[cfg(feature = "big-number")]
use crate::identity::hash_big_decimal;
#[cfg(feature = "json")]
use crate::identity::{hash_json, json_eq};

macro_rules! payload_eq {
    (Float32, $left:expr, $right:expr) => {
        canonical_f32_bits(*$left) == canonical_f32_bits(*$right)
    };
    (Float64, $left:expr, $right:expr) => {
        canonical_f64_bits(*$left) == canonical_f64_bits(*$right)
    };
    (Json, $left:expr, $right:expr) => {
        json_eq($left, $right)
    };
    ($variant:ident, $left:expr, $right:expr) => {
        $left == $right
    };
}

macro_rules! hash_payload {
    (Float32, $value:expr, $state:expr) => {
        canonical_f32_bits(*$value).hash($state)
    };
    (Float64, $value:expr, $state:expr) => {
        canonical_f64_bits(*$value).hash($state)
    };
    (BigDecimal, $value:expr, $state:expr) => {
        hash_big_decimal($value, $state)
    };
    (StringMap, $value:expr, $state:expr) => {
        hash_string_map($value, $state)
    };
    (Json, $value:expr, $state:expr) => {
        hash_json($value, $state)
    };
    ($variant:ident, $value:expr, $state:expr) => {
        $value.hash($state)
    };
}
```

Keep the existing generated `PartialEq`, `Eq`, and `Hash` implementations unchanged except that `BigDecimal` now selects the new macro arm.

- [ ] **Step 6: Format and run focused identity tests**

Run:

```bash
cargo +1.94.0 fmt --check
cargo +1.94.0 test --all-features --test integration_tests \
  value::value_identity_tests
cargo +1.94.0 test --no-default-features --features big-number \
  --test feature_contract_tests big_number_feature_preserves_values_and_wire_payloads
```

Expected: formatting is clean; every command passes; the `i64::MIN` regression completes without panic or scale-sized allocation.

- [ ] **Step 7: Commit the bounded hash implementation**

```bash
git add src/identity src/lib.rs src/value/value_identity.rs \
  tests/value/value_identity_tests.rs
git commit -m "fix(identity): canonicalize big decimal hashing"
```

---

### Task 2: Give MultiValues lawful equality and hashing

**Files:**
- Create: `src/multi_values/multi_values_identity.rs`
- Modify: `src/multi_values/mod.rs:12-22`
- Modify: `src/multi_values/multi_values.rs:31-73`
- Create: `tests/multi_values/multi_values_identity_tests.rs`
- Modify: `tests/multi_values/mod.rs:12-22`

**Interfaces:**
- Consumes: every helper exported crate-privately by `src/identity/mod.rs` and `for_each_value_type!`.
- Produces: manual `PartialEq`, marker `Eq`, and manual `Hash` for `MultiValues`, with ordered outer collections and canonical element identity.

- [ ] **Step 1: Add failing special-value and collection-key tests**

Create `tests/multi_values/multi_values_identity_tests.rs` with the repository header, the same all-feature imports used by `value_identity_tests.rs`, and these helpers and tests:

```rust
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use url::Url;

/// Returns the standard-library hash for equality-contract assertions.
fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Requires lawful equality and equal hashes for a representative pair.
fn assert_equal_hash<T>(left: &T, right: &T)
where
    T: Debug + Eq + Hash,
{
    assert_eq!(left, right);
    assert_eq!(hash(left), hash(right));
}

#[test]
fn test_multi_values_float_identity_is_reflexive_and_hash_consistent() {
    assert_equal_hash(
        &MultiValues::Float32(vec![-0.0, f32::from_bits(0x7fc0_0001)]),
        &MultiValues::Float32(vec![0.0, f32::from_bits(0x7fff_ffff)]),
    );
    assert_equal_hash(
        &MultiValues::Float64(vec![-0.0, f64::from_bits(0x7ff8_0000_0000_0001)]),
        &MultiValues::Float64(vec![0.0, f64::from_bits(0x7fff_ffff_ffff_ffff)]),
    );

    let mut cache = HashMap::new();
    cache.insert(MultiValues::Float64(vec![f64::NAN]), "first");
    cache.insert(
        MultiValues::Float64(vec![f64::from_bits(0x7fff_ffff_ffff_ffff)]),
        "replacement",
    );
    assert_eq!(cache.len(), 1);
    assert_eq!(cache[&MultiValues::Float64(vec![f64::NAN])], "replacement");
}

#[test]
fn test_multi_values_unset_variant_and_order_remain_part_of_identity() {
    assert_ne!(
        MultiValues::Unset(DataType::Int32),
        MultiValues::Int32(Vec::new()),
    );
    assert_ne!(
        MultiValues::Unset(DataType::Int32),
        MultiValues::Unset(DataType::Int64),
    );
    assert_ne!(
        MultiValues::Int32(vec![1, 2]),
        MultiValues::Int32(vec![2, 1]),
    );
    assert_ne!(
        MultiValues::Int32(vec![1]),
        MultiValues::Int64(vec![1]),
    );
}

#[test]
fn test_multi_values_unordered_payloads_hash_structurally() {
    let left_map = HashMap::from([
        ("b".to_owned(), "2".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ]);
    let right_map = HashMap::from([
        ("a".to_owned(), "1".to_owned()),
        ("b".to_owned(), "2".to_owned()),
    ]);
    assert_equal_hash(
        &MultiValues::StringMap(vec![left_map]),
        &MultiValues::StringMap(vec![right_map]),
    );
    assert_equal_hash(
        &MultiValues::Json(vec![serde_json::json!({"b": {"y": 2, "x": 1}, "a": 0})]),
        &MultiValues::Json(vec![serde_json::json!({"a": 0, "b": {"x": 1, "y": 2}})]),
    );
}

#[test]
fn test_multi_values_big_decimal_identity_is_canonical() {
    assert_equal_hash(
        &MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(10), 1)]),
        &MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(1), 0)]),
    );
    let extreme = MultiValues::BigDecimal(vec![BigDecimal::new(
        BigInt::from(1),
        i64::MIN,
    )]);
    let _ = hash(&extreme);
}

#[test]
fn test_multi_values_identity_covers_every_variant() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 17).unwrap();
    let time = NaiveTime::from_hms_nano_opt(12, 34, 56, 789).unwrap();
    let datetime = date.and_time(time);
    let values = vec![
        MultiValues::Unset(DataType::Bool),
        MultiValues::Bool(vec![true]),
        MultiValues::Char(vec!['x']),
        MultiValues::Int8(vec![-1]),
        MultiValues::Int16(vec![-2]),
        MultiValues::Int32(vec![-3]),
        MultiValues::Int64(vec![-4]),
        MultiValues::Int128(vec![-5]),
        MultiValues::UInt8(vec![1]),
        MultiValues::UInt16(vec![2]),
        MultiValues::UInt32(vec![3]),
        MultiValues::UInt64(vec![4]),
        MultiValues::UInt128(vec![5]),
        MultiValues::Float32(vec![f32::NAN]),
        MultiValues::Float64(vec![f64::NAN]),
        MultiValues::BigInteger(vec![BigInt::from(6)]),
        MultiValues::BigDecimal(vec![BigDecimal::from(7)]),
        MultiValues::String(vec!["text".to_owned()]),
        MultiValues::Date(vec![date]),
        MultiValues::Time(vec![time]),
        MultiValues::DateTime(vec![datetime]),
        MultiValues::Instant(vec![DateTime::<Utc>::from_naive_utc_and_offset(
            datetime, Utc,
        )]),
        MultiValues::Duration(vec![Duration::new(8, 9)]),
        MultiValues::Url(vec![Url::parse("https://example.com/path").unwrap()]),
        MultiValues::StringMap(vec![HashMap::from([(
            "key".to_owned(),
            "value".to_owned(),
        )])]),
        MultiValues::Json(vec![serde_json::json!({"items": [null, true, 42]})]),
    ];

    for value in &values {
        assert_eq!(value, value);
        let _ = hash(value);
    }
    let keys: HashSet<_> = values.into_iter().collect();
    assert_eq!(keys.len(), 26);
}
```

Register `mod multi_values_identity_tests;` in `tests/multi_values/mod.rs`.

- [ ] **Step 2: Run the focused test and verify Eq/Hash are absent**

Run:

```bash
cargo +1.94.0 test --all-features --test integration_tests \
  multi_values::multi_values_identity_tests
```

Expected: compilation fails because `MultiValues` does not implement `Eq` or `Hash`; the current derived float equality would also be non-reflexive.

- [ ] **Step 3: Remove the derived PartialEq and register the identity module**

In `src/multi_values/multi_values.rs`, change the generated derive to:

```rust
#[derive(Debug, Clone)]
```

In `src/multi_values/mod.rs`, add:

```rust
mod multi_values_identity;
```

immediately after `mod multi_values_getters;`.

- [ ] **Step 4: Implement ordered collection identity with canonical elements**

Create `src/multi_values/multi_values_identity.rs` with the repository header and this dispatch. Each special arm hashes the outer vector length once and then hashes elements in order:

```rust
//! Equality and hashing for [`super::MultiValues`].

use std::hash::{Hash, Hasher};

use crate::identity::{
    canonical_f32_bits,
    canonical_f64_bits,
    hash_string_map,
};
#[cfg(feature = "big-number")]
use crate::identity::hash_big_decimal;
#[cfg(feature = "json")]
use crate::identity::{hash_json, json_eq};

use super::MultiValues;

macro_rules! payloads_eq {
    (Float32, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left.iter().zip($right).all(|(left, right)| {
                canonical_f32_bits(*left) == canonical_f32_bits(*right)
            })
    };
    (Float64, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left.iter().zip($right).all(|(left, right)| {
                canonical_f64_bits(*left) == canonical_f64_bits(*right)
            })
    };
    (Json, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left.iter().zip($right).all(|(left, right)| {
                json_eq(left, right)
            })
    };
    ($variant:ident, $left:expr, $right:expr) => {
        $left == $right
    };
}

macro_rules! hash_payloads {
    (Float32, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            canonical_f32_bits(*value).hash($state);
        }
    }};
    (Float64, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            canonical_f64_bits(*value).hash($state);
        }
    }};
    (BigDecimal, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_big_decimal(value, $state);
        }
    }};
    (StringMap, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_string_map(value, $state);
        }
    }};
    (Json, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_json(value, $state);
        }
    }};
    ($variant:ident, $values:expr, $state:expr) => {
        $values.hash($state)
    };
}

macro_rules! impl_multi_values_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?
    ) => {
        impl PartialEq for MultiValues {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::Unset(left), Self::Unset(right)) => left == right,
                    $($(#[$cfg])*
                    (Self::$variant(left), Self::$variant(right)) => {
                        payloads_eq!($variant, left, right)
                    },)+
                    _ => false,
                }
            }
        }

        impl Eq for MultiValues {}

        impl Hash for MultiValues {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::mem::discriminant(self).hash(state);
                match self {
                    Self::Unset(data_type) => data_type.hash(state),
                    $($(#[$cfg])*
                    Self::$variant(values) => {
                        hash_payloads!($variant, values, state)
                    },)+
                }
            }
        }
    };
}

for_each_value_type!(impl_multi_values_identity);
```

- [ ] **Step 5: Format and run MultiValues identity tests**

Run:

```bash
cargo +1.94.0 fmt --check
cargo +1.94.0 test --all-features --test integration_tests \
  multi_values::multi_values_identity_tests
cargo +1.94.0 test --no-default-features --test core_feature_tests
```

Expected: all commands pass; NaN-containing collections are reflexive and usable in `HashSet`.

- [ ] **Step 6: Commit collection identity**

```bash
git add src/multi_values tests/multi_values
git commit -m "feat(identity): add multi-value equality and hashing"
```

---

### Task 3: Propagate Eq and Hash through wrapper types

**Files:**
- Modify: `src/value_container.rs:28-41`
- Modify: `src/named_value.rs:27-62`
- Modify: `src/named_multi_values.rs:26-67`
- Modify: `src/value_wire/value_wire_v1.rs:26-36`
- Modify: `tests/value_container_tests.rs`
- Modify: `tests/named_value_tests.rs`
- Modify: `tests/named_multi_values_tests.rs`
- Modify: `tests/value_wire_tests.rs`

**Interfaces:**
- Consumes: lawful `Eq + Hash` from `Value` and `MultiValues`.
- Produces: `Eq + Hash` for `ValueContainer`, `NamedValue`, `NamedMultiValues`, and `ValueWireV1`; shape and names remain identity components.

- [ ] **Step 1: Add failing compile-contract and semantic tests**

Add a private compile helper and this test to `tests/value_container_tests.rs`, importing `std::collections::HashSet`, `std::hash::Hash`, and all four wrapper types:

```rust
/// Requires a type to satisfy the complete hash-key contract.
fn assert_hash_key<T: Eq + Hash>() {}

#[test]
fn test_runtime_value_wrappers_implement_hash_key_contract() {
    assert_hash_key::<ValueContainer>();
    assert_hash_key::<NamedValue>();
    assert_hash_key::<NamedMultiValues>();
    assert_hash_key::<ValueWireV1>();

    assert_ne!(
        ValueContainer::Scalar(Value::Int32(1)),
        ValueContainer::Collection(MultiValues::Int32(vec![1])),
    );

    let keys = HashSet::from([
        ValueContainer::Collection(MultiValues::Float64(vec![f64::NAN])),
        ValueContainer::Collection(MultiValues::Float64(vec![
            f64::from_bits(0x7fff_ffff_ffff_ffff),
        ])),
        ValueContainer::Scalar(Value::Float64(f64::NAN)),
    ]);
    assert_eq!(keys.len(), 2);
}
```

Add one focused identity assertion to each matching test file:

```rust
// tests/named_value_tests.rs
#[test]
fn test_named_value_identity_includes_name() {
    assert_ne!(
        NamedValue::new("left", Value::Float64(f64::NAN)),
        NamedValue::new("right", Value::Float64(f64::NAN)),
    );
    assert_eq!(
        NamedValue::new("same", Value::Float64(f64::NAN)),
        NamedValue::new("same", Value::Float64(f64::NAN)),
    );
}

// tests/named_multi_values_tests.rs
#[test]
fn test_named_multi_values_identity_is_reflexive_with_nan() {
    let values = NamedMultiValues::new(
        "samples",
        MultiValues::Float32(vec![f32::NAN]),
    );
    assert_eq!(values, values);
}

// tests/value_wire_tests.rs
#[test]
fn test_value_wire_v1_identity_preserves_shape() {
    assert_ne!(
        ValueWireV1::from(Value::Int32(1)),
        ValueWireV1::from(MultiValues::Int32(vec![1])),
    );
    let value = ValueWireV1::from(MultiValues::Float64(vec![f64::NAN]));
    assert_eq!(value, value);
}
```

- [ ] **Step 2: Run the wrapper tests and verify the trait bounds fail**

Run:

```bash
cargo +1.94.0 test --all-features --test integration_tests \
  test_runtime_value_wrappers_implement_hash_key_contract
```

Expected: compilation fails because the wrappers currently derive only `PartialEq`.

- [ ] **Step 3: Derive Eq and Hash on each wrapper**

Import `std::hash::Hash` only where the derive macro requires no explicit import; no new import is necessary for derive paths. Change the four derives exactly as follows:

```rust
// ValueContainer and ValueWireV1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

// NamedValue and NamedMultiValues
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
```

Do not implement wrapper equality manually and do not change Serde behavior.

- [ ] **Step 4: Run every affected wrapper test**

Run:

```bash
cargo +1.94.0 fmt --check
cargo +1.94.0 test --all-features --test integration_tests \
  test_runtime_value_wrappers_implement_hash_key_contract
cargo +1.94.0 test --all-features --test integration_tests named_value_tests
cargo +1.94.0 test --all-features --test integration_tests named_multi_values_tests
cargo +1.94.0 test --all-features --test integration_tests value_wire_tests
```

Expected: all commands pass; names and scalar/collection shape remain significant.

- [ ] **Step 5: Commit wrapper trait propagation**

```bash
git add src/value_container.rs src/named_value.rs \
  src/named_multi_values.rs src/value_wire/value_wire_v1.rs \
  tests/value_container_tests.rs tests/named_value_tests.rs \
  tests/named_multi_values_tests.rs tests/value_wire_tests.rs
git commit -m "feat(identity): propagate hashable value containers"
```

---

### Task 4: Make numeric comparison errors operand-specific

**Files:**
- Modify: `src/numeric_comparison_error.rs:12-32`
- Modify: `src/value/value_numeric_comparison.rs:21-88`
- Modify: `tests/numeric_comparison_error_tests.rs:12-27`
- Modify: `tests/value/value_numeric_comparison_tests.rs:58-80`
- Verify: `../rs-metadata/src/filter/condition.rs:193-220`
- Verify: `../rs-metadata/src/schema/filter_validation.rs:295-310`

**Interfaces:**
- Consumes: `Value::Unset`, `Value::data_type`, the existing `NumericValueRef` projection, and `compare_numeric`.
- Produces:

```rust
#[non_exhaustive]
pub enum NumericComparisonError {
    LeftMissing { declared: DataType },
    RightMissing { declared: DataType },
    LeftNotNumeric { actual: DataType },
    RightNotNumeric { actual: DataType },
    LeftNaN,
    RightNaN,
    BothNaN,
}
```

- [ ] **Step 1: Replace error-display tests with all seven variants**

Replace the body of `test_numeric_comparison_error_display_and_equality` with:

```rust
let cases = [
    (
        NumericComparisonError::LeftMissing {
            declared: DataType::Int32,
        },
        "left value is missing: declared type is int32",
    ),
    (
        NumericComparisonError::RightMissing {
            declared: DataType::Float64,
        },
        "right value is missing: declared type is float64",
    ),
    (
        NumericComparisonError::LeftNotNumeric {
            actual: DataType::String,
        },
        "left value is not numeric: string",
    ),
    (
        NumericComparisonError::RightNotNumeric {
            actual: DataType::Bool,
        },
        "right value is not numeric: bool",
    ),
    (NumericComparisonError::LeftNaN, "left value is NaN"),
    (NumericComparisonError::RightNaN, "right value is NaN"),
    (NumericComparisonError::BothNaN, "both values are NaN"),
];

for (error, expected) in cases {
    assert_eq!(error.to_string(), expected);
}
```

- [ ] **Step 2: Add deterministic classification tests**

Replace `test_value_numeric_cmp_reports_operand_errors` with two tests:

```rust
#[test]
fn test_value_numeric_cmp_distinguishes_missing_and_non_numeric_operands() {
    assert_eq!(
        Value::Unset(DataType::Int32)
            .numeric_cmp(&Value::Int32(1), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftMissing {
            declared: DataType::Int32,
        }),
    );
    assert_eq!(
        Value::Int32(1).numeric_cmp(
            &Value::Unset(DataType::Float64),
            NumericComparisonPolicy::Exact,
        ),
        Err(NumericComparisonError::RightMissing {
            declared: DataType::Float64,
        }),
    );
    assert_eq!(
        Value::Unset(DataType::Int32).numeric_cmp(
            &Value::Unset(DataType::Int64),
            NumericComparisonPolicy::Exact,
        ),
        Err(NumericComparisonError::LeftMissing {
            declared: DataType::Int32,
        }),
    );
    assert_eq!(
        Value::String("x".to_owned())
            .numeric_cmp(&Value::Int32(1), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftNotNumeric {
            actual: DataType::String,
        }),
    );
    assert_eq!(
        Value::Int32(1)
            .numeric_cmp(&Value::Bool(true), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNotNumeric {
            actual: DataType::Bool,
        }),
    );
}

#[test]
fn test_value_numeric_cmp_reports_nan_position_after_type_validation() {
    let nan = Value::Float64(f64::NAN);
    let number = Value::Float64(0.0);
    assert_eq!(
        nan.numeric_cmp(&number, NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftNaN),
    );
    assert_eq!(
        number.numeric_cmp(&nan, NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNaN),
    );
    assert_eq!(
        nan.numeric_cmp(&Value::Float32(f32::NAN), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::BothNaN),
    );
    assert_eq!(
        nan.numeric_cmp(&Value::Bool(true), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNotNumeric {
            actual: DataType::Bool,
        }),
    );
}
```

- [ ] **Step 3: Run the focused tests and verify old variants are insufficient**

Run:

```bash
cargo +1.94.0 test --all-features --test integration_tests \
  numeric_comparison_error_tests
cargo +1.94.0 test --all-features --test integration_tests \
  value::value_numeric_comparison_tests
```

Expected: compilation fails because the new missing and positioned-NaN variants do not exist.

- [ ] **Step 4: Replace NumericComparisonError without compatibility variants**

Replace the enum body in `src/numeric_comparison_error.rs` with:

```rust
pub enum NumericComparisonError {
    /// The left operand is unset but retains a declared type.
    #[error("left value is missing: declared type is {declared}")]
    LeftMissing {
        /// Declared runtime type of the unset left operand.
        declared: DataType,
    },
    /// The right operand is unset but retains a declared type.
    #[error("right value is missing: declared type is {declared}")]
    RightMissing {
        /// Declared runtime type of the unset right operand.
        declared: DataType,
    },
    /// The concrete left operand is not numeric.
    #[error("left value is not numeric: {actual}")]
    LeftNotNumeric {
        /// Actual runtime type of the left operand.
        actual: DataType,
    },
    /// The concrete right operand is not numeric.
    #[error("right value is not numeric: {actual}")]
    RightNotNumeric {
        /// Actual runtime type of the right operand.
        actual: DataType,
    },
    /// Only the left operand is NaN.
    #[error("left value is NaN")]
    LeftNaN,
    /// Only the right operand is NaN.
    #[error("right value is NaN")]
    RightNaN,
    /// Both operands are NaN.
    #[error("both values are NaN")]
    BothNaN,
}
```

Keep `#[must_use]`, `#[non_exhaustive]`, and the existing derives.

- [ ] **Step 5: Implement the documented validation order**

Replace `numeric_cmp` and add the private helper below `as_numeric_ref`:

```rust
pub fn numeric_cmp(
    &self,
    other: &Self,
    policy: NumericComparisonPolicy,
) -> Result<Ordering, NumericComparisonError> {
    if let Self::Unset(declared) = self {
        return Err(NumericComparisonError::LeftMissing {
            declared: *declared,
        });
    }
    if let Self::Unset(declared) = other {
        return Err(NumericComparisonError::RightMissing {
            declared: *declared,
        });
    }

    let left = self.as_numeric_ref().ok_or_else(|| {
        NumericComparisonError::LeftNotNumeric {
            actual: self.data_type(),
        }
    })?;
    let right = other.as_numeric_ref().ok_or_else(|| {
        NumericComparisonError::RightNotNumeric {
            actual: other.data_type(),
        }
    })?;

    match (self.is_nan_numeric(), other.is_nan_numeric()) {
        (true, true) => return Err(NumericComparisonError::BothNaN),
        (true, false) => return Err(NumericComparisonError::LeftNaN),
        (false, true) => return Err(NumericComparisonError::RightNaN),
        (false, false) => {},
    }

    Ok(compare_numeric(left, right, policy).expect(
        "concrete non-NaN NumericValueRef values must be ordered",
    ))
}

/// Reports whether this concrete numeric value is NaN.
///
/// # Returns
///
/// `true` only for primitive floating-point NaN variants.
#[inline(always)]
fn is_nan_numeric(&self) -> bool {
    match self {
        Self::Float32(value) => value.is_nan(),
        Self::Float64(value) => value.is_nan(),
        _ => false,
    }
}
```

Update the `# Errors` Rustdoc to name all seven variants and state that missing operands are checked left-to-right, then concrete types, then NaN positions.

- [ ] **Step 6: Run rs-value and downstream numeric behavior tests**

Run:

```bash
cargo +1.94.0 fmt --check
cargo +1.94.0 test --all-features --test integration_tests \
  numeric_comparison_error_tests
cargo +1.94.0 test --all-features --test integration_tests \
  value::value_numeric_comparison_tests
cargo +1.94.0 test --all-features
```

Then from `../rs-metadata` run:

```bash
cargo +1.94.0 test --all-features
```

Expected: all tests pass; metadata continues to discard comparison errors intentionally and preserves existing filter results.

- [ ] **Step 7: Audit removed public variants and commit**

Run from `rs-value`:

```bash
rg -n 'UnorderedNaN|NumericComparisonError::(LeftNotNumeric|RightNotNumeric)' \
  src tests ../rs-config ../rs-metadata ../rs-retry
```

Expected: no `UnorderedNaN` matches; remaining left/right non-numeric matches correspond only to the new variants and their tests.

Commit:

```bash
git add src/numeric_comparison_error.rs \
  src/value/value_numeric_comparison.rs \
  tests/numeric_comparison_error_tests.rs \
  tests/value/value_numeric_comparison_tests.rs
git commit -m "feat(comparison): distinguish numeric operand errors"
```

---

### Task 5: Remove redundant DataTypeOf conversion bounds

**Files:**
- Modify: `src/multi_values/multi_values_converters.rs:13-20,91-349`
- Modify: `src/value_container.rs:19-26,379-450`
- Verify: `tests/feature_contract_tests.rs:75-124`
- Verify: `tests/public_api_boundary_tests.rs:146-182`

**Interfaces:**
- Consumes: `pub trait DataConversionTarget: DataTypeOf + Sized` from `qubit-datatype`.
- Produces: the same conversion API and behavior with only `T: DataConversionTarget` written at each call boundary.

- [ ] **Step 1: Run characterization tests before the refactor**

Run:

```bash
cargo +1.94.0 test --no-default-features --features converter \
  --test feature_contract_tests converter_feature_accepts_target_side_extension
cargo +1.94.0 test --all-features --test integration_tests \
  multi_values::multi_values_converters_tests
cargo +1.94.0 test --all-features --test integration_tests \
  test_value_container_conversion_covers_scalar_and_collection_dispatch
```

Expected: all tests pass and establish behavior before the signature-only cleanup.

- [ ] **Step 2: Simplify MultiValues converter bounds**

Remove `DataTypeOf` from the import list in `src/multi_values/multi_values_converters.rs`. In both private helpers and all eight public conversion methods, replace:

```rust
where
    T: DataConversionTarget,
    T: DataTypeOf,
```

with:

```rust
where
    T: DataConversionTarget,
```

Apply this exact change to `convert_first_with`, `convert_values_with`, `to`, `to_or`, `to_with`, `to_or_with`, `to_list`, `to_list_or`, `to_list_with`, and `to_list_or_with`. Do not change method bodies.

- [ ] **Step 3: Simplify ValueContainer converter bounds**

Remove `DataTypeOf` from the feature-gated import list in `src/value_container.rs`. Replace the same two-line bound with only `T: DataConversionTarget` in `to`, `to_with`, `to_list`, and `to_list_with`. Do not change dispatch or scalar-string splitting.

- [ ] **Step 4: Prove the redundant bound is gone and behavior is unchanged**

Run:

```bash
rg -n -U 'T: DataConversionTarget,\n[[:space:]]*T: DataTypeOf' src
cargo +1.94.0 fmt --check
cargo +1.94.0 test --no-default-features --features converter \
  --test feature_contract_tests converter_feature_accepts_target_side_extension
cargo +1.94.0 test --all-features --test integration_tests \
  multi_values::multi_values_converters_tests
cargo +1.94.0 test --all-features --test integration_tests \
  test_value_container_conversion_covers_scalar_and_collection_dispatch
```

Expected: `rg` prints no matches and every test passes.

- [ ] **Step 5: Commit the bound cleanup**

```bash
git add src/multi_values/multi_values_converters.rs src/value_container.rs
git commit -m "refactor(conversion): remove redundant target bounds"
```

---

### Task 6: Align documentation and verify the complete release

**Files:**
- Modify: `README.md:51-71`
- Modify: `README.zh_CN.md:40-55`
- Modify: `src/value/value.rs:43-71`
- Modify: `src/multi_values/multi_values.rs:31-61`
- Verify: `.rs-ci-cargo-matrix.json`
- Verify downstream: `../rs-config`, `../rs-metadata`, `../rs-retry`

**Interfaces:**
- Consumes: the final public trait and comparison behavior from Tasks 1-5.
- Produces: aligned English/Chinese user guidance and a fully verified change set across every direct local consumer.

- [ ] **Step 1: Document lawful runtime identity in English**

Replace the identity paragraph in `README.md` with:

```markdown
`Value`, `MultiValues`, `ValueContainer`, `NamedValue`, `NamedMultiValues`, and
`ValueWireV1` implement lawful `Eq` and `Hash`. Variants and scalar/collection
shape remain distinct; signed zeros and all NaN payloads within the same float
width are canonicalized; string maps and JSON objects hash independently of key
iteration order; and collection order remains significant. Use `numeric_cmp`
with an explicit `NumericComparisonPolicy` for mathematical comparison across
numeric variants.

These implementations are intended for Rust hash collections and in-memory
caches. Their hash output is not a stable fingerprint: it may change with the
hasher, Rust version, crate version, enabled features, platform, or
implementation. Do not persist `DefaultHasher` output or use it as a
distributed-cache key. A persistent identity format requires a separately
versioned canonical byte representation and fingerprint API.
```

- [ ] **Step 2: Add the aligned Chinese guidance**

Replace the corresponding paragraph in `README.zh_CN.md` with:

```markdown
`Value`、`MultiValues`、`ValueContainer`、`NamedValue`、`NamedMultiValues` 与
`ValueWireV1` 均实现了满足约束的 `Eq` 和 `Hash`。不同变体以及标量/集合形态保持
不同；同一浮点宽度内的正负零和所有 NaN payload 会被规范化；字符串映射与 JSON
对象的 hash 不依赖键迭代顺序；集合元素顺序仍然有意义。需要跨数值变体按数学值
比较时，请使用带显式 `NumericComparisonPolicy` 的 `numeric_cmp`。

这些实现面向 Rust hash 集合和进程内缓存，其 hash 输出不是稳定指纹：结果可能随
hasher、Rust 版本、crate 版本、启用的 feature、平台或实现变化。不要持久化
`DefaultHasher` 输出，也不要把它用作分布式缓存键。持久身份需要另行设计带版本的
规范字节表示与 fingerprint API。
```

- [ ] **Step 3: Align Value and MultiValues Rustdoc with implemented behavior**

Add this section after the behavior bullets in the generated `Value` Rustdoc in `src/value/value.rs`:

```rust
/// # Equality and hashing
///
/// Equality preserves enum-variant identity. Signed zero is canonicalized,
/// every NaN payload within one float width is equal, and unordered payloads
/// hash structurally. Standard hash output is suitable for in-memory keys but
/// is not a stable persistent fingerprint.
```

Add this corresponding section to the generated `MultiValues` Rustdoc in `src/multi_values/multi_values.rs`:

```rust
/// # Equality and hashing
///
/// Equality preserves the collection variant and element order. Float
/// elements use canonical signed-zero and NaN identity, while map-like
/// elements hash structurally. Standard hash output is suitable for in-memory
/// keys but is not a stable persistent fingerprint.
```

- [ ] **Step 4: Run formatting, documentation, and the full rs-value matrix**

Run from `rs-value`:

```bash
cargo +1.94.0 fmt --check
RUSTDOCFLAGS="-D warnings" cargo +1.94.0 doc --no-deps --all-features
cargo +1.94.0 test --all-features
./ci-check.sh
git diff --check
```

Expected: every command passes, including minimal, each optional type family, converter combinations, all-feature tests, rustdoc, and configured Clippy checks.

- [ ] **Step 5: Run focused static audits**

Run:

```bash
rg -n 'UnorderedNaN' src tests README.md README.zh_CN.md
rg -n -U 'T: DataConversionTarget,\n[[:space:]]*T: DataTypeOf' src
rg -n 'BigDecimal.*\.hash|\.hash\([^)]*BigDecimal' src
rg -n 'stable (hash|digest|fingerprint)|稳定.*(hash|摘要|指纹)' \
  README.md README.zh_CN.md src/value/value.rs src/multi_values/multi_values.rs
```

Expected: the first three searches print no matches. The final search may match only the explicit warnings that standard hashes are not stable fingerprints.

- [ ] **Step 6: Verify every direct local downstream crate**

Run from each repository in dependency order:

```bash
cd ../rs-config
cargo +1.94.0 test --all-features

cd ../rs-metadata
cargo +1.94.0 test --all-features

cd ../rs-retry
cargo +1.94.0 test --all-features
```

Expected: all tests pass. `rs-config` preserves serialized configuration and conversion behavior; `rs-metadata` preserves explicit-policy filtering; `rs-retry` continues to consume `ValueError` without matching the changed comparison error.

- [ ] **Step 7: Commit documentation separately**

Return to `rs-value` and commit only the documentation changes:

```bash
git add README.md README.zh_CN.md \
  src/value/value.rs src/multi_values/multi_values.rs
git commit -m "docs(identity): clarify runtime hash semantics"
```

- [ ] **Step 8: Inspect final repository state without pushing**

Run:

```bash
git status --short
git --no-pager log -6 --oneline
```

Expected: the worktree is clean and the six English, intent-grouped commits appear in task order. Do not push. If a worktree was used, merge these commits into the original repository's current branch, rerun `git status --short`, then remove the merged worktree.
