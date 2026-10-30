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

#[cfg(feature = "chrono")]
use chrono::Datelike;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeLimits;
#[cfg(feature = "json")]
use qubit_budget::json::JsonEncodeSession;
#[cfg(feature = "json")]
use qubit_json::encode::JsonEncoder;
use serde::Serialize;
use serde::Serializer;

use super::ValueWireEncodeError;
#[cfg(feature = "json")]
use super::ValueWireEncodePreflight;
use super::WireShapeRef;
use super::internal::WireSourceRef;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
use crate::multi_values::MultiValuesRepr;
use crate::value::ValueRepr;
#[cfg(feature = "big-decimal")]
use crate::wire::MAX_BIG_DECIMAL_ABSOLUTE_SCALE;
#[cfg(feature = "big-decimal")]
use crate::wire::is_valid_big_decimal_scale;
#[cfg(feature = "chrono")]
use crate::wire::validate_instant_year;

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
/// let payload = ValueWirePayloadRefV1::from_value(&value).unwrap();
/// let encoded = serde_json::to_string(&payload).unwrap();
/// assert_eq!(encoded, r#"{"scalar":{"int32":42}}"#);
/// let owned = qubit_value::ValueWirePayloadV1::try_from(Value::from(42_i32)).unwrap();
/// assert_eq!(encoded, serde_json::to_string(&owned).unwrap());
/// ```
#[must_use]
pub struct ValueWirePayloadRefV1<'a> {
    /// Borrowed source validated for V1 serialization.
    source: WireSourceRef<'a>,
}

