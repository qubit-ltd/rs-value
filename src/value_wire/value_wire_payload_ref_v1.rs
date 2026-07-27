// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload serialization.

use serde::{Serialize, Serializer};

use crate::{MultiValues, Value, ValueContainer};

use super::{ValueWireEncodeError, WireShapeRef};

/// Borrowed unversioned V1 payload for serialization without cloning.
#[must_use]
pub enum ValueWirePayloadRefV1<'a> {
    /// A scalar value.
    Scalar(&'a Value),
    /// A homogeneous collection.
    Collection(&'a MultiValues),
    /// A value with an explicit scalar-or-collection shape.
    Container(&'a ValueContainer),
}

impl<'a> ValueWirePayloadRefV1<'a> {
    /// Borrows a scalar after validating V1's finite-float invariant.
    pub fn from_value(value: &'a Value) -> Result<Self, ValueWireEncodeError> {
        validate_value(value)?;
        Ok(Self::Scalar(value))
    }

    /// Borrows a collection after validating V1's finite-float invariant.
    pub fn from_values(values: &'a MultiValues) -> Result<Self, ValueWireEncodeError> {
        validate_values(values)?;
        Ok(Self::Collection(values))
    }

    /// Borrows an explicit shape after validating V1's finite-float invariant.
    pub fn from_container(value: &'a ValueContainer) -> Result<Self, ValueWireEncodeError> {
        match value {
            ValueContainer::Scalar(value) => validate_value(value)?,
            ValueContainer::Collection(values) => validate_values(values)?,
        }
        Ok(Self::Container(value))
    }

    /// Returns the borrowed internal shape used by V1 serialization.
    pub(in crate::value_wire) fn shape(&self) -> WireShapeRef<'a> {
        match self {
            Self::Scalar(value) => WireShapeRef::Scalar((*value).into()),
            Self::Collection(values) => WireShapeRef::Collection((*values).into()),
            Self::Container(value) => (*value).into(),
        }
    }
}

/// Validates one scalar against V1's JSON finite-float invariant.
fn validate_value(value: &Value) -> Result<(), ValueWireEncodeError> {
    let non_finite = matches!(value, Value::Float32(value) if !value.is_finite())
        || matches!(value, Value::Float64(value) if !value.is_finite());
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat { data_type: value.data_type() });
    }
    Ok(())
}

/// Validates one collection against V1's JSON finite-float invariant.
fn validate_values(values: &MultiValues) -> Result<(), ValueWireEncodeError> {
    let non_finite = match values {
        MultiValues::Float32(values) => values.iter().any(|value| !value.is_finite()),
        MultiValues::Float64(values) => values.iter().any(|value| !value.is_finite()),
        _ => false,
    };
    if non_finite {
        return Err(ValueWireEncodeError::NonFiniteFloat { data_type: values.data_type() });
    }
    Ok(())
}

impl<'a> TryFrom<&'a Value> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a scalar.
    fn try_from(value: &'a Value) -> Result<Self, Self::Error> { Self::from_value(value) }
}

impl<'a> TryFrom<&'a MultiValues> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates a collection.
    fn try_from(values: &'a MultiValues) -> Result<Self, Self::Error> { Self::from_values(values) }
}

impl<'a> TryFrom<&'a ValueContainer> for ValueWirePayloadRefV1<'a> {
    type Error = ValueWireEncodeError;
    /// Borrows and validates an explicit shape.
    fn try_from(value: &'a ValueContainer) -> Result<Self, Self::Error> { Self::from_container(value) }
}

impl Serialize for ValueWirePayloadRefV1<'_> {
    /// Serializes the borrowed unversioned V1 shape.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.shape().serialize(serializer)
    }
}
