/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Processing Framework
//!
//! Provides type-safe value storage and access functionality, supporting single values, multiple values, and named values.
//!
//! # Module Description
//!
//! - `error` - Defines error types related to value processing
//! - `value` - Single value container implementation
//! - `multi_values` - Multiple values container implementation
//! - `named` - Named value implementation
//!
//! # Core Features
//!
//! - **Type Safety**: Compile-time type checking to avoid runtime type errors
//! - **Zero-Cost Abstraction**: Implemented using enums with no additional runtime overhead
//! - **Multi-Value Support**: Unified interface for single and multiple value access
//! - **Naming Support**: Provides naming functionality for values for easy identification and lookup
//! - **Type Conversion**: Provides two sets of APIs for type checking and type conversion
//!
//! # Usage Examples
//!
//! ## Single Value Operations
//!
//! ```rust,ignore
//! use common_rs::util::value::Value;
//!
//! // Create and access a single value
//! let value = Value::Int32(42);
//! assert_eq!(value.get_int32().unwrap(), 42);
//!
//! // Type conversion
//! let text = value.as_string().unwrap();
//! assert_eq!(text, "42");
//! ```
//!
//! ## Multiple Values Operations
//!
//! ```rust,ignore
//! use common_rs::util::value::MultiValues;
//!
//! // Create and access multiple values
//! let mut values = MultiValues::Int32(vec![1, 2, 3]);
//! assert_eq!(values.count(), 3);
//!
//! // Add values
//! values.add_int32(4).unwrap();
//! assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);
//! ```
//!
//! ## Named Value Operations
//!
//! ```rust,ignore
//! use common_rs::util::value::{NamedValue, Value};
//!
//! // Create a named value
//! let config = NamedValue::new("port", Value::Int32(8080));
//! assert_eq!(config.name(), "port");
//! assert_eq!(config.get_int32().unwrap(), 8080);
//! ```
//!
//! # Author
//!
//! Haixing Hu

// Sub-modules
mod error;
pub mod multi_values;
mod named_multi_values;
mod named_value;
mod value;

// Public exports
pub use error::{ValueError, ValueResult};
pub use multi_values::MultiValues;
pub use named_multi_values::NamedMultiValues;
pub use named_value::NamedValue;
pub use value::Value;
