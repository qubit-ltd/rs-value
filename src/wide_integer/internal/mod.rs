// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal serializer types for canonical wide integers.

mod display_integer;
mod integer_visitor;
mod parsed_integer;

pub(in crate::wide_integer) use display_integer::DisplayInteger;
pub(in crate::wide_integer) use integer_visitor::IntegerVisitor;
pub(in crate::wide_integer) use parsed_integer::ParsedInteger;
