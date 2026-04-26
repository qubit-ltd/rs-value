/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Multiple Values Module
//!
//! Public entry for multiple-values container implementations.

#[allow(clippy::module_inception)]
#[macro_use]
mod multi_values;
mod multi_values_accessors;
mod multi_values_converters;

pub use multi_values::MultiValues;

// Public implementation details used by `MultiValues` generic method bounds.
#[doc(hidden)]
pub use multi_values_converters::{
    MultiValuesAddArg, MultiValuesAdder, MultiValuesConstructor, MultiValuesFirstGetter,
    MultiValuesGetter, MultiValuesMultiAdder, MultiValuesSetArg, MultiValuesSetter,
    MultiValuesSetterSlice, MultiValuesSingleSetter,
};
