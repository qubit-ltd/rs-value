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
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use serde::Serialize;
use serde::Serializer;

use super::ValueWireEncodeError;
use super::ValueWirePayloadRefV1;
use super::serialize_wire;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;

/// Borrowed standalone V1 envelope for serialization without cloning.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the runtime payload borrowed for serialization.
///
/// # Examples
///
/// ```
/// use qubit_value::{Value, ValueWireRefV1};
///
/// let value = Value::from(42_i32);
/// let _wire = ValueWireRefV1::from_value(&value).unwrap();
/// ```
#[must_use]
pub struct ValueWireRefV1<'a> {
    /// Borrowed V1 payload carried by this versioned envelope.
    value: ValueWirePayloadRefV1<'a>,
}

impl<'a> ValueWireRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `value` - Scalar runtime value to validate and borrow.
    ///
    /// # Returns
    ///
    /// A standalone borrowed V1 envelope containing the scalar.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when the scalar violates V1 numeric
    /// representation constraints.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_value(value).map(Self::new)
    }
    /// Borrows a collection after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `values` - Homogeneous runtime collection to validate and borrow.
    ///
    /// # Returns
    ///
    /// A standalone borrowed V1 envelope containing the collection.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when an element violates V1 numeric
    /// representation constraints.
    pub fn from_values(values: &'a MultiValues) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_values(values).map(Self::new)
    }
    /// Borrows an explicit shape after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `value` - Explicit scalar-or-collection value to validate and borrow.
    ///
    /// # Returns
    ///
    /// A standalone borrowed V1 envelope preserving the original shape.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when any contained value violates V1
    /// numeric representation constraints.
    pub fn from_container(value: &'a ValueContainer) -> Result<Self, ValueWireEncodeError> {
        ValueWirePayloadRefV1::from_container(value).map(Self::new)
    }
    /// Wraps an already validated borrowed payload.
    ///
    /// # Parameters
    ///
    /// * `value` - Validated borrowed payload to place in the V1 envelope.
    ///
    /// # Returns
    ///
    /// A standalone borrowed V1 envelope.
    pub const fn new(value: ValueWirePayloadRefV1<'a>) -> Self {
        Self { value }
    }

    /// Encodes the borrowed V1 envelope into a compact JSON vector.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for the complete V1 envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] for resource or serialization failures.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 envelope with explicit JSON resource limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for the complete V1 envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits` or
    /// Serde rejects the envelope.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec_with_limits(&self, limits: JsonEncodeLimits) -> Result<Vec<u8>, ValueWireEncodeError> {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .to_vec(self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Encodes the borrowed V1 envelope to a writer with default limits.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete V1 JSON envelope.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete envelope is written.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] for resource, serialization, or writer
    /// failures.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer<W>(&self, writer: W) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(writer, super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 envelope to a writer with explicit limits.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete V1 JSON envelope.
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete envelope is written.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits`, Serde
    /// rejects the envelope, or `writer` rejects output.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer_with_limits<W>(&self, writer: W, limits: JsonEncodeLimits) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(writer, self)
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
