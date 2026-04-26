# Qubit Value

[![CircleCI](https://circleci.com/gh/qubit-ltd/rs-value.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rs-value)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-value/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-value?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

A type-safe value container framework built on `qubit_common::lang::DataType`,
providing unified abstractions for single values, multi-values, and named values
with generic construction/access/mutation, type conversion, and complete `serde`
serialization support.

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
- **Zero-Cost Abstractions**: Basic types are stored directly; extensive use of
  reference returns to avoid unnecessary copies
- **Named Values**: `NamedValue`/`NamedMultiValues` provide name binding for
  configuration/identification scenarios
- **Serde Support**: All core types implement `Serialize`/`Deserialize`
- **Big Number Support**: Full support for `BigInt` and `BigDecimal` for
  high-precision calculations
- **Extended Types**: Native support for `isize`/`usize`, `Duration`, `Url`,
  `HashMap<String, String>`, and `serde_json::Value`

### 📦 **Core Types**
- **`Value`**: Single value container with `Empty(DataType)` and 28 variants
  covering primitives, strings, date-time, big numbers, platform integers,
  duration, URL, string maps, and JSON
- **`MultiValues`**: Multi-value container corresponding to `Vec<T>` enum
  variants, with `Empty(DataType)`
- **`NamedValue`**: Name-bound `Value` providing `Deref/DerefMut` access to
  inner value
- **`NamedMultiValues`**: Name-bound `MultiValues` with `Deref/DerefMut` and
  `to_named_value()` conversion
- **`ValueError` & `ValueResult<T>`**: Standard error type and result alias

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-value = "0.4.3"
```

## Usage Examples

### Single Value Operations

```rust
use qubit_value::{Value, ValueError};
use qubit_common::lang::DataType;
use num_bigint::BigInt;
use bigdecimal::BigDecimal;
use std::str::FromStr;

// Generic construction and type-inferred retrieval
let v = Value::new(8080i32);
let port: i32 = v.get()?;  // Type inference from variable
assert_eq!(port, 8080);

// Named getter (returns Copy or reference)
assert_eq!(v.get_int32()?, 8080);

// Type inference in function parameters
fn check_port(p: i32) -> bool { p > 1024 }
assert!(check_port(v.get()?));  // Inferred as i32 from function signature

// Cross-type conversion via to<T>()
assert_eq!(v.to::<i64>()?, 8080i64);
assert_eq!(v.to::<String>()?, "8080".to_string());

// Big number with type inference
let big_int = Value::new(BigInt::from(12345678901234567890i64));
let num: BigInt = big_int.get()?;  // Type inference

// Empty value and type management
let mut any = Value::Int32(42);
any.clear();
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
any.set("hello")?;
assert_eq!(any.get_string()?, "hello");
```

### Extended Types

```rust
use qubit_value::Value;
use std::time::Duration;
use url::Url;
use std::collections::HashMap;

// Duration
let v = Value::new(Duration::from_secs(30));
let d: Duration = v.get()?;
assert_eq!(d, Duration::from_secs(30));
// String round-trip: "<nanoseconds>ns"
let s: String = v.to()?;
assert_eq!(s, "30000000000ns");
let v2 = Value::String("30000000000ns".to_string());
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
```

### Multi-Value Operations

```rust
use qubit_value::{MultiValues, ValueError};
use qubit_common::lang::DataType;

// Generic construction
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.count(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// Generic retrieval with type inference (clones Vec)
let nums: Vec<i32> = ports.get()?;

// Get first element
let first: i32 = ports.get_first()?;
assert_eq!(first, 8080);

// Generic add: single / Vec / slice
ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.add(&[8086, 8087][..])?;

// Generic set: replaces entire list
ports.set(vec![9001, 9002])?;
assert_eq!(ports.get_int32s()?, &[9001, 9002]);

// Merge (types must match)
let mut a = MultiValues::Int32(vec![1, 2]);
let b = MultiValues::Int32(vec![3, 4]);
a.merge(&b)?;
assert_eq!(a.get_int32s()?, &[1, 2, 3, 4]);

// Convert to single value (takes first element)
let single = a.to_value();
let first_val: i32 = single.get()?;
assert_eq!(first_val, 1);
```

### Named Value Operations

```rust
use qubit_value::{NamedValue, NamedMultiValues, Value, MultiValues};

// Named single value
let mut nv = NamedValue::new("timeout", Value::new(30i32));
assert_eq!(nv.name(), "timeout");
let timeout: i32 = nv.get()?;
assert_eq!(timeout, 30);

nv.set_name("read_timeout");
nv.set(45i32)?;
assert_eq!(nv.get_int32()?, 45);

// Named multi-value
let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
nmv.add(8082)?;
let first_port: i32 = nmv.get_first()?;
assert_eq!(first_port, 8080);

// Named multi-value → Named single value (takes first element)
let first_named = nmv.to_named_value();
assert_eq!(first_named.name(), "ports");
let val: i32 = first_named.get()?;
assert_eq!(val, 8080);
```

## API Reference

### Generic API

#### Construction
- **Single Value**: `Value::new<T>(t) -> Value`
- **Multi-Value**: `MultiValues::new<T>(Vec<T>) -> MultiValues`

Supported `T` for `new`: `bool`, `char`, `i8`, `i16`, `i32`, `i64`, `i128`,
`u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`, `String`, `&str`,
`NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`, `BigInt`,
`BigDecimal`, `isize`, `usize`, `Duration`, `Url`,
`HashMap<String, String>`, `serde_json::Value`.

#### Retrieval
- **Single Value**: `Value::get<T>(&self) -> ValueResult<T>`
- **Multi-Value**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **First Element**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`

`get<T>()` performs **strict type matching** — the stored variant must be
exactly `T`. For cross-type conversion use `to<T>()` instead.

#### Mutation
- **Single Value**: `Value::set<T>(&mut self, t) -> ValueResult<()>`
- **Multi-Value**:
  - `MultiValues::set<T, S>(&mut self, values: S) -> ValueResult<()>` where
    `S` can be `T`, `Vec<T>`, or `&[T]`
  - `MultiValues::add<T, S>(&mut self, values: S)` supports `T`, `Vec<T>`,
    or `&[T]`

#### Type Conversion
- **`Value::to<T>(&self) -> ValueResult<T>`** — converts to `T` according to
  the rules defined by `ValueConverter<T>`. Supports cross-type conversion
  with range checking where applicable.

**Supported target types and their accepted source variants:**

| Target `T` | Accepted source variants |
|---|---|
| `bool` | `Bool`; integer variants (0=false, non-zero=true); `String` |
| `i8` | `Int8`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `i16` | `Int16`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `i32` | `Int32`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `i64` | `Int64`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `i128` | `Int128`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `isize` | `IntSize`; `Bool`; `Char`; all integer variants; `Float32/64`; `String`; `BigInteger/BigDecimal` |
| `u8` | `UInt8`; `Bool`; `Char`; all integer variants (range checked); `String` |
| `u16` | `UInt8/16/32/64/128`; `Bool`; `Char`; signed integer variants (range checked); `String` |
| `u32` | `UInt8/16/32/64/128`; `Bool`; `Char`; signed integer variants (range checked); `String` |
| `u64` | `UInt8/16/32/64/128`; `Bool`; `Char`; signed integer variants (range checked); `String` |
| `u128` | `UInt8/16/32/64/128`; `Bool`; `Char`; signed integer variants (range checked); `String` |
| `usize` | `UIntSize`; `Bool`; `Char`; all integer variants (range checked); `String` |
| `f32` | `Float32/64`; `Bool`; `Char`; all integer variants; `String`; `BigInteger/BigDecimal` |
| `f64` | `Float64`; `Bool`; `Char`; all numeric variants; `String`; `BigInteger/BigDecimal` |
| `char` | `Char` |
| `String` | all variants (integers/floats/bool/char/date-time/`Duration`/`Url`/`StringMap`/`Json`) |
| `NaiveDate` | `Date` |
| `NaiveTime` | `Time` |
| `NaiveDateTime` | `DateTime` |
| `DateTime<Utc>` | `Instant` |
| `BigInt` | `BigInteger` |
| `BigDecimal` | `BigDecimal` |
| `Duration` | `Duration`; `String` (format: `<nanoseconds>ns`) |
| `Url` | `Url`; `String` |
| `HashMap<String, String>` | `StringMap` |
| `serde_json::Value` | `Json`; `String` (parsed as JSON); `StringMap` |

### Named API

#### Single Value
- **Getters**: `get_xxx()` methods — `get_bool()`, `get_int32()`,
  `get_string()`, `get_duration()`, `get_url()`, `get_string_map()`,
  `get_json()`, etc.
- **Setters**: `set_xxx()` methods — `set_bool()`, `set_int32()`,
  `set_string()`, `set_duration()`, `set_url()`, `set_string_map()`,
  `set_json()`, etc.

#### Multi-Value
- **Getters**: `get_xxxs()` — `get_int32s()`, `get_strings()`,
  `get_durations()`, `get_urls()`, `get_string_maps()`, `get_jsons()`, etc.
- **Setters**: `set_xxxs()` — `set_int32s()`, `set_strings()`, etc.
- **Adders**: `add_xxx()` — `add_int32()`, `add_string()`, `add_duration()`,
  `add_url()`, etc.
- **Slice Operations**: `*_slice` variants — `set_int32s_slice()`,
  `add_strings_slice()`, etc.

### JSON Utilities (on `Value`)
- `Value::from_json_value(serde_json::Value) -> Value`
- `Value::from_serializable<T: Serialize>(value: &T) -> ValueResult<Value>`
- `Value::deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T>`

### Utility Methods

#### Single Value
- `data_type()` — get the data type
- `is_empty()` — check if empty
- `clear()` — clear the value (preserves type)
- `set_type()` — change the type

#### Multi-Value
- `count()` — get element count
- `is_empty()` — check if empty
- `clear()` — clear all values (preserves type)
- `set_type()` — change the type
- `merge()` — merge with another multi-value (types must match)
- `to_value()` — convert to single value (takes first element)

## Error Types

```rust
use qubit_value::{ValueError, ValueResult};
use qubit_common::lang::DataType;

// Main error variants
ValueError::NoValue                          // Empty value accessed
ValueError::TypeMismatch { expected, actual }// get<T>() type mismatch
ValueError::ConversionFailed { from, to }    // to<T>() unsupported direction
ValueError::ConversionError(String)          // to<T>() range/parse failure
ValueError::IndexOutOfBounds { index, len }  // multi-value index error
ValueError::JsonSerializationError(String)   // JSON serialization failure
ValueError::JsonDeserializationError(String) // JSON deserialization failure
```

All operations that may fail return `ValueResult<T> = Result<T, ValueError>`.

## Supported Data Types

### Basic Scalar Types
- **Signed integers**: `i8`, `i16`, `i32`, `i64`, `i128`
- **Unsigned integers**: `u8`, `u16`, `u32`, `u64`, `u128`
- **Platform integers**: `isize`, `usize`
- **Floats**: `f32`, `f64`
- **Other**: `bool`, `char`

### String
- `String` (stored directly)

### Date/Time Types
- `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>` (via `chrono`)

### Big Number Types
- `BigInt`, `BigDecimal` (via `num-bigint` and `bigdecimal`)

### Extended Types
- **`isize` / `usize`**: Platform-dependent integers
- **`Duration`**: `std::time::Duration`; string representation `<ns>ns`
- **`Url`**: `url::Url`; string representation is the URL text
- **`HashMap<String, String>`**: String map; string representation is JSON
- **`serde_json::Value`**: JSON escape hatch for complex/custom types

## Serialization Support

All types implement `Serialize`/`Deserialize`:
- `Value`, `MultiValues`, `NamedValue`, `NamedMultiValues`

Full type information is preserved during serialization and validated during
deserialization.

## Performance Notes

- **Reference Returns**: `get_string()` returns `&str` to avoid cloning
- **Borrow Support**: `Value::new()` and `set()` accept `&str` (converted to
  `String`)
- **Smart Dispatch**: `MultiValues::set/add` accept `T`, `Vec<T>`, or `&[T]`
  and dispatch to the optimal internal path

## Dependencies

```toml
[dependencies]
qubit-common = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
url = { version = "2.5", features = ["serde"] }
num-traits = "0.2"
num-bigint = { version = "0.4", features = ["serde"] }
bigdecimal = { version = "0.4", features = ["serde"] }
```

## Testing & Code Coverage

This project maintains comprehensive test coverage with detailed validation of
all functionality. For coverage reports and testing instructions, see the
[COVERAGE.md](COVERAGE.md) documentation.

## License

Copyright (c) 2025 - 2026 Haixing Hu, Qubit Co. Ltd. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

---

For more information about Qubit open source projects, visit our
[GitHub organization](https://github.com/qubit-ltd).
