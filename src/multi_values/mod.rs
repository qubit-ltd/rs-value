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

// Internal lint override should only apply to this module tree.
#![allow(private_bounds)]

#[macro_use]
mod multi_values_core;
mod multi_values_accessors;
mod multi_values_converters;
mod multi_values_traits;

pub use multi_values_core::MultiValues;
