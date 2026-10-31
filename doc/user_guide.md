# qubit-value User Guide

[中文版本](user_guide.zh_CN.md) · [README](../README.md) · [Design](design.md) · [API documentation](https://docs.rs/qubit-value)

## Purpose and audience

This guide is for Rust developers who receive configuration, metadata, protocol
fields, or other values whose concrete type is known at runtime. It explains
how to keep those values type-safe without forcing every caller to create a
different ad-hoc enum.

The guide covers `qubit-value` 0.13. It focuses on the value layer. It does not
turn `qubit-value` into a configuration service, schema registry, or persistent
database. For ready-made key-value containers built directly on `Value`, see
[`rs-config`](https://github.com/qubit-ltd/rs-config) and
[`rs-metadata`](https://github.com/qubit-ltd/rs-metadata) near the end.

Rust snippets omit the surrounding function and assume a compatible `Result`
return type, so errors are propagated with `?` instead of being hidden by
`expect`.

## The problem and the model

A runtime value has two independent properties:

1. its declared `DataType`, such as `Int32`, `Duration`, or `StringMap`;
2. its shape, either one scalar or a homogeneous collection.

`qubit-value` keeps both properties explicit:

| Type | Stores | Use it when |
| --- | --- | --- |
| `Value` | one typed scalar or `Unset(DataType)` | a key has one value |
| `MultiValues` | one homogeneous collection or typed unset state | a key accepts repeated values |
| `ValueContainer` | `Scalar(Value)` or `Collection(MultiValues)` | the source shape itself matters |
| `NamedValue` | a name plus `Value` | a named property must travel with its value |
| `NamedMultiValues` | a name plus `MultiValues` | a named repeated property must travel with its values |

There are three states that should not be collapsed:

- `Unset(DataType::String)` means the type is declared, but no concrete value is
  present;
- `MultiValues::String(vec![])` is a concrete empty collection;
- `Json(Null)` is a concrete JSON value when the `json` feature is enabled.

`ValueContainer` also prevents a one-item collection from becoming a scalar:
`Collection(MultiValues::Int32(vec![42]))` remains a collection at every API and
Wire boundary.

## Practical scenario: read a runtime configuration object

Suppose a service receives scalar configuration properties `host`, `port`,
`timeout`, and `debug` at runtime. It also receives repeated `tags` input. The
success criteria are:

- a text port can be converted to `u16` with a range-checked error;
- an unset timeout can use a default without changing its declared type;
- tags remain a collection even when there is one tag;
- the same values can later be serialized with their runtime type and shape.

The core path is:

<!-- example:runtime-config run -->
```rust
use std::collections::HashMap;
use std::time::Duration;

use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value, ValueContainer};

let config = HashMap::from([
    ("host".to_owned(), Value::new("localhost".to_owned())),
    ("port".to_owned(), Value::new("8080".to_owned())),
    ("debug".to_owned(), Value::new(false)),
    (
        "timeout".to_owned(),
        Value::new_unset(DataType::Duration),
    ),
]);

let host: String = config["host"].get()?;
assert_eq!(host, "localhost");

let port: u16 = config["port"].to()?;
assert_eq!(port, 8080);

let debug: bool = config["debug"].get()?;
assert!(!debug);

let timeout: Duration = config["timeout"].get_or(Duration::from_secs(30))?;
assert_eq!(timeout, Duration::from_secs(30));

let tags = ValueContainer::Collection(MultiValues::new(["production"]));
assert!(tags.is_collection());
assert_eq!(tags.data_type(), DataType::String);
```

If this small map needs to become a complete, general-purpose configuration
object, use `Config` from [`rs-config`](https://github.com/qubit-ltd/rs-config).
It is built on `Value` and adds higher-level capabilities such as property
management, typed and multi-value reads, defaults, sections, conversion
policies, interpolation, and pluggable file/environment configuration sources.

The next step can encode `tags` or a larger `ValueContainer` through Wire V1.
The following sections explain the type table, feature selection, errors, and
the serialization choices before showing that complete round trip.

## Installation and feature selection

Use the following dependencies to run every example in this guide. The scenario
itself needs only `qubit-value` with `converter` plus `qubit-datatype`; the
additional crates support the Wire and embedded-document sections.

```toml
[dependencies]
qubit-value = { version = "0.13", features = ["all"] }
qubit-datatype = { version = "0.15", default-features = false, features = ["converter"] }
bigdecimal = "0.4"
qubit-budget = { version = "0.10", features = ["json"] }
qubit-json = "0.12"
qubit-redact = { version = "0.10", default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

The default feature set is empty. Add only the families required by the
application:

| Feature | Enables |
| --- | --- |
| `converter` | `Value::to`, `MultiValues::to_list`, and related conversion APIs |
| `chrono` | `Date`, `Time`, `DateTime`, and `Instant` |
| `big-integer` | `BigInteger` |
| `big-decimal` | `BigDecimal` |
| `big-number` | Compatibility alias for `big-integer` and `big-decimal` |
| `url` | `Url` |
| `json` | `Json` and bounded versioned JSON Wire encoding/decoding |
| `natural-json` | Convenience alias for `converter` + `json`; their combination also enables Natural JSON |
| `redact` | `Value` redacted views; the application also imports `Redact` from `qubit-redact` |
| `all` | `converter`, `chrono`, `big-number`, `url`, `json`, `natural-json`, and `redact` |

When exchanging concrete values between builds, the producer and consumer must
agree on the features needed by those values. A build without `chrono` can
still understand an unset declaration such as `Unset(DataType::Date)`, but it
cannot materialize a concrete `Date` payload.

## Supported `DataType` values

`DataType` is defined by `qubit-datatype` and is the complete type vocabulary
used by this crate. The 25 variants are:

| `DataType` | Rust representation | Feature | Notes |
| --- | --- | --- | --- |
| `Bool` | `bool` | — | Boolean value |
| `Char` | `char` | — | Unicode character |
| `Int8` | `i8` | — | 8-bit signed integer |
| `Int16` | `i16` | — | 16-bit signed integer |
| `Int32` | `i32` | — | 32-bit signed integer |
| `Int64` | `i64` | — | 64-bit signed integer |
| `Int128` | `i128` | — | 128-bit signed integer; Wire uses decimal text |
| `UInt8` | `u8` | — | 8-bit unsigned integer |
| `UInt16` | `u16` | — | 16-bit unsigned integer |
| `UInt32` | `u32` | — | 32-bit unsigned integer |
| `UInt64` | `u64` | — | 64-bit unsigned integer |
| `UInt128` | `u128` | — | 128-bit unsigned integer; Wire uses decimal text |
| `Float32` | `f32` | — | 32-bit float; Wire requires finite values |
| `Float64` | `f64` | — | 64-bit float; Wire requires finite values |
| `String` | `String` | — | UTF-8 text |
| `Date` | `chrono::NaiveDate` | `chrono` | Calendar date |
| `Time` | `chrono::NaiveTime` | `chrono` | Time of day |
| `DateTime` | `chrono::NaiveDateTime` | `chrono` | Date and local time |
| `Instant` | `chrono::DateTime<chrono::Utc>` | `chrono` | UTC time point |
| `BigInteger` | `num_bigint::BigInt` | `big-integer` | Arbitrary-precision integer |
| `BigDecimal` | `bigdecimal::BigDecimal` | `big-decimal` | Exact decimal with bounded Wire scale |
| `Duration` | `std::time::Duration` | — | Seconds/nanoseconds in Wire; text conversion is policy-driven |
| `Url` | `url::Url` | `url` | Parsed URL |
| `StringMap` | `HashMap<String, String>` | — | String-to-string map |
| `Json` | `serde_json::Value` | `json` | Arbitrary JSON structure |

The feature column describes the `qubit-value` feature required for a concrete
Rust value. `StringMap` is a native map type and does not require `json`; the
`Json` variant does.

## Core workflow

### Construct and inspect a single value

Use a typed constructor when the Rust type is known. Use `Value::new_unset` when
the key has a declared type but no value yet.

<!-- example:single-value run -->
```rust
use qubit_datatype::DataType;
use qubit_value::Value;

let mut value = Value::new(8080i32);
let port: i32 = value.get()?;
assert_eq!(port, 8080);
assert_eq!(value.data_type(), DataType::Int32);

value.unset();
assert!(value.is_unset());
assert_eq!(value.data_type(), DataType::Int32);

value.set_type(DataType::String);
value.set("8080");
assert_eq!(value.get_string()?, "8080");
```

`get<T>()` is strict. The stored variant must match `T`; it never guesses that
an integer string should be parsed. `get_or` supplies a default only when the
value is unset. A type mismatch is still an error.

### Construct and mutate homogeneous collections

`MultiValues::new`, `set`, and `add` accept vectors, arrays, slices, borrowed
vectors, and borrowed string collections where the corresponding conversion is
implemented.

<!-- example:multi-values run -->
```rust
use qubit_value::MultiValues;

let mut ports = MultiValues::new([8080i32, 8081, 8082]);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.set([9000, 9001]);
assert_eq!(ports.len(), 2);

let first: i32 = ports.get_first()?;
assert_eq!(first, 9000);
```

`add` is fallible because it must reject a different element type. `set`
replaces the entire collection and can change its element type. `Unset` and a
concrete empty collection remain distinguishable through `is_unset()` and
`is_empty()`.

#### First-item reads and source compatibility

`get_first`, `get_first_or`, and `get_first_or_else` return one scalar item
and require `T: StrictValueRead`. `get_first_ref` borrows one scalar item and
requires conversions from both `Value` and `MultiValues`. These bounds reject
collection targets at compile time. Use the complete-collection APIs when the
whole list is intended:

| Previous call | Use instead |
| --- | --- |
| `get_first::<Vec<i32>>()` | `get::<i32>()` |
| `get_first_ref::<[i32]>()` | `get_slice::<i32>()` |
| `get_first_or::<Vec<i32>>(default)` | `get_or::<i32>(default)` |
| `get_first_or_else::<Vec<i32>, _>(factory)` | `get_or_else::<i32, _>(factory)` |
| Scalar wrapper bounded only by `TryFrom<&MultiValues>` | Bound it by `T: StrictValueRead` |

The stricter bound is a source-compatibility change for collection targets and
generic wrappers with the old bound. Scalar reads, unset defaults, and
`get_first_or_else`'s lazy fallback behavior are unchanged.

### Convert with explicit policy

For adapters that must inspect runtime types, `Value::view()` returns
`ValueRef` and `MultiValues::view()` returns `MultiValuesRef`. Strings, maps,
JSON trees, and collection slices remain borrowed. The collection view offers
`data_type()`, `len()`, `is_empty()`, and `get(index)`; an indexed item is an
optional `ValueRef`, so no temporary owned `Value` is needed. Keep a wildcard
arm when matching these non-exhaustive views. With `converter`, a `ValueRef`
can be passed directly to `DataConverter`.

With `converter`, `to` applies the shared `qubit-datatype` conversion contract.
Use `to_with` when the default strict policy is not the policy the application
wants.

<!-- example:conversion run -->
```rust
use qubit_value::Value;

let text = Value::new("42".to_owned());
let number: u32 = text.to()?;
assert_eq!(number, 42);

let fallback: u16 = Value::new_unset(qubit_datatype::DataType::UInt16)
    .to_or(8080u16)?;
assert_eq!(fallback, 8080);
```

<a id="shared-conversion-session"></a>
### Share conversion limits across reads

Each `_with` call creates its own session. Use an `_in` method when related
reads should consume one shared budget. Successful item consumption accumulates
in that session; here the third conversion exceeds the shared two-item limit:

<!-- example:shared-conversion-session run -->
```rust
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionResource;
use qubit_datatype::ConversionSession;
use qubit_value::Value;
use qubit_value::ValueError;

let policy = ConversionPolicy::strict();
let limits = ConversionLimits::builder()
    .operation_limits(ConversionOperationLimits::builder().max_items(2).build())
    .build();
let mut session = ConversionSession::new(&policy, &limits);
assert_eq!(Value::from("41").to_in::<u16>(&mut session)?, 41);
assert_eq!(Value::from("42").to_in::<u16>(&mut session)?, 42);
let error = Value::from("43").to_in::<u16>(&mut session).unwrap_err();
assert!(matches!(error, ValueError::Conversion(ref source)
    if source.resource() == Some(ConversionResource::Items)));
assert_eq!(session.items_used(), 2);
```

`Value::to_in` reuses the caller's session. `MultiValues` and `ValueContainer`
provide corresponding `to_first_in` and `to_list_in` entry points. A session is
not an all-or-nothing batch transaction: successful consumption accumulates,
and a failed conversion may retain item or input consumption that was already
successfully charged. The precise failure accounting follows the
`qubit-datatype` contract; callers should not assume the whole batch rolls back.

`to_or` can also use a conversion policy's missing-value result, such as a
configured blank-as-missing behavior. It does not hide an ordinary type
mismatch or invalid conversion. The full source/target matrix and policy
details live in the [`qubit-datatype` API documentation](https://docs.rs/qubit-datatype/latest/qubit_datatype/).

Default parameters use `IntoValueDefault<T>`: for example, a string literal can
supply a `String` default without eagerly allocating it. Adaptation runs only
when the read needs a fallback. A policy-missing collection item never defaults,
including item zero in a first-item read; a concrete empty collection also
cannot supply a first-item default.

### Preserve names without changing value semantics

<!-- example:named-values run -->
```rust
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value};

let mut timeout = NamedValue::new("timeout", Value::new(30u64));
assert_eq!(timeout.name(), "timeout");
let timeout_seconds: u64 = timeout.value().get()?;
assert_eq!(timeout_seconds, 30);
timeout.value_mut().set(45u64);

let mut ports = NamedMultiValues::new("ports", MultiValues::new([8080u16, 8081]));
ports.values_mut().add(8082u16)?;
let first_port: u16 = ports
    .values()
    .get_first()
    ?;
assert_eq!(first_port, 8080);
```

<a id="numeric-comparison"></a>
## Compare numbers across runtime types

An `rs-metadata` filter may need to accept a stored numeric property even when
its concrete integer width differs from the query. `Value` identity still
includes the runtime variant: `Int32(1)` and `Int64(1)` are distinct values,
although `numeric_cmp` reports mathematical equality under the exact policy.
Import `NumericComparisonPolicy` from `qubit_datatype`; it is intentionally not
re-exported by `qubit_value`.

Use the exact policy when the filter needs a deterministic numeric ordering.
The application owns the acceptance decision; this example accepts equality
and separately handles an invalid NaN operand:

<!-- example:numeric-comparison run -->
```rust
use std::cmp::Ordering;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::{NumericComparisonError, Value};

let stored = Value::Int32(1);
let operand = Value::Int64(1);
assert_ne!(stored, operand);
let accepted = stored.numeric_cmp(&operand, NumericComparisonPolicy::Exact)? == Ordering::Equal;
assert!(accepted);
assert!(matches!(
    Value::Float64(f64::NAN).numeric_cmp(&operand, NumericComparisonPolicy::Exact),
    Err(NumericComparisonError::LeftNaN)
));
```

`numeric_cmp` returns `NumericComparisonError` for a missing left or right
operand, a nonnumeric left or right operand, or NaN on either side/both sides.
Validation checks missing operands left-to-right, then concrete types
left-to-right, then NaN positions. Unset does not mean zero. `Approximate` is
not a general tolerance or epsilon comparison: finite floats can cause mixed
representations to be projected to `f64`, and those pair-dependent results may
be non-transitive. Do not use it for `Ord`, sorting, grouping, or ordered
collection keys.

Numeric comparison is independent of conversion: `ConversionLimits` and
`ConversionSession` do not bound its work. Fixed-width comparisons do not
require heap allocation, while some paths involving large numbers may clone
coefficients or construct exact rational or decimal values. Cost depends on
the operand representations. Bound untrusted coefficient sizes and scales at
the input boundary, and account for cumulative comparison cost in batch
metadata filters. See the [lower-level comparison contract](https://docs.rs/qubit-datatype/latest/qubit_datatype/numeric/struct.NumberRef.html#method.compare).

<a id="wire-v1"></a>
## Wire V1: preserve type and shape across JSON

### When to use Wire V1

Runtime `Value` types do not implement Serde directly. Select a versioned Wire
adapter when a JSON boundary must reconstruct the exact runtime type and shape.
Use Natural JSON (`to_json_value`) when the receiver only needs ordinary JSON.

| Requirement | Use |
| --- | --- |
| Restore `Int32(42)` as `Int32`, not as an untyped JSON number | Wire V1 |
| Distinguish scalar `42` from collection `[42]` | Wire V1 |
| Preserve `Unset(DataType::String)` | Wire V1 |
| Produce normal `null`, numbers, strings, objects, and arrays for a JSON API | Natural JSON |

### Envelope, shape, and payload

`ValueWireV1` is the standalone version-one envelope. It contains numeric
`version: 1` and a typed `value` shape. The shape is either `scalar` or
`collection`; its payload key names are the lowercase Wire names of the
`DataType` variants.

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"string"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

`ValueWirePayloadV1` is the same typed shape without the outer version field;
use it only when another protocol already owns the versioned envelope.
`ValueWireRefV1` and `ValueWirePayloadRefV1` serialize borrowed values without
cloning them.

V1 rejects legacy bare enum representations such as `{"Int32":42}`. A
standalone document must use the versioned envelope shown above. The embedded
payload `{"scalar":{"int32":42}}` is valid only with the payload seed/DTO and
is not a standalone V1 document; a standalone document without `version` is
invalid.

### JSON number acceptance

Wire V1 accepts a runtime JSON number only when the strict number model can
reconstruct the same `serde_json::Number`: negative integers fit `i64`,
non-negative integers fit `u64`, and fractional or exponential values become
finite `f64` values without changing Number equality. This preserves the
runtime JSON identity used by `Value::Json`.

When another dependency enables `serde_json/arbitrary_precision`, a number such
as `1.2500`, `1e1`, or a decimal that cannot be preserved by `f64` is rejected
when constructing or serializing Wire data. The error does not include the
number text. V1 preserves accepted JSON values, not the spelling of the source
number token. A decoder-normalized number remains accepted if it meets the same
rule.

Callers that need exact wide integers or decimals should use the explicit
`BigInteger` or `BigDecimal` runtime types. For arbitrary numbers nested in a
JSON tree, define a string representation in the application schema. If
rounding is acceptable, convert to a standard finite `f64` before constructing
Wire data; the library does not round implicitly. Previously rounded data
cannot be recovered from Wire and must be reloaded from its original source.

JSON record positions in the envelope, named wrappers, and rich `Duration` and
`BigDecimal` payloads require objects with named fields. Array-shaped records
that may have been accepted by earlier builds are now rejected; change them to
objects such as `{"secs":1,"nanos":2}` or
`{"coefficient":"123","scale":4}`. Arrays remain valid for collection
payloads and for values whose declared type is `Json`.

For the canonical representations and limits of the feature-gated rich types,
see the [Wire V1 rich payload table](design.md#wire-v1-rich-payloads). The
following example shows one complete BigDecimal round trip. The JSON object is
the type's payload inside the versioned envelope; collections use an array of
payloads under the same tag. Its scale preserves the stored decimal
representation (`123.4500`), independently of `BigDecimal` identity/hash
normalization of trailing zeroes.

<!-- example:wire-rich-types run -->
```rust
use std::str::FromStr;
use bigdecimal::BigDecimal;
use qubit_value::{Value, ValueContainer, ValueWireV1};

let original = ValueContainer::from(Value::BigDecimal(BigDecimal::from_str("123.4500")?));
let wire = ValueWireV1::try_from(original.clone())?;
let bytes = wire.to_json_vec()?;
assert_eq!(
    serde_json::from_slice::<serde_json::Value>(&bytes)?,
    serde_json::json!({"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1234500","scale":4}}}})
);
let decoded = ValueWireV1::decode_json_slice(&bytes)?;
let restored: ValueContainer = decoded.into();
assert_eq!(restored, original);
```

V1 is closed. Existing tags, shapes, and payload representations cannot be
extended in place; a future runtime type requires a new wire version. String-map
keys and nested JSON object keys are emitted in lexicographic order under the
supported canonical JSON configuration.

Concrete `Instant` values must have a UTC year from 0 through 9999 so that
every encoded timestamp is readable by the V1 RFC3339 decoder. Encoding a
negative or five-digit year now returns `ValueWireEncodeError::InstantYearOutOfRange`;
this tightens acceptance for values that previously encoded but could not be
decoded. Keep such values in memory or choose an explicitly versioned
application representation before crossing a V1 boundary; normal four-digit
years need no migration.

`ValueWireEncodePreflight` uses conservative lower bounds. It now counts a
BigDecimal coefficient as string payload, but that estimate may be smaller
than its exact decimal text. JSON object admission also occurs before building
its sorting index, so an object rejected by its own structural limit avoids
that allocation. Preflight and final `JsonEncodeSession` accounting reduce
some avoidable work; they do not bound every encoding preparation allocation.

### End-to-end: `ValueContainer` to JSON and back

The following example creates an explicitly scalar value, converts it into the
owned Wire DTO, serializes it, applies input and semantic limits during decode,
and restores the original container.

<!-- example:wire-round-trip run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonEncodeLimits};
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;

let original = ValueContainer::Scalar(Value::new(8080i32));
let wire = ValueWireV1::try_from(original.clone())?;
let encode_limits = JsonEncodeLimits::builder()
    .max_output_bytes(64 * 1024)
    .max_depth(32)
    .max_nodes(128)
    .build();
let encoded = wire.to_json_vec_with_limits(encode_limits)?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"int32":8080}}}"#
);

let decode_limits = JsonDecodeLimits::builder()
    .max_input_bytes(64 * 1024)
    .max_depth(32)
    .max_nodes(128)
    .build();
let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits)?;
let restored: ValueContainer = decoded.into();

assert_eq!(restored, original);
assert!(restored.is_scalar());
assert_eq!(restored.data_type(), qubit_datatype::DataType::Int32);
```

