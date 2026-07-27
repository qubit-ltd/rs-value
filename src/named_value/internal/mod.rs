// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private wire representations for [`super::NamedValue`].

mod named_value_wire_owned;
mod named_value_wire_ref;

pub(in crate::named_value) use named_value_wire_owned::NamedValueWireOwned;
pub(in crate::named_value) use named_value_wire_ref::NamedValueWireRef;
