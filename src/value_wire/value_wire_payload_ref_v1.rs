// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload serialization.

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
use super::WireShapeRef;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
use crate::multi_values::MultiValuesRepr;
use crate::value::ValueRepr;
#[cfg(feature = "big-decimal")]
use crate::wire::MAX_BIG_DECIMAL_ABSOLUTE_SCALE;
#[cfg(feature = "big-decimal")]
use crate::wire::is_valid_big_decimal_scale;

/// Borrowed unversioned V1 payload for serialization without cloning.
///
/// Use one of the fallible constructors or the corresponding `TryFrom` impl
/// to create a payload. The private representation prevents callers from
/// bypassing V1 validation by constructing an unchecked shape directly.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the runtime payload borrowed for serialization.
///
/// # Examples
///
/// ```
/// use qubit_value::{Value, ValueWirePayloadRefV1};
///
/// let value = Value::from(42_i32);
/// let _payload = ValueWirePayloadRefV1::from_value(&value).unwrap();
/// ```
#[must_use]
pub struct ValueWirePayloadRefV1<'a> {
    /// Borrowed scalar-or-collection shape validated for V1 serialization.
    shape: WireShapeRef<'a>,
}

impl<'a> ValueWirePayloadRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `value` - Scalar runtime value to validate and borrow.
    ///
    /// # Returns
    ///
    /// A borrowed V1 payload preserving the scalar shape.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when the scalar contains a non-finite
    /// float or an out-of-range decimal scale.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        validate_value(value)?;
        Ok(Self {
            shape: WireShapeRef::Scalar(value.into()),
        })
    }

    /// Borrows a collection after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `values` - Homogeneous runtime collection to validate and borrow.
    ///
    /// # Returns
    ///
    /// A borrowed V1 payload preserving the collection shape.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when an element is a non-finite float
    /// or has an out-of-range decimal scale.
    pub fn from_values(values: &'a MultiValues) -> Result<Self, ValueWireEncodeError> {
        validate_values(values)?;
        Ok(Self {
            shape: WireShapeRef::Collection(values.into()),
        })
    }

    /// Borrows an explicit shape after validating V1's finite-float invariant.
    ///
    /// # Parameters
    ///
    /// * `value` - Explicit scalar-or-collection value to validate and borrow.
    ///
    /// # Returns
    ///
    /// A borrowed V1 payload preserving the original shape.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when any contained value violates the
    /// V1 numeric representation constraints.
    pub fn from_container(value: &'a ValueContainer) -> Result<Self, ValueWireEncodeError> {
        match value {
            ValueContainer::Scalar(value) => validate_value(value)?,
            ValueContainer::Collection(values) => validate_values(values)?,
        }
        Ok(Self {
            shape: value.into(),
        })
    }

    /// Returns the borrowed internal shape used by V1 serialization.
    ///
    /// # Returns
    ///
    /// A copyable borrowed view of the validated scalar-or-collection shape.
    #[must_use]
    #[inline(always)]
    pub(in crate::value_wire) fn shape(&self) -> WireShapeRef<'a> {
        self.shape
    }

    /// Encodes the borrowed V1 payload into a compact JSON vector.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for this unversioned payload.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds the default
    /// resource profile or Serde rejects the payload.
    #[cfg(feature = "json")]
    #[inline(always)]
    pub fn to_json_vec(&self) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.to_json_vec_with_limits(super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 payload with explicit JSON resource limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// Compact UTF-8 JSON bytes for this unversioned payload.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits` or
    /// Serde rejects the payload.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec_with_limits(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<Vec<u8>, ValueWireEncodeError> {
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .to_vec(self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Encodes the borrowed V1 payload to a writer with default limits.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete JSON payload.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete payload is written.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] for resource, serialization, or writer
    /// failures.
    #[cfg(feature = "json")]
    #[inline(always)]
    pub fn to_json_writer<W>(&self, writer: W) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.to_json_writer_with_limits(writer, super::default_json_encode_limits())
    }

    /// Encodes the borrowed V1 payload to a writer with explicit limits.
    ///
    /// # Type Parameters
    ///
    /// * `W` - Destination writer type.
    ///
    /// # Parameters
    ///
    /// * `writer` - Destination receiving the complete JSON payload.
    /// * `limits` - Resource limits enforced during JSON encoding.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete payload is written.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when encoding exceeds `limits`, Serde
    /// rejects the payload, or `writer` rejects output.
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
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(writer, self)
            .map_err(ValueWireEncodeError::from)
    }
}