The decode helpers accept a complete top-level Wire document and use the
`qubit-budget` JSON adapter. The Wire DTOs intentionally implement `Serialize`
only; they do not implement generic `Deserialize`, because a general Serde
deserializer cannot enforce raw-input and structural limits.
`NamedValue` and `NamedMultiValues` are embedding wrappers and do implement
generic `Deserialize`; they validate the V1 schema and scalar/collection shape
but inherit resource accounting from the supplied deserializer. Use their
bounded decode helpers for complete, untrusted JSON documents, or decode them
through a resource-bounded outer deserializer when nested.
`ValueWireV1::default_json_decode_limits()` and
`default_json_encode_limits()` provide the directional V1 profiles. Pass a
`JsonDecodeLimits` or `JsonEncodeLimits` value when the application owns a
different input, output, or value budget.

When `serde_json/arbitrary_precision` is unified through another dependency,
use these bounded helpers or a `JsonDecoder::decode_unique_*` path. Generic
Serde readers can expose JSON numbers as serde_json's private map encoding.
The unique-key decoder builds an intermediate tree, preserves object keys as
ordinary user keys, and enforces the configured `max_depth`; it has no separate
fixed 127-container limit. The tree uses extra memory, and errors from
deserializing the target type do not have original source coordinates. A
130-level value succeeds when the configured depth allows it, while direct
`serde_json::from_slice` may hit Serde's default recursion limit.

