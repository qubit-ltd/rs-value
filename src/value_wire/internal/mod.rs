// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal payload and envelope types for the V1 wire contract.

mod collection_wire_owned;
mod collection_wire_ref;
#[cfg(feature = "json")]
mod display_length;
mod scalar_wire_owned;
mod scalar_wire_ref;
mod wire_data_type_v1;
mod wire_envelope_owned;
mod wire_envelope_ref;
mod wire_shape_owned;
mod wire_shape_ref;

pub(in crate::value_wire) use collection_wire_owned::CollectionWireOwned;
pub(in crate::value_wire) use collection_wire_ref::CollectionWireRef;
#[cfg(feature = "json")]
pub(in crate::value_wire) use display_length::display_length;
pub(in crate::value_wire) use scalar_wire_owned::ScalarWireOwned;
pub(in crate::value_wire) use scalar_wire_ref::ScalarWireRef;
pub(in crate::value_wire) use wire_data_type_v1::WireDataTypeV1;
pub(in crate::value_wire) use wire_envelope_owned::WireEnvelopeOwned;
pub(in crate::value_wire) use wire_envelope_ref::WireEnvelopeRef;
pub(in crate::value_wire) use wire_shape_owned::WireShapeOwned;
pub(in crate::value_wire) use wire_shape_ref::WireShapeRef;