/// Validates one scalar against V1's JSON finite-float invariant.
///
/// # Parameters
///
/// * `value` - Scalar value to validate without modifying it.
///
/// # Returns
///
/// `Ok(())` when the scalar is representable by the V1 wire contract.
///
/// # Errors
///
/// Returns [`ValueWireEncodeError`] for a non-finite float or an out-of-range
/// arbitrary-precision decimal scale.
pub(in crate::value_wire) fn validate_value(value: &Value) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "big-decimal")]
    if let ValueRepr::BigDecimal(value) = &value.repr {
        validate_big_decimal_scale(value.as_bigint_and_exponent().1)?;
    }
    let non_finite = matches!(&value.repr, ValueRepr::Float32(value) if !value.is_finite())
        || matches!(&value.repr, ValueRepr::Float64(value) if !value.is_finite());
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: value.data_type(),
        });
    }
    Ok(())
}

/// Validates one collection against V1's JSON finite-float invariant.
///
/// # Parameters
///
/// * `values` - Collection to validate without modifying it.
///
/// # Returns
///
/// `Ok(())` when every element is representable by the V1 wire contract.
///
/// # Errors
///
/// Returns [`ValueWireEncodeError`] for a non-finite float element or an
/// out-of-range arbitrary-precision decimal scale.
pub(in crate::value_wire) fn validate_values(
    values: &MultiValues,
) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "big-decimal")]
    if let MultiValuesRepr::BigDecimal(values) = &values.repr {
        for value in values {
            validate_big_decimal_scale(value.as_bigint_and_exponent().1)?;
        }
    }
    let non_finite = match &values.repr {
        MultiValuesRepr::Float32(values) => values.iter().any(|value| !value.is_finite()),
        MultiValuesRepr::Float64(values) => values.iter().any(|value| !value.is_finite()),
        _ => false,
    };
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: values.data_type(),
        });
    }
    Ok(())
}

/// Validates the decimal exponent accepted by V1's bounded payload format.
///
/// # Parameters
///
/// * `scale` - Decimal scale to compare with the V1 inclusive bound.
///
/// # Returns
///
/// `Ok(())` when the scale is within the supported magnitude.
///
/// # Errors
///
/// Returns [`ValueWireEncodeError::BigDecimalScaleTooLarge`] when `scale`
/// exceeds the V1 bound.
#[cfg(feature = "big-decimal")]
fn validate_big_decimal_scale(scale: i64) -> Result<(), ValueWireEncodeError> {
    if is_valid_big_decimal_scale(scale) {
        return Ok(());
    }
    Err(ValueWireEncodeError::BigDecimalScaleTooLarge {
        scale,
        maximum_absolute_scale: MAX_BIG_DECIMAL_ABSOLUTE_SCALE,
    })
}

impl<'a> TryFrom<&'a Value> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a scalar.
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Self::from_value(value)
    }
}

impl<'a> TryFrom<&'a MultiValues> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a collection.
    fn try_from(values: &'a MultiValues) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

impl<'a> TryFrom<&'a ValueContainer> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates an explicit shape.
    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> {
        Self::from_container(value)
    }
}

impl Serialize for ValueWirePayloadRefV1<'_> {
    /// Serializes the borrowed unversioned V1 shape.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.shape().serialize(serializer)
    }
}
