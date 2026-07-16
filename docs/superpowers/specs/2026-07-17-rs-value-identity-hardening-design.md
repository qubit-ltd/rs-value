# rs-value Identity and Numeric Error Hardening

## Status

Approved for implementation on 2026-07-17.

This design follows the 2026-07-16 runtime redesign. It preserves that
design's separation between representation identity and policy-driven numeric
comparison while hardening BigDecimal hashing and extending lawful identity to
the complete value-container family.

Breaking changes to `NumericComparisonError` are intentional. Compatibility
aliases and deprecated error variants are out of scope.

## Context

`Value` now implements a lawful `PartialEq + Eq + Hash` contract. Different
variants remain distinct, signed zero is canonicalized, all NaN payloads of the
same width are equal, and unordered maps and JSON objects hash independently of
iteration order.

Four gaps remain:

1. `Value::BigDecimal` delegates to `bigdecimal::BigDecimal::hash`. In
   `bigdecimal` 0.4.10, hashing a non-zero value with a negative scale expands
   that scale into a zero-filled string. Hashing cost can therefore depend on
   the absolute scale rather than the stored coefficient size. The V1 wire
   payload accepts an arbitrary `i64` scale, so this is reachable from both
   direct construction and deserialization.
2. `MultiValues` still derives ordinary `PartialEq`. A collection containing
   NaN is not equal to itself, and `MultiValues` cannot implement `Eq` or
   `Hash`. This limitation propagates to `ValueContainer`, the named wrappers,
   and `ValueWireV1`.
3. Conversion APIs redundantly require both `DataConversionTarget` and
   `DataTypeOf`, even though the former already inherits the latter.
4. `NumericComparisonError` conflates a typed unset value with a concrete
   non-numeric value and does not identify which operand contains NaN.

The README also needs to distinguish Rust's standard in-memory `Hash` contract
from a stable cross-process or persistent fingerprint.

## Goals

1. Make BigDecimal hashing proportional to the stored coefficient size and
   independent of the absolute scale.
2. Preserve BigDecimal's equality semantics while guaranteeing that equal
   decimals produce equal hashes.
3. Give `MultiValues`, `ValueContainer`, `NamedValue`, `NamedMultiValues`, and
   `ValueWireV1` lawful `Eq + Hash` implementations consistent with `Value`.
4. Remove redundant `DataTypeOf` bounds from conversion APIs.
5. Distinguish missing, non-numeric, and NaN operands in numeric comparison
   errors.
6. Document that standard hashing is not a versioned persistent fingerprint.

## Non-goals

- Do not change `Value` equality into cross-variant mathematical equality.
- Do not implement `PartialOrd` or `Ord` for runtime value containers.
- Do not make standard hash outputs stable across Rust versions, crate
  versions, feature sets, platforms, or hasher implementations.
- Do not add a canonical fingerprint or cryptographic digest API.
- Do not move the BigDecimal hashing helper into `rs-datatype` yet.
- Do not validate or reject large BigDecimal scales solely to protect hashing.
- Do not redesign `ValueError::NoValue` or the unset/empty collection model.
- Do not redesign or remove the named wrappers or `ValueWireV1`.
- Do not add `Hash` to `rs-config::Config` or other downstream aggregate types.

## Internal identity module

Introduce a private, responsibility-specific module instead of a generic
`utils` module:

```text
src/
├── identity/
│   ├── mod.rs
│   └── big_decimal_hash.rs
├── value/
│   └── value_identity.rs
└── multi_values/
    └── multi_values_identity.rs
```

`src/identity/mod.rs` owns crate-private payload identity helpers shared by
single- and multi-value containers:

- canonical `f32` identity bits;
- canonical `f64` identity bits;
- structural JSON equality and hashing;
- sorted string-map hashing;
- the feature-gated BigDecimal hashing export.

`src/identity/big_decimal_hash.rs` contains only the standalone BigDecimal
hashing algorithm. It must not import `Value`, `MultiValues`, wire types, or
conversion types. This keeps the file independently movable if another crate
later needs the same algorithm.

The helper is crate-private:

```rust
pub(crate) fn hash_big_decimal<H: Hasher>(
    value: &BigDecimal,
    state: &mut H,
);
```

No public hashing utility is added in this release. A future migration to
`rs-datatype` or a dedicated crate requires evidence from at least one
additional production consumer and a separate API design.

## Canonical BigDecimal hashing

### Required identity

BigDecimal equality remains delegated to `BigDecimal::eq`. The replacement
hash must satisfy the one-way Rust hash contract:

```text
left == right  =>  hash(left) == hash(right)
```

Unequal decimals may collide, as permitted by `Hash`, but the implementation
should retain the natural distinction between normalized coefficient/scale
pairs.

### Algorithm

For a `BigDecimal`, obtain `(coefficient, scale)` from
`as_bigint_and_exponent()`.

1. If the coefficient is zero, hash one fixed zero marker. Scale and textual
   sign do not participate because every zero representation is equal.
2. Convert the non-zero coefficient to its base-10 coefficient string. This
   allocation is bounded by the coefficient's stored digit count.
