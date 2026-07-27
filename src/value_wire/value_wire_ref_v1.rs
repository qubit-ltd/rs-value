// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 envelope serialization.

use serde::{Serialize, Serializer};

use crate::{MultiValues, Value, ValueContainer};

use super::{serialize_wire, ValueWireEncodeError, ValueWirePayloadRefV1};

/// Borrowed standalone V1 envelope for serialization without cloning.
#[must_use]
pub struct ValueWireRefV1<'a> { value: ValueWirePayloadRefV1<'a> }

impl<'a> ValueWireRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_value(value).map(Self::new)
    }
    /// Borrows a collection after validating V1's finite-float invariant.
    pub fn from_values(values: &'a MultiValues) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_values(values).map(Self::new)
    }
    /// Borrows an explicit shape after validating V1's finite-float invariant.
    pub fn from_container(value: &'a ValueContainer) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_container(value).map(Self::new)
    }
    /// Wraps an already validated borrowed payload.
    pub const fn new(value: ValueWirePayloadRefV1<'a>) -> Self { Self { value } }
}

impl<'a> TryFrom<&'a Value> for ValueWireRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a scalar.
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl<'a> TryFrom<&'a MultiValues> for ValueWireRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a collection.
    fn try_from(values: &'a MultiValues) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

impl<'a> TryFrom<&'a ValueContainer> for ValueWireRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates an explicit shape.
    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> {
        Self::from_container(value)
    }
}

impl Serialize for ValueWireRefV1<'_> {
    /// Serializes the borrowed runtime shape through the V1 envelope.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serialize_wire(self.value.shape(), serializer)
    }
}
