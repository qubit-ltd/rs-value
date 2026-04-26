/***************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Module
//!
//! Public entry for the single-value container implementation.

#[allow(clippy::module_inception)]
mod value;
mod value_accessors;
mod value_converters;

pub use value::Value;

// Public implementation details used by `Value` generic method bounds.
#[doc(hidden)]
pub use value::{ValueConstructor, ValueConverter, ValueGetter, ValueSetter};
