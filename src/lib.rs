// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Processing Framework
//!
//! Provides type-safe value storage and access functionality, supporting single
//! values, collections, explicit scalar-or-collection shape, and named values.
//!
//! # Public API Overview
//!
//! - [`Value`] stores one typed scalar, including an explicit `Unset(DataType)`
//!   state.
//! - [`MultiValues`] stores one homogeneous typed collection.
//! - [`ValueRef`] and [`MultiValuesRef`] expose borrowed semantic views while
//!   keeping runtime storage private.
//! - [`ValueContainer`] preserves whether storage is scalar or collection.
//! - [`NamedValue`] and [`NamedMultiValues`] provide name wrappers.
//! - [`ValueWireV1`] and [`ValueWirePayloadV1`] name explicit Serde DTOs.
//! - [`ValueWireRefV1`] and [`ValueWirePayloadRefV1`] serialize borrowed
//!   values.
//!
//! # Core behavior
//!
//! - [`Value::get`] and [`MultiValues::get`] perform strict typed reads.
//! - [`ValueContainer`] preserves whether the source supplied a scalar or an
//!   explicit collection, even when the collection contains one item.
//! - `to` methods use `qubit-datatype` conversion policy and resource limits.
//! - Optional type families and conversion methods are available only when the
//!   corresponding crate features are enabled; all-features documentation shows
//!   the superset of those APIs.
//! - [`Value::is_unset`] and [`ValueContainer::is_unset`] indicate that no
//!   concrete value is stored.
//! - [`MultiValues::is_unset`] distinguishes no collection from a concrete
//!   collection; [`MultiValues::is_empty`] reports only that its length is
//!   zero.
//! - Generic `set` replaces a value infallibly; [`MultiValues::add`] remains
//!   fallible because appended values must have the same data type.
//! - Serde uses the strict, type-preserving [`ValueWireV1`] envelope. Its
//!   canonical JSON representation is byte-stable for the same value under the
//!   supported `serde_json` version and configuration. String-map keys and
//!   nested JSON object keys are emitted in lexicographic order. Other Serde
//!   formats are supported as representations, but are outside this byte-level
//!   stability contract. With both `converter` and `json`, `to_json_value`
//!   provides a separate natural JSON projection with the same ordering.
//! - Version one rejects the pre-0.10 externally tagged representation.
//! - Non-finite floats may exist in memory, but V1 Serde and natural JSON
//!   reject them because JSON has no `NaN` or infinity number literals.
//! - JSON numbers follow `qubit-json`'s explicit range contract: negative
//!   integers fit `i64`, non-negative integers fit `u64`, and fractional or
//!   exponential values are finite `f64`. Wider exact values use the crate's
//!   explicit string-based integer and decimal wire representations.
//!
//! # Usage Examples
//!
//! ## Single Value Operations
//!
//! ```rust
//! use qubit_value::Value;
//!
//! // Create and access a single value
//! let value = Value::Int32(42);
//! assert_eq!(value.get_int32().unwrap(), 42);
//!
//! // Strict generic access
//! let number: i32 = value.get().unwrap();
//! assert_eq!(number, 42);
//! ```
//!
//! ## Multiple Values Operations
//!
//! ```rust
//! use qubit_value::MultiValues;
//!
//! // Create and access multiple values
//! let mut values = MultiValues::Int32(vec![1, 2, 3]);
//! assert_eq!(values.len(), 3);
//!
//! // Add values
//! values.add(4).unwrap();
//! assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);
//! ```
//!
//! ## Named Value Operations
//!
//! ```rust
//! use qubit_value::{NamedValue, Value};
//!
//! // Create a named value
//! let config = NamedValue::new("port", Value::Int32(8080));
//! assert_eq!(config.name(), "port");
//! assert_eq!(config.value().get_int32().unwrap(), 8080);
//! ```
//!
//! ## Explicit Shape Operations
//!
//! ```rust
//! use qubit_value::ValueContainer;
//!
//! let scalar = ValueContainer::from(42_i32);
//! let collection = ValueContainer::from(vec![42_i32]);
//! assert!(scalar.is_scalar());
//! assert!(collection.is_collection());
//! ```

// Sub-modules
mod finite_float;
mod identity;
mod into_value_default;
#[macro_use]
mod value_type_table;
#[cfg(all(feature = "converter", feature = "json"))]
mod json;
mod multi_values;
mod named_multi_values;
mod named_value;
mod numeric_comparison_error;
#[cfg(all(feature = "converter", feature = "json"))]
mod strict_json;
mod strict_value_read;
mod value;
mod value_container;
mod value_error;
mod value_missing;
mod value_wire;
mod wide_integer;
mod wire;

// Public exports
pub use self::into_value_default::IntoValueDefault;
pub use self::multi_values::MultiValues;
pub use self::multi_values::MultiValuesRef;
pub use self::named_multi_values::NamedMultiValues;
pub use self::named_value::NamedValue;
pub use self::numeric_comparison_error::NumericComparisonError;
pub use self::strict_value_read::StrictValueRead;
pub use self::value::Value;
pub use self::value::ValueRef;
pub use self::value_container::ValueContainer;
pub use self::value_error::ValueError;
pub use self::value_error::ValueResult;
pub use self::value_missing::ValueMissing;
#[cfg(feature = "json")]
pub use self::value_wire::ValueWireDecodeError;
pub use self::value_wire::ValueWireEncodeError;
pub use self::value_wire::ValueWirePayloadRefV1;
pub use self::value_wire::ValueWirePayloadV1;
pub use self::value_wire::ValueWireRefV1;
pub use self::value_wire::ValueWireV1;
