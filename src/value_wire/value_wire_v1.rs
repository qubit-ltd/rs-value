// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Public version-one wire DTO.

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

use super::VALUE_WIRE_V1_VERSION;

/// Stable version-one wire DTO for a scalar or homogeneous collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueWireV1 {
    /// Explicit runtime shape and typed payload represented by this DTO.
    value: ValueContainer,
}

impl ValueWireV1 {
    /// Numeric version emitted and accepted by this DTO.
    pub const VERSION: u8 = VALUE_WIRE_V1_VERSION;

    /// Creates a V1 DTO from an explicit scalar-or-collection container.
    ///
    /// # Arguments
    ///
    /// * `value` - Runtime container whose exact type and shape are preserved.
    ///
    /// # Returns
    ///
    /// A V1 DTO containing `value`.
    #[inline]
    pub const fn new(value: ValueContainer) -> Self {
        Self { value }
    }

    /// Returns the runtime container represented by this DTO.
    ///
    /// # Returns
    ///
    /// A shared reference to the preserved runtime container.
    #[inline(always)]
    pub const fn container(&self) -> &ValueContainer {
        &self.value
    }

    /// Consumes the DTO and returns its runtime container.
    ///
    /// # Returns
    ///
    /// The preserved runtime container.
    #[inline(always)]
    pub fn into_container(self) -> ValueContainer {
        self.value
    }
}

impl From<Value> for ValueWireV1 {
    /// Wraps a runtime scalar in a V1 DTO.
    #[inline]
    fn from(value: Value) -> Self {
        Self::new(ValueContainer::Scalar(value))
    }
}

impl From<MultiValues> for ValueWireV1 {
    /// Wraps a runtime collection in a V1 DTO.
    #[inline]
    fn from(values: MultiValues) -> Self {
        Self::new(ValueContainer::Collection(values))
    }
}

impl From<ValueContainer> for ValueWireV1 {
    /// Wraps an explicit runtime shape in a V1 DTO.
    #[inline]
    fn from(value: ValueContainer) -> Self {
        Self::new(value)
    }
}

impl From<ValueWireV1> for ValueContainer {
    /// Unwraps the runtime container from a V1 DTO.
    #[inline]
    fn from(value: ValueWireV1) -> Self {
        value.into_container()
    }
}

impl Serialize for ValueWireV1 {
    /// Serializes the contained runtime shape through the V1 contract.
    #[inline(always)]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ValueWireV1 {
    /// Deserializes a validated V1 runtime container into the DTO.
    #[inline(always)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ValueContainer::deserialize(deserializer).map(Self::new)
    }
}