The decoder's nesting limit does not mean that every later operation on a value
is non-recursive. JSON equality and hashing use iterative traversal, while
cloning, ordinary `Debug` formatting, and dropping JSON payloads still follow
recursive paths in the underlying representation. Keep untrusted JSON nesting
bounded, and consider the thread-stack cost of these lifecycle operations before
raising a decoder depth limit. JSON constructed directly in memory does not pass
through decoder limits. Borrowed reads avoid unnecessary clones but do not
remove recursive destruction of the source value. There is no universal safe
depth across platforms and thread configurations.

### Borrowed Wire encoding

Use a borrowed adapter when the source value already lives long enough for the
serialization call and cloning would be unnecessary.

<!-- example:borrowed-wire run -->
```rust
use qubit_value::{Value, ValueWireRefV1};

let value = Value::new("service-a".to_owned());
let borrowed = ValueWireRefV1::from_value(&value)?;
let encoded = borrowed.to_json_vec()?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"string":"service-a"}}}"#
);
```

For an already versioned outer protocol, use
`ValueWirePayloadRefV1::from_value`, `from_values`, or `from_container` and
call `to_json_vec()` or `to_json_writer()` on the borrowed payload. These
constructors are fallible because they validate finite floats and bounded
`BigDecimal` scale before exposing a serializable payload. JSON object keys
have no serde_json Number-marker reservation.

