// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private wire representations for [`super::NamedMultiValues`].

mod named_multi_values_wire_owned;
mod named_multi_values_wire_ref;

pub(in crate::named_multi_values) use named_multi_values_wire_owned::NamedMultiValuesWireOwned;
pub(in crate::named_multi_values) use named_multi_values_wire_ref::NamedMultiValuesWireRef;
