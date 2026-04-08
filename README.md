# Qubit Value

[![CircleCI](https://circleci.com/gh/qubit-ltd/rust-value.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rust-value)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rust-value/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rust-value?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

A type-safe value container framework built on `qubit_common::lang::DataType`, providing unified abstractions for single
values, multi-values, and named values with generic construction/access/mutation, type conversion, and complete `serde`
serialization support.

## Overview

Qubit Value provides a comprehensive solution for handling dynamically-typed values in a type-safe manner. It bridges
the gap between static typing and runtime flexibility, offering powerful abstractions for value storage, retrieval, and
conversion while maintaining Rust's safety guarantees.

> **Configuration Object Support**: If you need configuration objects based on different types of multi-value designs,
 consider using the [qubit-config](https://github.com/qubit-ltd/rust-config) crate, which provides comprehensive
 configuration management functionality. You can find more information on [GitHub](https://github.com/qubit-ltd/rust-config)
 and [crates.io](https://crates.io/crates/qubit-config).

## Features

### 🎯 **Core Design**
- **Enum-Based Architecture**: Uses `Value`/`MultiValues` enums to cover the Java interface+implementation hierarchy
- **Type Safety**: Enum variants carry static types; failures are expressed through `Result<T, ValueError>`
- **Zero-Cost Abstractions**: Basic types are stored directly; extensive use of reference returns to avoid unnecessary copies
- **Named Values**: `NamedValue`/`NamedMultiValues` provide name binding for configuration/identification scenarios
- **Serde Support**: All core types implement `Serialize`/`Deserialize`
- **Big Number Support**: Full support for `BigInt` and `BigDecimal` for high-precision calculations

### 📦 **Core Types**
- **`Value`**: Single value container with `Empty(DataType)` and 20+ variants for primitives, strings, bytes, date-time, big numbers, etc.
- **`MultiValues`**: Multi-value container corresponding to `Vec<T>` enum variants, with `Empty(DataType)`
- **`NamedValue`**: Name-bound `Value` providing `Deref/DerefMut` access to inner value
- **`NamedMultiValues`**: Name-bound `MultiValues` with `Deref/DerefMut` and `to_named_value()` conversion
- **`ValueError` & `ValueResult<T>`**: Standard error type and result alias

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-value = "0.2.0"
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

// Cross-type conversion
assert_eq!(v.to::<i64>()?, 8080i64);
assert_eq!(v.to::<String>()?, "8080".to_string());

// Big number with type inference
let big_int = Value::new(BigInt::from(12345678901234567890i64));
let num: &BigInt = big_int.get()?;  // Type inference
assert_eq!(num, &BigInt::from(12345678901234567890i64));

let big_decimal = Value::new(BigDecimal::from_str("123.456")?);
let decimal: &BigDecimal = big_decimal.get()?;  // Type inference
assert_eq!(decimal, &BigDecimal::from_str("123.456")?);

// Empty value and type management
let mut any = Value::Int32(42);
any.clear();
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::String);

// Generic setter with type inference (dispatches to named setter)
any.set("hello")?; // &str -> String, type inferred from argument
assert_eq!(any.get_string()?, "hello");
```

### Multi-Value Operations

```rust
use qubit_value::{MultiValues, ValueError};
use qubit_common::lang::DataType;
use num_bigint::BigInt;
use bigdecimal::BigDecimal;

// Generic construction
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.count(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// Generic retrieval with type inference (clones Vec)
let nums: Vec<i32> = ports.get()?;  // Type inferred from variable
assert_eq!(nums, vec![8080, 8081, 8082]);

// Type inference in iteration
for port in ports.get::<i32>()? {  // Returns Vec<i32>
    println!("Port: {}", port);
}

// Get first element with type inference
let first: i32 = ports.get_first()?;  // Type inferred from variable
assert_eq!(first, 8080);

// Generic add with type inference: single/Vec/slice
ports.add(8083)?;                    // Type inferred from argument
ports.add(vec![8084, 8085])?;        // Type inferred from Vec<i32>
ports.add(&[8086, 8087][..])?;       // Type inferred from slice
assert_eq!(ports.get_int32s()?, &[8080,8081,8082,8083,8084,8085,8086,8087]);

// Generic set with type inference: single/Vec/slice, replaces entire list
ports.set(9000)?;                    // Type inferred from argument
ports.set(vec![9001, 9002])?;        // Type inferred from Vec<i32>
ports.set(&[9003, 9004][..])?;       // Type inferred from slice
assert_eq!(ports.get_int32s()?, &[9003, 9004]);

// Big number multi-value with type inference
let mut big_nums = MultiValues::new(vec![
    BigInt::from(123456789),
    BigInt::from(987654321)
]);
assert_eq!(big_nums.count(), 2);
big_nums.add(BigInt::from(111111111))?;  // Type inferred from argument
let nums_vec: Vec<BigInt> = big_nums.get()?;  // Type inference
assert_eq!(nums_vec.len(), 3);

// Merge (types must match)
let mut a = MultiValues::Int32(vec![1,2]);
let b = MultiValues::Int32(vec![3,4]);
a.merge(&b)?;
assert_eq!(a.get_int32s()?, &[1,2,3,4]);

// Convert to single value (takes first element)
let single = a.to_value();
let first_val: i32 = single.get()?;  // Type inference
assert_eq!(first_val, 1);
```

### Named Value Operations

```rust
use qubit_value::{NamedValue, NamedMultiValues, Value, MultiValues};
use num_bigint::BigInt;

// Named single value with type inference
let mut nv = NamedValue::new("timeout", Value::new(30i32));
assert_eq!(nv.name(), "timeout");
let timeout: i32 = nv.get()?;  // Type inferred from variable
assert_eq!(timeout, 30);

// Generic setter with type inference
nv.set_name("read_timeout");
nv.set(45i32)?;  // Type inferred from argument
assert_eq!(nv.get_int32()?, 45);

// Named multi-value with type inference
let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
assert_eq!(nmv.name(), "ports");
nmv.add(8082)?;  // Type inferred from existing values
let first_port: i32 = nmv.get_first()?;  // Type inference
assert_eq!(first_port, 8080);

// Named multi-value → Named single value (takes first element)
let first_named = nmv.to_named_value();
assert_eq!(first_named.name(), "ports");
let val: i32 = first_named.get()?;  // Type inference
assert_eq!(val, 8080);

// Big number named value with type inference
let big_named = NamedValue::new("big_number", Value::new(BigInt::from(12345678901234567890i64)));
assert_eq!(big_named.name(), "big_number");
let big_num: &BigInt = big_named.get()?;  // Type inference
assert_eq!(big_num, &BigInt::from(12345678901234567890i64));
```

## API Reference

### Generic API

#### Construction
- **Single Value**: `Value::new<T>(t) -> Value`
- **Multi-Value**: `MultiValues::new<T>(Vec<T>) -> MultiValues`

#### Retrieval
- **Single Value**: `Value::get<T>(&self) -> ValueResult<T>`
- **Multi-Value**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **First Element**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`

#### Mutation
- **Single Value**: `Value::set<T>(&mut self, t) -> ValueResult<()>`
- **Multi-Value**:
  - `MultiValues::set<'a, T, S>(&mut self, values: S) -> ValueResult<()>` where `S` can be `T`, `Vec<T>`, or `&[T]`
  - `MultiValues::add<'a, T, S>(&mut self, values: S)` supports `T`, `Vec<T>`, or `&[T]`

### Named API

#### Single Value
- **Getters**: `get_xxx()` methods like `get_int32()`, `get_string()`, etc.
- **Setters**: `set_xxx()` methods like `set_int32()`, `set_string()`, etc.

#### Multi-Value
- **Getters**: `get_xxxs()` methods like `get_int32s()`, `get_strings()`, etc.
- **Setters**: `set_xxxs()` methods like `set_int32s()`, `set_strings()`, etc.
- **Adders**: `add_xxx()` methods like `add_int32()`, `add_string()`, etc.
- **Slice Operations**: `*_slice` methods like `set_int32s_slice()`, `add_strings_slice()`, etc.

### Type Conversion

- **Cross-Type Conversion**: `Value::as_bool()`, `as_int32()`, `as_int64()`, `as_float64()`, `as_string()`, etc.
- **Multi to Single**: `MultiValues::to_value()` retrieves the first value

### Utility Methods

#### Single Value
- `data_type()` - Get the data type
- `is_empty()` - Check if empty
- `clear()` - Clear the value
- `set_type()` - Change the type

#### Multi-Value
- `count()` - Get element count
- `is_empty()` - Check if empty
- `clear()` - Clear all values
- `set_type()` - Change the type
- `merge()` - Merge with another multi-value
- `to_value()` - Convert to single value

## Error Types

```rust
use qubit_value::{ValueError, ValueResult};
use qubit_common::lang::DataType;

// Main error variants
let no_value = ValueError::NoValue;
let type_mismatch = ValueError::TypeMismatch {
    expected: DataType::Int32,
    actual: DataType::String,
};
let conversion_failed = ValueError::ConversionFailed {
    from: DataType::String,
    to: DataType::Int32,
};
let conversion_error = ValueError::ConversionError("cannot parse number".to_string());
let index_error = ValueError::IndexOutOfBounds {
    index: 5,
    len: 3,
};

fn demo() -> ValueResult<()> { Ok(()) }
```

All operations that may fail return `ValueResult<T> = Result<T, ValueError>`.

## Supported Data Types

### Basic Types
- **Integers**: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`
- **Floats**: `f32`, `f64`
- **Other**: `bool`, `char`, `String`

### String and Bytes
- **String**: `String` (stored directly)
- **Bytes**: `Vec<u8>` (stored directly)

### Date/Time Types
- **Chrono Integration**: `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`

### Big Number Types
- **Arbitrary Precision**: `BigInt`, `BigDecimal` (from `num-bigint` and `bigdecimal` crates)

## Performance Optimizations

### Reference Returns
String and byte array named getters return references:
- `get_string() -> &str`
- `get_byte_array() -> &[u8]`

### Borrow Support
`Value::new()` and `::set()` accept borrowed types:
- `&str` automatically converted to `String`
- `&[u8]` automatically converted to `Vec<u8>`

### Smart Dispatch
`MultiValues::set/add` support three input forms: `T`, `Vec<T>`, `&[T]`, automatically dispatching to the optimal path.

## Serialization Support

### Complete Support
All types implement `Serialize`/`Deserialize`:
- `Value`
- `MultiValues`
- `NamedValue`
- `NamedMultiValues`

### Type Preservation
- Full type information is maintained during serialization
- Type validation is performed during deserialization

## Differences from Java Version

### Architectural Differences
- **Inheritance Hierarchy → Enum Representation**: Uses Rust enums instead of Java's inheritance system
- **Type Checking**: Shifts from runtime type checking to compile-time checking + minimal runtime matching
- **Error Handling**: Unified error returns instead of exceptions; cross-type conversion through explicit `as_xxx()` methods

### Performance Advantages
- **Zero-Cost Abstractions**: Enum variants store directly with no additional runtime overhead
- **Memory Efficiency**: Basic types stored directly, avoiding boxing
- **Reference Optimization**: Extensive use of reference returns to avoid unnecessary copies

## Extension Directions

### Implemented Features
- ✅ **Big Number Support**: `BigInt`/`BigDecimal` support (via third-party crates)
- ✅ **Complete Type Coverage**: Support for all basic types and common composite types
- ✅ **Serialization Support**: Complete `serde` serialization/deserialization

### Future Extensions
- 🔄 **More Conversions**: Extend `as_xxx()` conversion coverage
- 🔄 **Custom Types**: Procedural macros for enum types/custom types
- 🔄 **Performance Optimization**: Further optimize memory usage and access performance

## Dependencies

```toml
[dependencies]
qubit-common = "0.3.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
num-traits = "0.2"
num-bigint = { version = "0.4", features = ["serde"] }
bigdecimal = { version = "0.4", features = ["serde"] }
```

## Testing & Code Coverage

This project maintains comprehensive test coverage with detailed validation of all functionality. For coverage reports and testing instructions, see the [COVERAGE.md](COVERAGE.md) documentation.

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

For more information about Qubit open source projects, visit our [GitHub organization](https://github.com/qubit-ltd).