All four Wire DTOs and both named wrappers also run a conservative payload
preflight when their explicit-limit `to_json_vec_with_limits` or
`to_json_writer_with_limits` helpers are called. The final encoder remains
authoritative for the complete payload or envelope. A lower-bound preflight
error can therefore reject early, while a successful preflight neither
guarantees encoding success nor bounds every preparation allocation.

### Embedded values and a shared `JsonDecodeSession`

`decode_json_slice_with_limits` is for a complete top-level Wire document. If a
value is nested inside a larger JSON document, use the shared `qubit-budget`
Serde adapter for the complete outer document so every JSON node is charged in
one session.

<!-- example:shared-decode-session run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
use qubit_json::decode::JsonDecoder;
use qubit_value::{ValueContainer, ValueWireV1Seed};

let input = br#"{"version":1,"value":{"collection":{"int32":[1,2]}}}"#;
let limits = JsonDecodeLimits::builder()
    .max_input_bytes(64usize * 1024)
    .build();
let session = JsonDecodeSession::from_limits(limits);
let mut decoder = JsonDecoder::new(session);
let decoded = decoder.decode_unique_seed_utf8(ValueWireV1Seed::new(), input)?;
let restored: ValueContainer = decoded.into();
assert!(restored.is_collection());
```

For a value embedded in a larger outer object, call `next_value_seed` with the
same `ValueWireV1Seed` (or `ValueWirePayloadV1Seed` when the outer protocol owns
the version). The outer object and embedded V1 envelope then share one JSON
document budget. Reusing the same `JsonDecoder` for later complete documents
keeps session accounting cumulative, and each call rejects trailing content.

### Embedded payload budget: one outer session

An outer struct can invoke `ValueWirePayloadV1Seed` from a field
`deserialize_with` function. The derived visitor then charges both fields to
the same decoder session. Each 32-byte string fits by itself, but the two
payloads together exceed a 64-byte cumulative payload budget.

<!-- example:embedded-payload-budget run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
use qubit_json::decode::JsonDecoder;
use qubit_value::{ValueWirePayloadV1, ValueWirePayloadV1Seed};
use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeSeed;

#[derive(Debug, Deserialize)]
struct Outer {
    #[serde(deserialize_with = "decode_payload")]
    first: ValueWirePayloadV1,
    #[serde(deserialize_with = "decode_payload")]
    second: ValueWirePayloadV1,
}

fn decode_payload<'de, D>(deserializer: D) -> Result<ValueWirePayloadV1, D::Error>
where
    D: Deserializer<'de>,
{
    ValueWirePayloadV1Seed::new().deserialize(deserializer)
}

let input = br#"{"first":{"scalar":{"string":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"second":{"scalar":{"string":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}"#;
let tight = JsonDecodeLimits::builder()
    .max_string_bytes(64usize)
    .max_payload_bytes(64usize)
    .build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(tight));
assert!(decoder.decode_unique_utf8::<Outer>(input).is_err());

let wide = JsonDecodeLimits::builder()
    .max_string_bytes(64usize)
    .max_payload_bytes(128usize)
    .build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(wide));
let outer = decoder.decode_unique_utf8::<Outer>(input)?;
assert_eq!(outer.first.container().data_type().as_str(), "string");
assert_eq!(outer.second.container().data_type().as_str(), "string");

let numeric_input = br#"{"version":1,"value":{"scalar":{"uint64":18446744073709551615}}}"#;
let limits = JsonDecodeLimits::builder().max_input_bytes(4096usize).build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits));
let wire = decoder.decode_unique_seed_utf8(
    qubit_value::ValueWireV1Seed::new(),
    numeric_input,
)?;
let restored: qubit_value::ValueContainer = wire.into();
assert_eq!(restored.data_type(), qubit_datatype::DataType::UInt64);
```

