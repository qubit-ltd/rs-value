# qubit-value User Guide

[中文版本](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-value)

## Purpose and audience

This guide is for Rust developers who receive configuration, metadata, protocol
fields, or other values whose concrete type is known at runtime. It explains
how to keep those values type-safe without forcing every caller to create a
different ad-hoc enum.

The guide covers `qubit-value` 0.10. It focuses on the value layer. It does not
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

The core dependency is:

```toml
[dependencies]
qubit-value = { version = "0.10", features = ["all"] }
qubit-datatype = { version = "0.11", default-features = false }
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
| `json` | `Json` and Wire JSON decoding/resource limits; Natural JSON also requires `converter` |
| `redact` | `Value` redacted views; the application also imports `Redact` from `qubit-redact` |
| `all` | `converter`, `chrono`, `big-number`, `url`, `json`, and `redact` |

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

### Convert with explicit policy

With `converter`, `to` applies the shared `qubit-datatype` conversion contract.
Use `to_with` when the default strict policy is not the policy the application
wants.

```rust
use qubit_value::Value;

let text = Value::new("42".to_owned());
let number: u32 = text.to()?;
assert_eq!(number, 42);

let fallback: u16 = Value::new_unset(qubit_datatype::DataType::UInt16)
    .to_or(8080u16)?;
assert_eq!(fallback, 8080);
```

`to_or` can also use a conversion policy's missing-value result, such as a
configured blank-as-missing behavior. It does not hide an ordinary type
mismatch or invalid conversion. The full source/target matrix and policy
details live in the [`qubit-datatype` API documentation](https://docs.rs/qubit-datatype/latest/qubit_datatype/).

### Preserve names without changing value semantics

```rust
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value};

let mut timeout = NamedValue::new("timeout", Value::new(30u64));
assert_eq!(timeout.name(), "timeout");
assert_eq!(timeout.value().get()?, 30);
timeout.value_mut().set(45u64);

let mut ports = NamedMultiValues::new("ports", MultiValues::new([8080u16, 8081]));
ports.values_mut().add(8082u16)?;
let first_port: u16 = ports
    .values()
    .get_first()
    ?;
assert_eq!(first_port, 8080);
```

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

V1 is closed. Existing tags, shapes, and payload representations cannot be
extended in place; a future runtime type requires a new wire version. String-map
keys and nested JSON object keys are emitted in lexicographic order under the
supported canonical JSON configuration.

### End-to-end: `ValueContainer` to JSON and back

The following example creates an explicitly scalar value, converts it into the
owned Wire DTO, serializes it, applies input and semantic limits during decode,
and restores the original container.

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonEncodeLimits, JsonResource, JsonValueLimits};
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;

let original = ValueContainer::Scalar(Value::new(8080i32));
let wire = ValueWireV1::try_from(original.clone())?;
let structure = StructureLimits::<StructureResource, usize>::builder()
    .depth_limit(ResourceLimit::new(JsonResource::Depth, 32))
    .nodes_limit(ResourceLimit::new(JsonResource::Nodes, 128))
    .build();
let values = JsonValueLimits::<JsonResource, usize>::builder()
    .structure_limits(structure)
    .build();
let encode_limits = JsonEncodeLimits::<JsonResource, usize>::builder()
    .output_bytes_limit(ResourceLimit::new(
        JsonResource::OutputBytes,
        64 * 1024,
    ))
    .value_limits(values)
    .build();
let encoded = wire.to_json_vec_with_limits(encode_limits)?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"int32":8080}}}"#
);

let decode_limits = JsonDecodeLimits::<JsonResource, usize>::builder()
    .input_bytes_limit(ResourceLimit::new(
        JsonResource::InputBytes,
        64 * 1024,
    ))
    .value_limits(values)
    .build();
let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits)?;
let restored: ValueContainer = decoded.into();

assert_eq!(restored, original);
assert!(restored.is_scalar());
assert_eq!(restored.data_type(), qubit_datatype::DataType::Int32);
```

The decode helpers accept a complete top-level Wire document and use the
generic `qubit-budget` JSON/Serde adapter.
`ValueWireV1::default_json_decode_limits()` and
`default_json_encode_limits()` provide the directional V1 profiles. Pass a
`JsonDecodeLimits` or `JsonEncodeLimits` value when the application owns a
different input, output, or value budget.

### Borrowed Wire encoding

Use a borrowed adapter when the source value already lives long enough for the
serialization call and cloning would be unnecessary.

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
constructors are fallible because they
validate finite floats, bounded `BigDecimal` scale, and reserved JSON object
keys before exposing a serializable payload.

### Embedded values and a shared `JsonDecodeSession`

`decode_json_slice_with_limits` is for a complete top-level Wire document. If a
value is nested inside a larger JSON document, use the shared `qubit-budget`
Serde adapter for the complete outer document so every JSON node is charged in
one session.

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession, JsonResource};
use qubit_json::decode::JsonDecoder;
use qubit_budget::ResourceLimit;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;
use serde::Deserialize;

#[derive(Deserialize)]
struct Request {
    value: ValueWireV1,
}

let input = br#"{"value":{"version":1,"value":{"collection":{"int32":[1,2]}}}}"#;
let limits = JsonDecodeLimits::<JsonResource, usize>::builder()
    .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 64 * 1024))
    .build();
let mut session = JsonDecodeSession::new(limits);
let request: Request = JsonDecoder::new(&mut session).decode(input)?;
let restored: ValueContainer = request.value.into();
assert!(restored.is_collection());
```

The outer object and embedded V1 envelope are both part of the same generic JSON
document budget. Reusing one `JsonDecodeSession` accumulates usage across multiple
embedded values and rejects trailing content after the complete document.

### Wire-specific type and input boundaries

- `Int128`, `UInt128`, and `BigInteger` use canonical decimal strings rather
  than JSON numbers that might lose precision.
- `BigDecimal` uses an exact coefficient/scale payload and rejects a scale whose
  absolute value exceeds the V1 bound.
- `Duration` uses a `{ "secs": ..., "nanos": ... }` payload with nanos below
  one second.
- `Float32` and `Float64` may contain non-finite values in memory, but V1
  rejects NaN and infinities because JSON has no such number literals.
- `Json(null)` is a concrete JSON value and is distinct from `Unset(Json)`.
- A concrete rich type can be decoded only by a build with its corresponding
  feature. Unsupported feature-gated payloads are rejected rather than guessed.
- Unknown fields, unknown types, wrong scalar/collection shapes, non-numeric
  versions, and pre-0.10 externally tagged documents are rejected.

## Natural JSON

With `converter` and `json`, Natural JSON projects a runtime value into ordinary
`serde_json::Value`. The following example shows the exact JSON string emitted
for several common values:

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

## Troubleshooting

### `get<T>()` returns a type mismatch

Inspect `value.data_type()` and use a typed getter when the source type must be
exact. If conversion is intended, enable `converter` and call `to<T>()` with a
policy that matches the application.

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

## Further reading

- [README](../README.md)
- [中文 README](../README.zh_CN.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-value)
- [`qubit-datatype` conversion documentation](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
