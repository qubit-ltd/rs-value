/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # Multiple Values Module
//!
//! Public entry for multiple-values container implementations.

#[allow(clippy::module_inception)]
#[macro_use]
mod multi_values;
mod multi_values_accessors;
mod multi_values_add_arg;
mod multi_values_adder;
mod multi_values_constructor;
mod multi_values_constructor_arg;
mod multi_values_converters;
mod multi_values_first_getter;
mod multi_values_getter;
mod multi_values_multi_adder;
mod multi_values_multi_adder_slice;
mod multi_values_set_arg;
mod multi_values_setter;
mod multi_values_setter_slice;
mod multi_values_single_setter;

/// Private marker trait used to prevent downstream implementations.
mod sealed {
    pub trait MultiValuesAddArgSealed {}
    pub trait MultiValuesAdderSealed<T> {}
    pub trait MultiValuesConstructorArgSealed {}
    pub trait MultiValuesConstructorSealed<T> {}
    pub trait MultiValuesFirstGetterSealed<T> {}
    pub trait MultiValuesGetterSealed<T> {}
    pub trait MultiValuesMultiAdderSealed<T> {}
    pub trait MultiValuesSetArgSealed {}
    pub trait MultiValuesSetterSealed<T> {}
    pub trait MultiValuesSetterSliceSealed<T> {}
    pub trait MultiValuesSingleSetterSealed<T> {}
}

pub use multi_values::MultiValues;