### Wire-specific type and input boundaries

- `Int128`, `UInt128`, and `BigInteger` use canonical decimal strings rather
  than JSON numbers that might lose precision.
- `BigDecimal` uses an exact coefficient/scale payload and rejects a scale whose
  absolute value exceeds the V1 bound.
- `Duration` uses a `{ "secs": ..., "nanos": ... }` payload with nanos below
  one second.
- Concrete `Instant` payloads require UTC years from 0 through 9999. Negative
  and five-digit years now return `InstantYearOutOfRange`, a compatibility
  tightening for values that previously encoded but were not decodable. Keep
  such values in memory or use an explicitly versioned application format;
  ordinary four-digit years are unchanged.
- `Float32` and `Float64` may contain non-finite values in memory, but V1
  rejects NaN and infinities because JSON has no such number literals.
- `Json(null)` is a concrete JSON value and is distinct from `Unset(Json)`.
- Generic `Json` values use the `qubit-json` numeric contract: negative
  integers fit `i64`, non-negative integers fit `u64`, and fractions are finite
  `f64`. A former serde_json private Number-marker key is ordinary.
- Frontends receiving integers above `2^53 - 1` need a preserving parser and
  `BigInt` or string mapping. A JavaScript `n` suffix is not valid JSON.
- A concrete rich type can be decoded only by a build with its corresponding
  feature. Unsupported feature-gated payloads are rejected rather than guessed.