3. Count and remove only trailing ASCII `0` digits from that coefficient
   string. A leading minus sign is retained.
4. Compute the effective scale as:

   ```text
   effective_scale = i128::from(scale) - i128::from(trailing_zero_count)
   ```

   `i128` prevents overflow for every currently supported target where
   `usize` is at most 64 bits and handles `i64::MIN` without calling `abs`.
5. Hash a non-zero domain marker, the normalized coefficient slice, and the
   effective scale in that order.

The implementation must never:

- call `BigDecimal::hash`;
- call `abs` on the scale;
- allocate or iterate `abs(scale)` bytes;
- format the expanded decimal value;
- serialize through JSON or the V1 wire format.

The algorithm is `O(d)`, where `d` is the number of stored coefficient digits.
Its memory use is also `O(d)` and is independent of the scale magnitude.

### Examples

The following representations hash identically because BigDecimal equality
considers them equal:

```text
coefficient=1,     scale=0
coefficient=10,    scale=1
coefficient=10000, scale=4
```

Similarly, all zero coefficients hash identically regardless of scale.

## Shared payload identity

The internal identity module preserves the existing payload rules:

- `f32`: normalize both signed zeros to positive zero and every NaN payload to
  one `f32` NaN bit pattern;
- `f64`: the same rule within the `f64` width;
- `HashMap<String, String>`: compare through map equality and hash entries in
  sorted key order;
- JSON: structural equality, ordered arrays, and recursively sorted object
  hashing;
- BigDecimal: use `BigDecimal::eq` and the new canonical hash helper;
- all remaining payloads: delegate to their own `Eq + Hash` implementations.

The shared helpers remain implementation details. `Value` and `MultiValues`
own their trait implementations and variant dispatch.

## MultiValues equality and hashing

Remove the derived `PartialEq` implementation from `MultiValues` and add a
manual implementation in `multi_values_identity.rs`.

### Equality

- Different enum variants are always unequal.
- Two `Unset` variants are equal only when their declared `DataType` values are
  equal.
- Concrete collections compare lengths and elements in order.
- Each element uses the same payload identity rules as the corresponding
  `Value` variant.
- Empty vectors of different variants remain unequal.
- A concrete empty vector remains unequal to `Unset` of the same type.

These rules make collections containing NaN reflexive, symmetric, and
transitive.

### Hashing

- Hash the collection variant discriminant.
- Hash the declared type for `Unset`.
- For a concrete collection, hash its length and every element in order using
  the shared payload hash helper.
- Do not construct temporary `Value` instances for collection elements.
- Do not serialize collections as an intermediate representation.

Collection order remains significant. For example, `[1, 2]` and `[2, 1]` are
not equal and generally hash differently.

## Trait propagation

After `MultiValues` implements `Eq + Hash`, extend the same lawful identity to:

- `ValueContainer`;
- `NamedValue`;
- `NamedMultiValues`;
- `ValueWireV1`.

These types may derive `PartialEq + Eq + Hash` when their fields make the
derived behavior identical to the documented semantics.

`ValueContainer` identity includes shape:

- `Scalar(Value::Int32(1))` is not equal to
  `Collection(MultiValues::Int32(vec![1]))`;
- scalar unset and collection unset remain distinct even with the same
  declared data type.

Names participate in named-wrapper identity. The V1 DTO identity is the
identity of its contained shape and payload, not its serialized byte sequence.

## Conversion bound cleanup

`DataConversionTarget` already has the declaration:

```rust
pub trait DataConversionTarget: DataTypeOf + Sized
```

Remove explicit `T: DataTypeOf` bounds wherever `T: DataConversionTarget` is
already present in `rs-value`, including:

- internal batch conversion helpers;
- all `MultiValues::{to,to_with,to_or,to_or_with}` methods;
- all `MultiValues::{to_list,to_list_with,to_list_or,to_list_or_with}` methods;
- all `ValueContainer::{to,to_with,to_list,to_list_with}` methods.

Remove imports that become unused. This is a source-level simplification only;
it does not widen or narrow the set of supported target types.

## Numeric comparison errors

Replace the current error variants with:

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

No compatibility variants are retained.

### Classification order

`Value::numeric_cmp` performs deterministic validation before delegating to the
numeric kernel:

1. If the left operand is `Unset`, return `LeftMissing`.
2. If the right operand is `Unset`, return `RightMissing`.
3. If the left concrete operand is non-numeric, return `LeftNotNumeric`.
4. If the right concrete operand is non-numeric, return `RightNotNumeric`.
5. Detect NaN on both numeric operands:
   - both NaN: `BothNaN`;
   - only left: `LeftNaN`;
   - only right: `RightNaN`.
6. Delegate the remaining concrete ordered numeric pair to `compare_numeric`.

After these checks, the lower-level kernel must return `Some(Ordering)` for the
`NumericValueRef` values produced by `Value`. A defensive internal assertion
may document this invariant, but public code must not misclassify a future
unexpected `None` as a positioned NaN.

