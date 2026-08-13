# Qubit Value

[![Rust CI](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-value/coverage-badge.json)](https://qubit-ltd.github.io/rs-value/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-value` gives Rust applications one type-safe boundary for values that are
known only at runtime. It is useful when configuration, metadata, protocol
fields, or user input can be a boolean, number, string, date, collection, or
structured JSON value, but the application still needs explicit types,
controlled conversion, and predictable errors.

## The problem it solves

Without a shared runtime value model, each key-value subsystem tends to invent
its own `enum`, conversion rules, unset semantics, and serialization format.
That creates three recurring problems:

- a missing value, an explicitly empty collection, and JSON `null` are easily
  confused;
- a one-item collection can be accidentally treated as a scalar;
- values crossing a process or storage boundary lose their runtime type, or
  accept conversions that were never intended.

`Value` stores one typed scalar, `MultiValues` stores one homogeneous
collection, and `ValueContainer` preserves the explicit scalar-or-collection
shape. `Unset(DataType)` retains the declared type without pretending that a
concrete value exists.

## Quick start: a small runtime configuration map

This is a small configuration-like map: each key stores a different `Value`,
then the reader chooses strict access, explicit conversion, or a typed default.
The snippet assumes it is inside a function that returns a compatible `Result`,
so `?` can propagate value errors.

```rust
use std::collections::HashMap;
use std::time::Duration;

use qubit_datatype::DataType;
use qubit_value::Value;

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
```

If you need a complete, general-purpose configuration object instead of
assembling a map yourself, use `Config` from
[`rs-config`](https://github.com/qubit-ltd/rs-config). It builds on `Value` and
adds higher-level capabilities such as property management, typed and
multi-value reads, defaults, sections, conversion policies, interpolation, and
pluggable file/environment configuration sources.

`get()` is a strict type read: it does not silently convert. `to()` uses the
shared conversion rules from `qubit-datatype`; failed conversions remain
errors. The `converter` feature is required for `to()` and `to_or()`; `get_or()`
only supplies a fallback for an unset value and does not convert.

Use `to_with` when the boundary needs an explicit policy and limits. Every
`to_with` call creates a fresh `ConversionSession`, so independent reads do not
share cumulative consumption:

```rust
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_value::Value;

let policy = ConversionPolicy::env_friendly();
let limits = ConversionLimits::default();
let first = Value::new(" 8080 ").to_with::<u16>(&policy, &limits)?;
let second = Value::new(" 8081 ").to_with::<u16>(&policy, &limits)?;
assert_eq!((first, second), (8080, 8081));
```

## Installation

Add the core crate and its type vocabulary to `Cargo.toml`:

```toml
[dependencies]
qubit-value = "0.10"
qubit-datatype = { version = "0.11", default-features = false }
```

The default feature set is empty. Enable only the families you use:

| Feature | Additional `DataType` or capability |
| --- | --- |
| `converter` | Cross-type conversion APIs such as `Value::to` |
| `chrono` | `Date`, `Time`, `DateTime`, and `Instant` |
| `big-integer` | `BigInteger` backed by `num_bigint::BigInt` |
| `big-decimal` | `BigDecimal` backed by `bigdecimal::BigDecimal` |
| `big-number` | Compatibility alias for both big-number features |
| `url` | `Url` backed by `url::Url` |
| `json` | `Json` backed by `serde_json::Value` and bounded JSON Wire decoding; Natural JSON also requires `converter` |
| `redact` | Policy-aware redacted views through `qubit-redact` |
| `all` | `converter`, `chrono`, `big-number`, `url`, `json`, and `redact` |

## Supported `DataType` values

`qubit_datatype::DataType` is the closed runtime type vocabulary used by
`Value`, `MultiValues`, and `Unset`. The same type identifies a scalar and its
homogeneous collection form.

| `DataType` | Rust value | Feature | Typical use |
| --- | --- | --- | --- |
| `Bool` | `bool` | — | flags and switches |
| `Char` | `char` | — | one Unicode character |
| `Int8` / `Int16` / `Int32` / `Int64` / `Int128` | `i8` … `i128` | — | signed integers |
| `UInt8` / `UInt16` / `UInt32` / `UInt64` / `UInt128` | `u8` … `u128` | — | unsigned integers |
| `Float32` / `Float64` | `f32` / `f64` | — | finite or in-memory floating-point values |
| `String` | `String` | — | text and text-backed input |
| `Date` | `chrono::NaiveDate` | `chrono` | calendar date |
| `Time` | `chrono::NaiveTime` | `chrono` | time of day |
| `DateTime` | `chrono::NaiveDateTime` | `chrono` | date and local time |
| `Instant` | `chrono::DateTime<chrono::Utc>` | `chrono` | UTC time point |
| `BigInteger` | `num_bigint::BigInt` | `big-integer` | arbitrary-precision integer |
| `BigDecimal` | `bigdecimal::BigDecimal` | `big-decimal` | arbitrary-precision decimal |
| `Duration` | `std::time::Duration` | — | elapsed time |
| `Url` | `url::Url` | `url` | parsed URL |
| `StringMap` | `HashMap<String, String>` | — | string-key/string-value properties |
| `Json` | `serde_json::Value` | `json` | arbitrary JSON structure |

The 25 variants above are the complete current `DataType` enum. A feature-gated
variant can still appear in an `Unset(DataType)` declaration, but a concrete
value of that type requires the corresponding feature in the build that stores
or reads it.

## What it provides

- `Value` and `MultiValues` have typed constructors, typed getters, generic
  mutation, borrowed reads, and explicit unset state.
- `ValueContainer::Scalar` and `ValueContainer::Collection` preserve shape;
  a one-item collection remains a collection.
- `get_or`/`to_or` and collection variants make fallback behavior explicit:
  unset values can use defaults, while type mismatches and ordinary conversion
  failures are still reported.
- `NamedValue` and `NamedMultiValues` attach a key to a runtime value without
  changing the value's type semantics.
- `ValueWireV1` provides a versioned, type-preserving JSON representation with
  bounded `to_json_vec()` and `to_json_writer()` entry points; explicit
  `JsonDecodeLimits` and `JsonEncodeLimits` are accepted by the corresponding
  directional `_with_limits` methods. Decode uses a caller-configured
  `JsonDecodeSession`; encode uses `JsonEncodeSession` to enforce structure and
  output bytes online. Use Wire V1 when the receiver must reconstruct the exact
  `DataType` and shape.
- Natural JSON helpers produce ordinary `null`, scalar, object, and array
  values when runtime type tags are not wanted.

The crate does not provide a complete configuration store, schema registry,
file format, or distributed cache. It provides the typed value layer those
systems can build on. Its `Eq`/`Hash` implementations are suitable for
in-memory Rust collections, not persistent fingerprints or distributed-cache
keys.

## Built on `Value`

Two sibling crates use this value model for key-value containers:

- [`rs-config`](https://github.com/qubit-ltd/rs-config) provides typed
  configuration properties, configuration-file and environment-oriented
  access, and policy-controlled reads. Use it when the key-value data describes
  application configuration.
- [`rs-metadata`](https://github.com/qubit-ltd/rs-metadata) provides typed
  metadata/property storage and filtering. Use it when values describe
  resources, records, or searchable application metadata.

## Wire V1 or Natural JSON?

Choose Wire V1 when a receiver must distinguish `Int32(42)` from `String("42")`,
preserve scalar versus collection shape, or retain `Unset(DataType)`. A typical
document is:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

Choose `to_json_value()` when the boundary is ordinary application JSON and
the receiver only needs JSON semantics. Wire V1 is closed and versioned;
Natural JSON intentionally omits runtime type tags. The user guide contains the
full Wire workflow, borrowed payload examples, feature compatibility rules,
and resource-limit handling.

Directional failures retain operation accounting: a rejected charge does not
consume that request, but accepted input, output, node, or payload consumption
from earlier in the same session is not rolled back. Construct a fresh session
for each independent wire operation.

For example, Natural JSON produces these exact JSON strings:

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

The scalar number becomes `42`, the string keeps its JSON quotes, an unset
value becomes `null`, a concrete collection becomes an array, and string-map
keys are emitted in dictionary order.

## Learn more

- [English user guide](doc/user_guide.md)
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-value)
- [`qubit-datatype` conversion contract](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
- [中文 README](README.zh_CN.md)

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