- Unknown fields, unknown types, wrong scalar/collection shapes, non-numeric
  versions, legacy bare enum representations, and standalone documents without
  the version envelope are rejected.

## Natural JSON

`natural-json` is a convenience alias for `converter` + `json`. Enabling both
features explicitly also exposes projection methods for existing callers.

`to_json_value_with` checks the complete operation before constructing the final
JSON tree. All collection elements share item, input-byte, output-payload,
structural-node and structural-payload limits; duration formatting uses one
shared conversion session. Depth includes the root, and the outer collection
array consumes one node. Input bytes count borrowed strings, map keys and values,
and JSON strings and keys. Output payload counts JSON strings, keys and number
text, excluding punctuation, escaping, booleans and null encoding bytes.
Big-number coefficient and scale limits are checked before formatting. Bounded
measurement and preflight add a traversal so final tree allocation occurs only
after every limit check succeeds. Source strings stay borrowed until the final
JSON string allocation; rich formatted values are cached and reused, including
Duration formatting through the shared session. Measuring integers and Float64
does not allocate a temporary heap string. Float32 currently creates a temporary
decimal String to preserve its existing projection semantics.

JSON numbers are range-checked during projection even when the `serde_json::Value`
was constructed directly in memory: integer tokens must fit `i64` or `u64`, and
fractional or exponent tokens must represent a finite `f64`. Out-of-range
integers and non-finite numbers return conversion errors. Represent arbitrary-
precision values as strings, or use the corresponding Wire rich-number payload
when the runtime type must be preserved.

