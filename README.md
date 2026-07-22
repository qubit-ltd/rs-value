# Qubit Value

[![Rust CI](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-value/coverage-badge.json)](https://qubit-ltd.github.io/rs-value/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

A type-safe value container framework built on `qubit_datatype::DataType`.
It provides single-value, homogeneous multi-value, and named wrappers with
strict access, generic mutation, option-controlled conversion, and tagged
Serde representations.

## Overview

Qubit Value provides a comprehensive solution for handling dynamically-typed
values in a type-safe manner. It bridges the gap between static typing and
runtime flexibility, offering powerful abstractions for value storage, retrieval,
and conversion while maintaining Rust's safety guarantees.

> **Configuration Object Support**: If you need configuration objects based on
> different types of multi-value designs, consider using the
> [qubit-config](https://github.com/qubit-ltd/rs-config) crate, which provides
> comprehensive configuration management functionality. You can find more
> information on [GitHub](https://github.com/qubit-ltd/rs-config) and
> [crates.io](https://crates.io/crates/qubit-config).

## Features

### 🎯 **Core Design**
- **Enum-Based Architecture**: Uses `Value`/`MultiValues` enums to represent all
  supported data types
- **Type Safety**: Enum variants carry static types; failures are expressed
  through `Result<T, ValueError>`
- **Borrowed Access**: Typed getters return references where the stored type is
  not `Copy`
- **Named Values**: `NamedValue`/`NamedMultiValues` provide name binding for
  configuration/identification scenarios
- **Two JSON Boundaries**: Tagged Serde preserves data types; natural JSON
  projection produces ordinary `null`, scalar, object, and array values
- **Ergonomic Defaults**: `get_or`, `to_or`, and list-default APIs accept
  scalar defaults, borrowed string literals, arrays, slices, vectors, and
  borrowed vectors
- **Flexible Collection Inputs**: `MultiValues::new/set/add` accept direct
  arrays, slices, vectors, borrowed vectors, and borrowed string collections
- **Big Number Support**: Optional `BigInt` and `BigDecimal` variants
- **Extended Types**: Native support for `Duration`, `Url`,
  `HashMap<String, String>`, and `serde_json::Value`

### 📦 **Core Types**
- **`Value`**: Single value container with `Unset(DataType)` and up to 25
  platform-independent concrete variants, depending on enabled features
- **`MultiValues`**: Multi-value container corresponding to `Vec<T>` enum
  variants, with `Unset(DataType)`
- **`ValueContainer`**: Explicit `Scalar(Value)` or
  `Collection(MultiValues)` shape without length-based inference
- **`NamedValue`**: Name-bound `Value` with explicit `value()` and
  `value_mut()` access to the inner value
- **`NamedMultiValues`**: Name-bound `MultiValues` with borrowed
  `first_named_value()` and consuming `into_first_named_value()` conversions
- **`ValueError` & `ValueResult<T>`**: Standard error type and result alias

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

Unset values are always typed. Prefer `Value::new_unset(data_type)` and
`MultiValues::new_unset(data_type)`; neither container implements `Default`.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-value = "0.10"
```

The default feature set is empty. Enable only the required families, or use
`all` as the convenience feature:

| Feature | Enables |
|---|---|
| `chrono` | Date, time, date-time, and UTC instant variants |
| `big-integer` | `BigInt` variants |
| `big-decimal` | `BigDecimal` variants |
| `big-number` | Compatibility alias for `big-integer` and `big-decimal` |
| `url` | URL variants |
| `json` | `serde_json::Value` variants |
| `converter` | Core conversion APIs without enabling rich type families |
| `all` | `converter`, `chrono`, `big-number`, `url`, and `json` |

Natural JSON projection requires both `converter` and `json`. For example,
use `features = ["converter", "json"]` without enabling other rich families.

The basic examples below use only the default feature set. Conversion examples
need `converter`; examples using `Url`, JSON, or Serde additionally need the
owning crates as direct dependencies. For the complete extended example:

```toml
[dependencies]
qubit-value = { version = "0.10", features = ["all"] }
qubit-datatype = { version = "0.8", default-features = false }
url = "2.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

The feature set is part of the runtime value contract. A V1 wire envelope keeps
its versioned object structure across builds, but a concrete rich-type payload
such as `date`, `biginteger`, `bigdecimal`, `url`, or `json` can be read only by
a build with the corresponding feature enabled. Producers and consumers that
exchange those payloads should therefore agree on the required type features.
An unset payload such as `{"unset":"date"}` preserves a declared
`DataType`, but does not make the concrete date value available in a build
without `chrono`.

## Usage Examples

### Single Value Operations

```rust
use qubit_datatype::DataType;
use qubit_value::Value;

# fn main() -> Result<(), qubit_value::ValueError> {

// Generic construction and type-inferred retrieval
let v = Value::new(8080i32);
let port: i32 = v.get()?;  // Type inference from variable
assert_eq!(port, 8080);

// Named getter (returns Copy or reference)
assert_eq!(v.get_int32()?, 8080);

// Type inference in function parameters
fn check_port(p: i32) -> bool { p > 1024 }
assert!(check_port(v.get()?));  // Inferred as i32 from function signature

// Empty value and type management
let mut any = Value::Int32(42);
any.clear();
assert!(any.is_unset());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
any.set("hello");
assert_eq!(any.get_string()?, "hello");
# Ok(())
# }
```

### Extended Types

```rust
use qubit_value::Value;
use std::time::Duration;
use url::Url;
use std::collections::HashMap;

# fn main() -> Result<(), Box<dyn std::error::Error>> {

// Duration
let v = Value::new(Duration::from_secs(30));
let d: Duration = v.get()?;
assert_eq!(d, Duration::from_secs(30));
// Default String conversion uses milliseconds.
let s: String = v.to()?;
assert_eq!(s, "30000ms");
let v2 = Value::String("30s".to_string());
let d2: Duration = v2.to()?;
assert_eq!(d2, Duration::from_secs(30));

// Url
let url = Url::parse("https://example.com").unwrap();
let v = Value::new(url.clone());
let got: Url = v.get()?;
assert_eq!(got, url);
// Parse from string
let v2 = Value::String("https://example.com".to_string());
let got2: Url = v2.to()?;
assert_eq!(got2, url);

// HashMap<String, String>
let mut map = HashMap::new();
map.insert("host".to_string(), "localhost".to_string());
let v = Value::new(map.clone());
let got: HashMap<String, String> = v.get()?;
assert_eq!(got, map);

// JSON escape hatch
let j = serde_json::json!({"key": "value"});
let v = Value::from_json_value(j.clone());
let got: serde_json::Value = v.get()?;
assert_eq!(got, j);

// Serialize any type to JSON
#[derive(serde::Serialize, serde::Deserialize)]
struct Config { host: String, port: u16 }
let cfg = Config { host: "localhost".to_string(), port: 8080 };
let v = Value::from_serializable(&cfg)?;
let restored: Config = v.deserialize_json()?;
# Ok(())
# }
```

### Multi-Value Operations

```rust
use qubit_value::{MultiValues, ValueError};
use qubit_datatype::DataType;

// Generic construction from a Vec<T>
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.len(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// Direct arrays, slices, vectors, and borrowed vectors are accepted
let array_ports = MultiValues::new([8080i32, 8081, 8082]);
let more_ports = [9000i32, 9001];
let borrowed = MultiValues::new(more_ports.as_slice());
let owned = vec![7000i32, 7001];
let borrowed_vec = MultiValues::new(&owned);

// String lists can be built directly from &str collections
let servers = MultiValues::new(["api", "worker", "cache"]);
assert_eq!(servers.get_strings()?, &["api", "worker", "cache"]);

// Generic retrieval with type inference (clones Vec)
let nums: Vec<i32> = ports.get()?;

// Get first element
let first: i32 = ports.get_first()?;
assert_eq!(first, 8080);

// Generic add: single / Vec / slice
ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.add(&[8086, 8087][..])?;
ports.add([8088, 8089])?;

// Generic set: replaces entire list
ports.set(vec![9001, 9002]);
ports.set([9100, 9101]);
ports.set(&owned);
assert_eq!(ports.get_int32s()?, &[7000, 7001]);

// Merge (types must match)
let mut a = MultiValues::Int32(vec![1, 2]);
let b = MultiValues::Int32(vec![3, 4]);
a.merge(&b)?;
assert_eq!(a.get_int32s()?, &[1, 2, 3, 4]);

// Convert to single value (takes first element)
let single = a.first_value();
let first_val: i32 = single.get()?;
assert_eq!(first_val, 1);
```

### Defaulted Reads and Conversions

Defaulted APIs use the fallback only when the container is unset. A concrete
empty `MultiValues` vector remains an empty result; it does not trigger the
fallback. Type mismatches and failed conversions still return errors.

```rust
use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value};

// Strict reads with defaults
let value = Value::Unset(DataType::String);
let host: String = value.get_or("localhost")?;
assert_eq!(host, "localhost");

let value = Value::String("8080".to_string());
let port: u16 = value.to_or(9000u16)?;
assert_eq!(port, 8080);

// Multi-value strict reads with collection defaults
let values = MultiValues::Unset(DataType::String);
let paths: Vec<String> = values.get_or(["cache", "tmp"])?;
assert_eq!(paths, vec!["cache".to_string(), "tmp".to_string()]);

// First-value conversion with a scalar default
let values = MultiValues::Unset(DataType::UInt16);
let port: u16 = values.to_or(8080u16)?;
assert_eq!(port, 8080);

// List conversion with array or slice defaults
let values = MultiValues::Unset(DataType::String);
let tags: Vec<String> = values.to_list_or(["blue", "green"])?;
assert_eq!(tags, vec!["blue".to_string(), "green".to_string()]);
```

### Collection Argument Forms

The collection-style APIs accept the convenient forms you normally have at the
call site. This applies to `MultiValues::new`, `MultiValues::set`,
`MultiValues::add`, and defaulted list reads such as `get_or` and
`to_list_or`.

```rust
use qubit_datatype::DataType;
use qubit_value::MultiValues;

let array_values = MultiValues::new([1i32, 2, 3]);
let slice_source = [4i32, 5, 6];
let slice_values = MultiValues::new(slice_source.as_slice());
let vec_source = vec![7i32, 8, 9];
let vec_values = MultiValues::new(vec_source.clone());
let borrowed_vec_values = MultiValues::new(&vec_source);

let mut values = MultiValues::Unset(DataType::Int32);
values.set([10, 11, 12]);
values.add(slice_source.as_slice())?;
values.add(&vec_source)?;

let strings = MultiValues::new(["api", "worker"]);
let fallback: Vec<String> = MultiValues::Unset(DataType::String)
    .get_or(["cache", "tmp"])?;
```

### Named Value Operations

```rust
use qubit_value::{NamedValue, NamedMultiValues, Value, MultiValues};

// Named single value
let mut nv = NamedValue::new("timeout", Value::new(30i32));
assert_eq!(nv.name(), "timeout");
let timeout: i32 = nv.value().get()?;
assert_eq!(timeout, 30);

nv.set_name("read_timeout");
nv.value_mut().set(45i32);
assert_eq!(nv.value().get_int32()?, 45);

// Named multi-value
let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
nmv.values_mut().add(8082)?;
let first_port: i32 = nmv.values().get_first()?;
assert_eq!(first_port, 8080);

// Named multi-value → Named single value (takes first element)
let first_named = nmv.first_named_value();
assert_eq!(first_named.name(), "ports");
let val: i32 = first_named.value().get()?;
assert_eq!(val, 8080);
```

## API Reference

### Generic API

#### Construction
- **Single Value**: `Value::new<T>(t) -> Value`
- **Multi-Value**: `MultiValues::new<S>(values) -> MultiValues`

`MultiValues::new` accepts `Vec<T>`, `&Vec<T>`, `&[T]`, `[T; N]`, and
`&[T; N]`. For string values it also accepts `Vec<&str>`, `&Vec<&str>`,
`&[&str]`, `[&str; N]`, and `&[&str; N]`, producing `Vec<String>` internally.

Supported `T` for `new`: `bool`, `char`, `i8`, `i16`, `i32`, `i64`, `i128`,
`u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`, `String`, `&str`,
`NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`, `BigInt`,
`BigDecimal`, `Duration`, `Url`,
`HashMap<String, String>`, `serde_json::Value`.

#### Retrieval
- **Single Value**: `Value::get<T>(&self) -> ValueResult<T>`
- **Single Value with Default**: `Value::get_or<T>(&self, default) -> ValueResult<T>`
- **Multi-Value**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **Multi-Value with Default**: `MultiValues::get_or<T>(&self, default) -> ValueResult<Vec<T>>`
- **First Element**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`
- **First Element with Default**: `MultiValues::get_first_or<T>(&self, default) -> ValueResult<T>`

`get<T>()` performs **strict type matching** — the stored variant must be
exactly `T`. For cross-type conversion use `to<T>()` instead.

#### Mutation
- **Single Value**: `Value::set<T: Into<Value>>(&mut self, value) -> ()`
- **Multi-Value**:
  - `MultiValues::set<S: Into<MultiValues>>(&mut self, values) -> ()`
    replaces the entire collection and may change its type
  - `MultiValues::add<S: Into<MultiValues>>(&mut self, values) -> ValueResult<()>`
    appends only when the element type matches
  - Both accept scalar, `Vec<T>`, `&Vec<T>`, `&[T]`, `[T; N]`, and
    `&[T; N]` forms supported by `Into<MultiValues>`
  - String collections also accept `Vec<&str>`, `&Vec<&str>`, `&[&str]`,
    `[&str; N]`, and `&[&str; N]`

#### Type Conversion
- **`Value::to<T>(&self) -> ValueResult<T>`** — converts to `T` according to
  the shared conversion rules. Supports cross-type conversion with range
  checking where applicable.
- **`Value::to_or<T>(&self, default) -> ValueResult<T>`** — converts to `T`,
  or returns the default when the value is unset.
- **`Value::to_or_with<T>(&self, default, options) -> ValueResult<T>`** —
  same fallback behavior while using explicit conversion options.
- **`MultiValues::to<T>(&self) -> ValueResult<T>`** — converts the first stored
  value.
- **`MultiValues::to_or<T>(&self, default) -> ValueResult<T>`** — converts the
  first stored value, or returns the default only when the container is unset.
- **`MultiValues::to_or_with<T>(&self, default, options) -> ValueResult<T>`** —
  same fallback behavior while using explicit conversion options.
- **`MultiValues::to_list<T>(&self) -> ValueResult<Vec<T>>`** — converts all
  stored values.
- **`MultiValues::to_list_with<T>(&self, options) -> ValueResult<Vec<T>>`** —
  converts all stored values with explicit conversion options.
- **`MultiValues::to_list_or<T>(&self, default) -> ValueResult<Vec<T>>`** —
  converts all stored values, or returns the default when the container is
  unset. A concrete empty vector stays empty.
- **`MultiValues::to_list_or_with<T>(&self, default, options) -> ValueResult<Vec<T>>`** —
  same list fallback behavior while using explicit conversion options.

The complete source/target matrix, range rules, parsing behavior, and option
semantics are defined by the authoritative
[`qubit-datatype` conversion contract](https://docs.rs/qubit-datatype/latest/qubit_datatype/).
This crate only adds container semantics: strict `get`, first-item collection
conversion, unset error mapping, and indexed list errors.

### Typed and Named API

#### Single Value
- **Getters**: `get_xxx()` methods — `get_bool()`, `get_int32()`,
  `get_string()`, `get_duration()`, `get_url()`, `get_string_map()`,
  `get_json()`, etc.
- **Mutation**: use the generic `set()` method. Typed setters were removed
  because they added no behavior beyond the generic API.

#### Multi-Value
- **Getters**: `get_xxxs()` — `get_int32s()`, `get_strings()`,
  `get_durations()`, `get_urls()`, `get_string_maps()`, `get_jsons()`, etc.
- **Mutation**: use generic `set()` and `add()` for scalar, owned collection,
  array, slice, and borrowed-vector inputs.

### JSON Utilities
- `Value::from_json_value(serde_json::Value) -> Value`
- `Value::from_serializable<T: Serialize>(value: &T) -> ValueResult<Value>`
- `Value::deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T>`
- `Value::to_json_value(&self) -> ValueResult<serde_json::Value>`
- `MultiValues::to_json_value(&self) -> ValueResult<serde_json::Value>`
- `ValueContainer::to_json_value(&self) -> ValueResult<serde_json::Value>`

Tagged Serde and natural JSON are separate contracts. Tagged Serde preserves
the variant name, while natural JSON maps unset to `null`; every concrete
collection remains an array, including one-item collections. Natural JSON
represents 128-bit and big numbers as strings.
Non-finite floats may be stored in memory, but both JSON-facing contracts reject
`NaN`, positive infinity, and negative infinity because JSON defines no such
number literals.

### Utility Methods

#### Single Value
- `data_type()` — get the data type
- `is_unset()` — check whether no concrete value is stored
- `is_numeric()` — classify a concrete numeric value
- `is_nan()` — test whether the concrete value is a floating-point NaN
- `unset()` / `clear()` — remove the value while preserving its declared type
- `set_type()` — change the type

#### Multi-Value
- `len()` — get element count
- `is_unset()` — distinguish unset from a concrete empty vector
- `is_numeric()` — classify a concrete numeric collection
- `unset()` — remove the concrete vector while preserving its declared type
- `clear()` — clear a concrete vector while preserving its concrete state;
  unset remains unset
- `set_type()` — change the type
- `merge()` — merge with another multi-value (types must match)
- `first_value()` / `into_first_value()` — convert the first element to a
  single value

## Error Types

```rust
use qubit_value::{ValueError, ValueResult};
use qubit_datatype::DataType;

// Main error variants
ValueError::NoValue                          // Unset value accessed
ValueError::TypeMismatch { expected, actual }// get<T>() type mismatch
ValueError::DataConversion(DataConversionError) // structured to<T>() failure
ValueError::DataListConversion(DataListConversionError) // indexed list failure
```

All operations that may fail return `ValueResult<T> = Result<T, ValueError>`.
Conversion errors preserve the shared structured source error; list errors also
preserve the original `source_index`. `to()` uses the strict conversion profile
by default. Use `to_with()` and `DataConversionOptions::lossy()` when all
documented lossy behavior is intentional, or replace only the required
`NumericConversionOptions` policy. Numeric text and `BigInt` resource caps are
configurable through `NumericConversionLimits`. Text is not trimmed unless
enabled in `StringConversionOptions`.

`Value`, `MultiValues`, and `ValueError` are non-exhaustive public enums.
Downstream `match` expressions must keep a wildcard arm so future variants do
not break source compatibility.

## Supported Data Types

### Basic Scalar Types
- **Signed integers**: `i8`, `i16`, `i32`, `i64`, `i128`
- **Unsigned integers**: `u8`, `u16`, `u32`, `u64`, `u128`
- **Floats**: `f32`, `f64`
- **Other**: `bool`, `char`

### String
- `String` (stored directly)

### Date/Time Types
- `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>` (via `chrono`)

### Big Number Types
- `BigInt`, `BigDecimal` (via `num-bigint` and `bigdecimal`)

### Extended Types
- **`Duration`**: `std::time::Duration`; string conversion uses the
  configured duration unit, defaulting to milliseconds such as `1500ms`.
  Parsing accepts `ns`, `us`, `ms`, `s`, `m`, `h`, and `d` suffixes; strings
  without a suffix are interpreted using the configured duration unit.
- **`Url`**: `url::Url`; string representation is the URL text
- **`HashMap<String, String>`**: String map; string representation is JSON
- **`serde_json::Value`**: JSON escape hatch for complex/custom types

## Serialization Contracts

Enabled types implement `Serialize`/`Deserialize`:

- `Value`, `MultiValues`, `ValueContainer`, `NamedValue`, `NamedMultiValues`
- `ValueWireV1`, the public version-one wire DTO

Type-preserving Serde uses one strict versioned envelope:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

The V1 compatibility guarantee applies to this JSON object structure. The
Serde implementations may be used with other serializers, but their
format-specific representation is not part of the V1 stability contract.
V1 is closed: its existing tags, shapes, and payload representations cannot
change, and a future runtime data type requires a new wire version instead of
extending V1.

Collections use `collection` instead of `scalar`; an unset payload uses
`{"unset":"int32"}`. `Value` accepts only scalar, `MultiValues` accepts only
collection, and `ValueContainer` accepts both shapes. `Value` and
`ValueContainer::Scalar` have identical wire, as do `MultiValues` and
`ValueContainer::Collection`. Named wrappers keep their outer `name`/`value`
fields and place this envelope in `value`.

Version 0.10 intentionally rejects former externally tagged forms such as
`{"Int32":42}`, `{"Unset":"int32"}`, and `{"Scalar":{"Int32":42}}`. It also
rejects missing or unknown fields, versions other than numeric `1`, unknown
shapes and types, and mismatched runtime entry shapes.

`Int128`, `UInt128`, and `BigInteger` payloads use canonical decimal strings.
`BigDecimal` uses an exact `{"coefficient":"...","scale":i64}` payload.
`Duration` uses `{"secs":u64,"nanos":u32}` and requires nanos below one
second. Float payloads must be finite.

Generic Serde deserialization does not impose an external message-size budget.
For untrusted JSON, use `ValueWireV1::decode_json_slice()` to enforce the
default one-mebibyte limit before parsing, or select a protocol-specific budget
with `ValueWireLimits`:

```rust
use qubit_value::{ValueWireLimits, ValueWireV1};

let input = br#"{"version":1,"value":{"scalar":{"int32":42}}}"#;
let limits = ValueWireLimits::new(64 * 1024);
let wire = ValueWireV1::decode_json_slice_with_limits(input, limits)?;
assert!(wire.container().is_scalar());
# Ok::<(), qubit_value::ValueWireDecodeError>(())
```

These decode methods accept a complete top-level `ValueWireV1` document. When
`Value`, `MultiValues`, or `ValueContainer` is embedded in an outer JSON
document, call `ValueWireLimits::check_json_bytes()` with the complete outer
input length before invoking that document's Serde decoder.

This type-preserving V1 wire is separate from `to_json_value()`, which emits
natural JSON without runtime type tags and projects unset values to `null`.
Duration projection is exact by default; use `to_json_value_with()` and lossy
conversion options when unit rounding is intentional.

For the full wire-format rationale and feature-availability details, see the
[user guide](doc/user_guide.md) and its [Chinese translation](doc/user_guide.zh_CN.md).

## Performance Notes

- **Reference Returns**: `get_string()` returns `&str` to avoid cloning
- **Borrow Support**: `Value::new()` and `set()` accept `&str` (converted to
  `String`)
- **Flexible Inputs**: `MultiValues::new/set/add` accept direct arrays, slices,
  vectors, and borrowed vectors for supported element types
- **Borrowed Defaults**: Defaulted reads can use borrowed string literals and
  borrowed collection values without forcing callers to allocate first

## Dependencies

```toml
[dependencies]
qubit-datatype = { version = "0.8", default-features = false }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
serde_json = { version = "1.0", optional = true }
chrono = { version = "0.4", features = ["serde"], optional = true }
url = { version = "2.5", features = ["serde"], optional = true }
num-bigint = { version = "0.4", optional = true }
bigdecimal = { version = "0.4", optional = true }
```

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-value](https://github.com/qubit-ltd/rs-value)
