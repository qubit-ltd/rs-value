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
//! - [`ValueContainer`] preserves whether storage is scalar or collection.
//! - [`NamedValue`] and [`NamedMultiValues`] provide name wrappers.
//! - [`ValueWireV1`] names the versioned, type-preserving Serde contract.
//!
//! # Core behavior
//!
//! - [`Value::get`] and [`MultiValues::get`] perform strict typed reads.
//! - [`ValueContainer`] preserves whether the source supplied a scalar or an
//!   explicit collection, even when the collection contains one item.
//! - `to` methods use `qubit-datatype` conversion rules and options.
//! - Optional type families and conversion methods are available only when the
//!   corresponding crate features are enabled; all-features documentation shows
//!   the superset of those APIs.
//! - [`Value::is_unset`] and [`MultiValues::is_unset`] distinguish an unset
//!   container from a concrete value or concrete empty collection.
//! - Generic `set` replaces a value infallibly; [`MultiValues::add`] remains
//!   fallible because appended values must have the same data type.
//! - Serde uses the strict, type-preserving [`ValueWireV1`] envelope. V1
//!   compatibility covers its documented JSON structure; other serializer
//!   formats are outside that stability contract. With both `converter` and
//!   `json`, `to_json_value` provides a separate natural JSON projection.
//! - Version one rejects the pre-0.10 externally tagged representation.
//! - Non-finite floats may exist in memory, but V1 Serde and natural JSON
//!   reject them because JSON has no `NaN` or infinity number literals.
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
//! assert_eq!(values.count(), 3);
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

#![cfg_attr(docsrs, feature(doc_cfg))]

// Sub-modules
mod finite_float;
mod identity;
mod into_value_default;
#[macro_use]
mod value_type_table;
#[cfg(all(feature = "converter", feature = "json"))]
mod json;
pub mod multi_values;
mod named_multi_values;
mod named_value;
mod numeric_comparison_error;
#[cfg(all(feature = "converter", feature = "json"))]
mod strict_json;
mod strict_value_list_read;
mod strict_value_read;
mod value;
mod value_container;
mod value_error;
mod value_wire;
mod wide_integer;
mod wire;

// Public exports
pub use into_value_default::IntoValueDefault;
pub use multi_values::MultiValues;
pub use named_multi_values::NamedMultiValues;
pub use named_value::NamedValue;
pub use numeric_comparison_error::NumericComparisonError;
pub use strict_value_list_read::StrictValueListRead;
pub use strict_value_read::StrictValueRead;
pub use value::Value;
pub use value_container::ValueContainer;
pub use value_error::{
    ValueError,
    ValueResult,
};
pub use value_wire::ValueWireV1;