Projection budget errors use `ValueError::JsonProjectionLimit`, preserving the
resource, exact budget facts and optional collection index. Ordinary conversion
errors retain their existing types. Each call starts fresh accounting. Use
Wire `JsonEncodeLimits` or a bounded rs-json encoder to constrain final encoded
bytes; `ConversionLimits` is not a JSON encoding length limit.

### Redact a named secret before displaying a value

When a value is shown in logs or diagnostics, define which named fields are
sensitive and use `qubit-redact` to produce the display text. This example
marks `api_key` as secret, keeps an ordinary label visible, then demonstrates
the explicit fallback for a result that cannot fit its output budget:

<!-- example:redacted-output run -->
```rust
use std::collections::HashMap;

use qubit_redact::RedactionPolicy;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_value::Value;

let value = Value::new(HashMap::from([
    ("api_key".to_owned(), "raw-secret".to_owned()),
    ("label".to_owned(), "visible".to_owned()),
]));
let policy = RedactionPolicy::builder()
    .fields(|fields| { fields.raise("api_key", Sensitivity::Secret); })?
    .build()?;
let output = Redactor::new(policy).redact_text(&value);
let text = output.text_or_marker("<redaction incomplete>");
assert!(!text.contains("raw-secret"));
assert!(text.contains("visible"));

let bounded = RedactionPolicy::builder()
    .limits(|limits| { limits.max_output_bytes(0); })?
    .build()?;
let output = Redactor::new(bounded).redact_text(&value);
assert_eq!(output.text_or_marker("<redaction incomplete>"), "<redaction incomplete>");
```

`Value`'s ordinary `Debug` output does not apply a redaction policy. A bare
scalar has no business field name for a field rule to match, and redaction
cannot identify every secret from its contents; classify fields explicitly
where the application knows their meaning.

### Bounded materialization from Serde

`Value::from_serializable` validates and materializes a strict JSON value but
does not use a finite resource budget. `JsonEncodeLimits` and
`ConversionLimits` do not limit its intermediate JSON tree. When serialized
input needs bounded materialization, encode it with a caller-owned
`JsonValueBudget` transaction, commit only after encoding succeeds, and then
wrap the resulting JSON value:

<!-- example:bounded-serializable run -->
```rust
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;
use qubit_json::value::JsonValueEncoder;
use qubit_value::Value;

let limits = JsonValueLimits::<JsonResource, usize>::builder()
    .max_depth(8)
    .max_nodes(64)
    .max_string_bytes(1024)
    .max_payload_bytes(4096)
    .build();
let mut budget = JsonValueBudget::new(limits);
let mut transaction = budget.transaction();
let owner = JsonValueEncoder::new().encode_in_transaction(&vec!["bounded"], &mut transaction)?;
transaction.commit()?;
let value = Value::from_json_value(owner.into_value());
assert_eq!(value.get_json()?.as_array().unwrap().len(), 1);
```

If encoding fails, propagate the error and drop the transaction without
committing it; no partial budget state is retained.

With `natural-json`, Natural JSON projects a runtime value into ordinary
`serde_json::Value`. The following example shows the exact JSON string emitted
for several common values:

<!-- example:natural-json run -->
```rust
use std::collections::HashMap;

use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value};

assert_eq!(Value::new(42i32).to_json_value()?.to_string(), "42");
assert_eq!(
    Value::new("localhost".to_owned())
        .to_json_value()?
        .to_string(),
    r#""localhost""#,
);
assert_eq!(
    Value::new_unset(DataType::String)
        .to_json_value()?
        .to_string(),
    "null",
);
assert_eq!(
    MultiValues::new([8080i32, 8081])
        .to_json_value()?
        .to_string(),
    "[8080,8081]",
);
assert_eq!(
    Value::new(HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ]))
    .to_json_value()?
    .to_string(),
    r#"{"a":"1","z":"26"}"#,
);
```

The resulting strings are `42`, `"localhost"`, `null`, `[8080,8081]`, and
`{"a":"1","z":"26"}` respectively. A scalar remains a scalar, an unset
value becomes `null`, a concrete collection always becomes an array, and
string-map keys are emitted in dictionary order.

For a single map value, the equivalent construction is:

<!-- example:natural-json-map run -->
```rust
use qubit_value::Value;

let value = Value::new(std::collections::HashMap::from([
    ("host".to_owned(), "localhost".to_owned()),
]));
let json = value.to_json_value()?;
assert_eq!(json.to_string(), r#"{"host":"localhost"}"#);
```

