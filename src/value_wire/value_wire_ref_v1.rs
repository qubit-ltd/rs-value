// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 envelope serialization.

#[cfg(feature = "json")]
use std::io::Write;

#[cfg(feature = "json")]
use qubit_budget::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_budget::encode_to_vec;
#[cfg(feature = "json")]
use qubit_budget::encode_to_writer;
use serde::Serialize;
use serde::Serializer;

use super::ValueWireEncodeError;
use super::ValueWirePayloadRefV1;
use super::serialize_wire;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;

/// Borrowed standalone V1 envelope for serialization without cloning.
#[must_use]
pub struct ValueWireRefV1<'a> {
    value: ValueWirePayloadRefV1<'a>,
}

impl<'a> ValueWireRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_value(value).map(Self::new)
    }
    /// Borrows a collection after validating V1's finite-float invariant.
    pub fn from_values(
        values: &'a MultiValues,
    ) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_values(values).map(Self::new)
    }
    /// Borrows an explicit shape after validating V1's finite-float invariant.
    pub fn from_container(
        value: &'a ValueContainer,
    ) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_container(value).map(Self::new)
    }
    /// Wraps an already validated borrowed payload.
    pub const fn new(value: ValueWirePayloadRefV1<'a>) -> Self {
        Self { value }
    }

    /// Encodes the borrowed V1 envelope into a compact JSON vector.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 envelope with explicit JSON resource limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let mut session = JsonEncodeSession::new(limits);
        encode_to_vec(self, &mut session).map_err(ValueWireEncodeError::from)
    }

    /// Encodes the borrowed V1 envelope to a writer with default limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer<W>(
        &self,
        writer: W,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(
            writer,
            super::default_json_encode_limits(),
        )
    }

    /// Encodes the borrowed V1 envelope to a writer with explicit limits.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer_with_limits<W>(
        &self,
        writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        let mut session = JsonEncodeSession::new(limits);
        encode_to_writer(writer, self, &mut session)
            .map_err(ValueWireEncodeError::from)
    }
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(self.value.shape(), serializer)
    }
}
