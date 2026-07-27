// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unversioned V1 payload for use inside an already-versioned protocol.

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

use crate::{
    MultiValues,
    Value,
    ValueContainer,
};

use super::value_wire_payload_ref_v1::{
    validate_value,
    validate_values,
};
use super::{
    ValueWireEncodeError,
    WireShapeOwned,
    WireShapeRef,
};

/// Typed V1 scalar-or-collection payload without an enclosing version field.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueWirePayloadV1 {
    /// Preserved runtime shape and payload.
    value: ValueContainer,
}

impl ValueWirePayloadV1 {
    /// Borrows the preserved runtime value.
    #[inline(always)]
    pub const fn container(&self) -> &ValueContainer {
        &self.value
    }

    /// Consumes this payload and returns its runtime value.
    #[inline(always)]
    pub fn into_container(self) -> ValueContainer {
        self.value
    }

    /// Builds a payload after enforcing V1's finite-float invariant.
    fn try_new(value: ValueContainer) -> Result<Self, ValueWireEncodeError> {
        match &value {
            ValueContainer::Scalar(value) => validate_value(value)?,
            ValueContainer::Collection(values) => validate_values(values)?,
        }
        Ok(Self { value })
    }

    /// Wraps a payload decoded through V1's finite-number Serde adapters.
    pub(in crate::value_wire) const fn from_decoded(
        value: ValueContainer,
    ) -> Self {
        Self { value }
    }
}

impl TryFrom<Value> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates a scalar for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Self::try_new(value.into())
    }
}

impl TryFrom<MultiValues> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates a collection for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: MultiValues) -> Result<Self, Self::Error> {
        Self::try_new(value.into())
    }
}

impl TryFrom<ValueContainer> for ValueWirePayloadV1 {
    type Error = ValueWireEncodeError;

    /// Validates an explicitly shaped value for use in a V1 payload.
    #[inline(always)]
    fn try_from(value: ValueContainer) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl From<ValueWirePayloadV1> for ValueContainer {
    /// Extracts the shaped runtime value from a V1 payload.
    #[inline(always)]
    fn from(value: ValueWirePayloadV1) -> Self {
        value.into_container()
    }
}

impl Serialize for ValueWirePayloadV1 {
    /// Serializes the unversioned V1 shape.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        WireShapeRef::from(&self.value).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValueWirePayloadV1 {
    /// Deserializes an unversioned V1 shape.
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_decoded(
            WireShapeOwned::deserialize(deserializer)?.into(),
        ))
    }
}
