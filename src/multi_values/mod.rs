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
mod multi_values_add_arg;
mod multi_values_adder;
mod multi_values_constructor;
mod multi_values_converters;
mod multi_values_first_getter;
mod multi_values_getter;
mod multi_values_multi_adder;
mod multi_values_multi_adder_slice;
mod multi_values_set_arg;
mod multi_values_setter;
mod multi_values_setter_slice;
mod multi_values_single_setter;

pub use multi_values::MultiValues;

// Public implementation details used by `MultiValues` generic method bounds.
#[doc(hidden)]
pub use multi_values_add_arg::MultiValuesAddArg;
#[doc(hidden)]
pub use multi_values_adder::MultiValuesAdder;
#[doc(hidden)]
pub use multi_values_constructor::MultiValuesConstructor;
#[doc(hidden)]
pub use multi_values_first_getter::MultiValuesFirstGetter;
#[doc(hidden)]
pub use multi_values_getter::MultiValuesGetter;
#[doc(hidden)]
pub use multi_values_multi_adder::MultiValuesMultiAdder;
#[doc(hidden)]
pub use multi_values_set_arg::MultiValuesSetArg;
#[doc(hidden)]
pub use multi_values_setter::MultiValuesSetter;
#[doc(hidden)]
pub use multi_values_setter_slice::MultiValuesSetterSlice;
#[doc(hidden)]
pub use multi_values_single_setter::MultiValuesSingleSetter;