impl<'a> ValueWirePayloadRefV1<'a> {
    /// Borrows a scalar after validating the V1 wire representation
    /// constraints.
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
    /// Returns [`ValueWireEncodeError::NonFiniteFloat`] for a non-finite float,
    /// [`ValueWireEncodeError::BigDecimalScaleTooLarge`] for an unsupported
    /// decimal scale, or [`ValueWireEncodeError::InstantYearOutOfRange`] when
    /// an Instant's UTC year is outside 0 through 9999.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        validate_value(value)?;
        Ok(Self {
            source: WireSourceRef::Scalar(value),
        })
    }

    /// Borrows a collection after validating the V1 wire representation
    /// constraints.
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
    /// Returns [`ValueWireEncodeError::NonFiniteFloat`] for a non-finite float
    /// element, [`ValueWireEncodeError::BigDecimalScaleTooLarge`] for an
    /// unsupported decimal scale, or
    /// [`ValueWireEncodeError::InstantYearOutOfRange`] when an Instant's
    /// UTC year is outside 0 through 9999.
    pub fn from_values(values: &'a MultiValues) -> Result<Self, ValueWireEncodeError> {
        validate_values(values)?;
        Ok(Self {
            source: WireSourceRef::Collection(values),
        })
    }

    /// Borrows an explicit shape after validating the V1 wire representation
    /// constraints.
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
    /// Returns [`ValueWireEncodeError::NonFiniteFloat`] for a non-finite float,
    /// [`ValueWireEncodeError::BigDecimalScaleTooLarge`] for an unsupported
    /// decimal scale, or [`ValueWireEncodeError::InstantYearOutOfRange`] when
    /// an Instant's UTC year is outside 0 through 9999.
    pub fn from_container(value: &'a ValueContainer) -> Result<Self, ValueWireEncodeError> {
        match value {
            ValueContainer::Scalar(value) => Self::from_value(value),
            ValueContainer::Collection(values) => Self::from_values(values),
        }
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
    /// Runs a conservative payload preflight before encoding, then enforces
    /// `limits` authoritatively while encoding. The preflight may report a
    /// lower-bound budget error and does not bound every preparation
    /// allocation.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when the payload preflight or final
    /// encoding exceeds `limits`, or Serde rejects the payload.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_vec_with_limits(&self, limits: JsonEncodeLimits) -> Result<Vec<u8>, ValueWireEncodeError> {
        self.check_encode_preflight(limits)?;
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
    /// Runs a conservative payload preflight before encoding, then enforces
    /// `limits` authoritatively while encoding. The preflight may report a
    /// lower-bound budget error and does not bound every preparation
    /// allocation.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when the payload preflight or final
    /// encoding exceeds `limits`, Serde rejects the payload, or `writer`
    /// rejects output.
    #[cfg(feature = "json")]
    #[inline]
    pub fn to_json_writer_with_limits<W>(
        &self,
        mut writer: W,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError>
    where
        W: Write,
    {
        self.check_encode_preflight(limits)?;
        let session = JsonEncodeSession::from_limits(limits);
        JsonEncoder::new(session)
            .write_buffered(&mut writer, self)
            .map_err(ValueWireEncodeError::from)
    }

    /// Returns the borrowed internal shape used by V1 serialization.
    ///
    /// # Returns
    ///
    /// A copyable borrowed view of the validated scalar-or-collection shape.
    #[must_use]
    #[inline(always)]
    pub(in crate::value_wire) fn shape(&self) -> WireShapeRef<'a> {
        match self.source {
            WireSourceRef::Scalar(value) => WireShapeRef::Scalar(value.into()),
            WireSourceRef::Collection(values) => WireShapeRef::Collection(values.into()),
        }
    }

    /// Checks the borrowed runtime payload against conservative JSON limits.
    ///
    /// # Parameters
    ///
    /// * `limits` - JSON resource limits used to check payload preparation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the conservative payload preflight accepts the source.
    ///
    /// # Errors
    ///
    /// Returns [`ValueWireEncodeError`] when a lower-bound payload estimate
    /// exceeds `limits`. The final encoder performs the authoritative check.
    #[cfg(feature = "json")]
    pub(in crate::value_wire) fn check_encode_preflight(
        &self,
        limits: JsonEncodeLimits,
    ) -> Result<(), ValueWireEncodeError> {
        let mut checker = ValueWireEncodePreflight::new(limits);
        match self.source {
            WireSourceRef::Scalar(value) => checker.check_value(value),
            WireSourceRef::Collection(values) => checker.check_values(values),
        }
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
/// Returns [`ValueWireEncodeError::NonFiniteFloat`] for a non-finite float,
/// [`ValueWireEncodeError::BigDecimalScaleTooLarge`] for an unsupported
/// arbitrary-precision decimal scale, or
/// [`ValueWireEncodeError::InstantYearOutOfRange`] for an Instant whose UTC
/// year is outside 0 through 9999.
pub(in crate::value_wire) fn validate_value(value: &Value) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "chrono")]
    if let ValueRepr::Instant(value) = &value.repr {
        validate_instant_year(value.year())?;
    }
    #[cfg(feature = "big-decimal")]
    if let ValueRepr::BigDecimal(value) = &value.repr {
        validate_big_decimal_scale(value.as_bigint_and_scale().1)?;
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

/// Validates one collection against the V1 wire representation constraints.
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
/// Returns [`ValueWireEncodeError::NonFiniteFloat`] for a non-finite float
/// element, [`ValueWireEncodeError::BigDecimalScaleTooLarge`] for an
/// unsupported arbitrary-precision decimal scale, or
/// [`ValueWireEncodeError::InstantYearOutOfRange`] for an Instant whose UTC
/// year is outside 0 through 9999.
pub(in crate::value_wire) fn validate_values(values: &MultiValues) -> Result<(), ValueWireEncodeError> {
    #[cfg(feature = "chrono")]
    if let MultiValuesRepr::Instant(values) = &values.repr {
        for value in values {
            validate_instant_year(value.year())?;
        }
    }
    #[cfg(feature = "big-decimal")]
    if let MultiValuesRepr::BigDecimal(values) = &values.repr {
        for value in values {
            validate_big_decimal_scale(value.as_bigint_and_scale().1)?;
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