The error messages contain operand position and data type where relevant but
never include source values.

## Standard Hash versus persistent fingerprints

The English and Chinese READMEs and the `Value` rustdoc must state:

- `Eq + Hash` makes the runtime types lawful keys for Rust hash collections and
  in-memory caches;
- order-independent map hashing does not make the resulting `u64` a stable
  digest;
- hash outputs may change with the chosen hasher, Rust version, crate version,
  feature set, platform, or implementation changes;
- callers must not persist `DefaultHasher` output or use it as a distributed
  cache key;
- a future persistent identity format requires a separately versioned,
  canonical byte representation and fingerprint API.

This release does not introduce that API.

## Downstream impact

### rs-config

`Property` and `Config` continue to use `ValueContainer` through their existing
APIs. Their serialized representation and conversion behavior do not change.
Existing equality becomes reflexive for properties containing float NaN inside
collections.

No `Hash` implementation is added to `Config`; its property map and
configuration-specific options need a separate identity decision.

### rs-metadata

Metadata continues to store scalar `Value` instances. Numeric filter equality
continues to use explicit `NumericComparisonPolicy`, not `Value::eq` or a hash
set, because approximate comparison is not suitable as hash-key equality.

Existing calls that discard `numeric_cmp` errors remain source-compatible
except for tests or code that explicitly pattern-match the renamed public error
variants.

### Other rs-* crates

Search all manifests and Rust sources for direct `qubit-value` dependencies and
explicit `NumericComparisonError` matches. Crates that only receive
`ValueError` transitively require no change.

## Testing strategy

Tests remain external and mirror source paths.

### BigDecimal hashing

- equal values with different coefficient/scale encodings hash identically;
- zero values with positive, zero, negative, maximum, and minimum scales hash
  identically;
- positive and negative non-zero values remain reflexive and hashable;
- extreme negative scale values hash without scale-sized expansion;
- debug-only regression coverage exercises `i64::MIN` so an accidental return
  to upstream `BigDecimal::hash` fails promptly instead of allocating;
- a `HashSet<Value>` deduplicates equivalent BigDecimal representations.

Tests must avoid inputs that could intentionally allocate unbounded memory if
the implementation regresses in release mode.

### MultiValues identity

- every enabled variant is reflexive and hashable;
- different variants remain unequal;
- `Unset` includes the declared type;
- concrete empty and unset collections remain distinct;
- positive and negative zero elements compare and hash equally;
- different NaN payloads compare and hash equally for each float width;
- collection element order remains significant;
- StringMap insertion order is irrelevant within each element;
- nested JSON object order is irrelevant within each element;
- BigDecimal elements use canonical hashing;
- `HashSet<MultiValues>` and `HashMap<MultiValues, _>` compile and behave as
  expected.

### Trait propagation

- `ValueContainer`, `NamedValue`, `NamedMultiValues`, and `ValueWireV1` satisfy
  compile-time `Eq + Hash` bounds;
- shape and name remain part of identity;
- representative instances work as hash collection keys.

### Numeric comparison errors

- left and right unset values report the declared type;
- left and right concrete non-numeric values report the actual type;
- left-only, right-only, and dual NaN cases have distinct errors;
- ordered numeric pairs preserve exact and approximate comparison behavior;
- downstream metadata filters preserve their current match results.

### Feature and downstream verification

- default feature tests compile without BigDecimal or JSON helpers;
- `big-number`, `json`, and `all` feature builds cover their identity helpers;
- repository feature-contract tests continue to exercise supported feature
  combinations;
- `rs-config` and `rs-metadata` test suites pass against the new local
  `rs-value`;
- stale searches find no redundant `T: DataTypeOf` bound paired with
  `T: DataConversionTarget` in `rs-value`.

## Implementation order

1. Add failing public-behavior tests for BigDecimal hashing.
2. Add the private identity module and standalone BigDecimal hash helper.
3. Refactor `Value` identity to use shared helpers.
4. Add failing `MultiValues` identity tests and implement its manual traits.
5. Propagate `Eq + Hash` to the wrapper types and add compile-contract tests.
6. Replace `NumericComparisonError` and migrate direct callers and tests.
7. Remove redundant conversion bounds and imports.
8. Update English and Chinese README text and affected rustdoc.
9. Verify `rs-value`, then `rs-config`, `rs-metadata`, and every discovered
   direct downstream crate in dependency order.

## Success criteria

- No `Value` or `MultiValues` hash path calls `BigDecimal::hash`.
- BigDecimal hashing work is bounded by stored coefficient size, not scale.
- Every runtime value container in `rs-value` has lawful `Eq + Hash` semantics.
- Collections containing NaN are equal to themselves.
- String-map and JSON object order independence is preserved at every nesting
  level and inside collections.
- Conversion methods require only `T: DataConversionTarget`.
- Numeric comparison errors distinguish missing operands, concrete
  non-numeric operands, and NaN position.
- Documentation does not imply that standard hashes are stable persistent
  fingerprints.
- All affected feature, crate, downstream, and documentation checks pass.