Natural JSON intentionally loses the runtime `DataType` tag. An unset value
projects to `null`, and every concrete collection projects to an array,
including a one-item collection. Use Wire V1 when those distinctions must be
reconstructed.

<a id="errors-and-diagnostics"></a>
## Errors and diagnostics

Value operations return `ValueResult<T>`, an alias for `Result<T, ValueError>`.
The important categories are:

| Error | Meaning |
| --- | --- |
| `ValueError::Missing` | value is unset, a collection is empty, or conversion produced no value |
| `ValueError::TypeMismatch` | strict `get<T>()` requested a different type |
| `ValueError::Conversion` | a scalar conversion is unsupported or invalid |
| `ValueError::ListConversion` | a collection conversion failed and retains the source index |
| `ValueWireEncodeError` | a value violates V1 encoding rules, such as a non-finite float |
| `ValueWireDecodeError` | JSON, version, shape, feature, or resource-limit validation failed |

Handle missing values separately from invalid values. A default is appropriate
for an intentionally absent configuration property, not for a malformed port or
a type mismatch. For diagnostics containing string maps or JSON objects, use
the explicit `redact` view when the application has sensitive fields; ordinary
`Debug` formatting is not implicitly redacted.

`ValueWireDecodeError::InvalidJson` carries `ValueWireJsonError`, which exposes
only the Serde category and line/column metadata. It does not retain the
original error message or input text. Target-type failures from the unique-key
tree path have no source coordinates and report zero for line and column.

This example converts a direct Serde error, which retains its source coordinates.

<!-- example:safe-wire-json-error run -->
```rust
use qubit_value::ValueWireDecodeError;
use serde_json::error::Category;

let source = serde_json::from_str::<u64>(r#""not a number""#).unwrap_err();
let error = ValueWireDecodeError::from(source);
let ValueWireDecodeError::InvalidJson(json) = error else {
    panic!("direct Serde conversion must preserve safe metadata");
};
assert_eq!(json.classify(), Category::Data);
assert!(json.line() > 0);
assert!(json.column() > 0);
```

This changes the `InvalidJson` payload from `serde_json::Error` to
`ValueWireJsonError`. Downstream code that names the old payload type,
constructs the variant directly, or downcasts its source should match the safe
type and use `classify()`, `line()`, and `column()` instead. Ordinary typed
collections now charge each visible element to the redaction item budget, so a
policy that previously printed every numeric item may now return a truncated
view.

## Troubleshooting

### `get<T>()` returns a type mismatch

Inspect `value.data_type()` and use a typed getter when the source type must be
exact. If conversion is intended, enable `converter` and call `to<T>()` with a
policy that matches the application.

<a id="default-values"></a>
### A default is not used

Check whether the container is actually unset. A concrete empty `MultiValues`
is not an unset collection. Also check whether the conversion policy classifies
the source as missing; ordinary invalid conversions do not use `to_or` defaults.

### Wire decoding rejects a value

Check, in order:

1. the input is one complete JSON document rather than a fragment;
2. `version` is numeric `1`;
3. the `scalar`/`collection` shape matches the intended container;
4. the receiving build enables the feature for the concrete type;
5. the input and decoded structure fit the supplied `JsonDecodeLimits` profile;
6. the value contains no non-finite float or invalid bounded payload.

### A JSON boundary loses type information

That is expected from Natural JSON. Replace `to_json_value()` with
`ValueWireV1` when the receiver must restore `DataType`, unset state, or shape.

## Limitations and best practices

- Keep the feature sets of Wire producers and consumers explicit when concrete
  `chrono`, big-number, URL, or JSON payloads cross a boundary.
- Use `ValueContainer` whenever scalar versus collection is part of the input
  contract; do not infer shape from collection length.
- Use bounded Wire decode entry points for untrusted complete JSON input. For an
  embedded value, account for the complete outer document and share one budget.
- Do not treat `Eq`/`Hash` output as a persistent fingerprint. Hash behavior is
  for in-memory Rust collections and may vary with hasher, platform, versions,
  features, or implementation.
- Do not assume ordinary `Debug` output is redacted. Create a redacted view
  explicitly when the `redact` feature and policy crate are in use.
- Natural JSON is an interoperability projection, not a lossless runtime value
  format.

## Built on `Value`

`Value` is intentionally a reusable value layer rather than a complete key-value
product. Two sibling crates build directly on it:

- [`rs-config`](https://github.com/qubit-ltd/rs-config) is for application
  configuration properties and configuration-oriented reads from sources such
  as files or environment variables.
- [`rs-metadata`](https://github.com/qubit-ltd/rs-metadata) is for typed metadata
  and property values attached to resources or records, including filtering and
  querying scenarios.

Use these crates when you need key management and domain-level operations in
addition to the typed storage and conversion primitives described here.

Their read names have different contracts: `Config::get` converts according to
the reader policy, while `Metadata::get` reads strictly and `Metadata::convert`
requests conversion explicitly. Both reuse `IntoValueDefault` for defaults.

## Further reading

- [README](../README.md)
- [中文 README](../README.zh_CN.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-value)
- [`qubit-datatype` conversion documentation](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
