// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private abstractions shared by finite-float Serde adapters.

mod finite_float;
mod finite_float_sequence_visitor;

pub(in crate::finite_float) use finite_float::FiniteFloat;
pub(in crate::finite_float) use finite_float_sequence_visitor::FiniteFloatSequenceVisitor;
