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
mod value_core;
mod value_traits;

pub use value::{Value, ValueConstructor, ValueConverter, ValueGetter, ValueSetter};
