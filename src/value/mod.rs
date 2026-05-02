/***************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Value Module
//!
//! Public entry for the single-value container implementation.

#[allow(clippy::module_inception)]
mod value;
mod value_accessors;
mod value_constructor;
mod value_converter;
mod value_converters;
mod value_getter;
mod value_setter;

/// Private marker trait used to prevent downstream implementations.
mod sealed {
    pub trait ValueConstructorSealed<T> {}
    pub trait ValueConverterSealed<T> {}
    pub trait ValueGetterSealed<T> {}
    pub trait ValueSetterSealed<T> {}
}

pub use value::Value;

// Public implementation details used by `Value` generic method bounds.
#[doc(hidden)]
pub use value::{
    ValueConstructor,
    ValueConverter,
    ValueGetter,
    ValueSetter,
};
