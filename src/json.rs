// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection for value containers.

use std::time::Duration;

use qubit_datatype::{DataConversionError, DataListConversionError, DataType, InvalidValueReason};
use serde::Serialize;
use serde_json::{Map, Number, Value as JsonValue};

use crate::{MultiValues, Value, ValueError, ValueResult};

/// Converts a value through Serde when its table class is a JSON number.
fn serde_number<T>(value: &T, from: DataType) -> ValueResult<JsonValue>
where
    T: Serialize,
{
    serde_json::to_value(value).map_err(|_| {
        DataConversionError::InvalidValue {
            from,
            to: DataType::Json,
            reason: InvalidValueReason::Serialization {
                format: qubit_datatype::DataFormat::Json,
            },
        }
        .into()
    })
}

/// Converts a finite float to a JSON number.
fn finite_float(value: f64, from: DataType) -> ValueResult<JsonValue> {
    Number::from_f64(value)
        .map(JsonValue::Number)
        .ok_or_else(|| {
            DataConversionError::InvalidValue {
                from,
                to: DataType::Json,
                reason: InvalidValueReason::NonFinite,
            }
            .into()
        })
}

/// Formats a duration as rounded whole milliseconds with an `ms` suffix.
fn duration_string(value: &Duration) -> String {
    const NANOS_PER_MILLISECOND: u128 = 1_000_000;

    let total_nanos = value.as_nanos();
    let millis = total_nanos / NANOS_PER_MILLISECOND;
    let remainder = total_nanos % NANOS_PER_MILLISECOND;
    let rounded = millis + u128::from(remainder >= NANOS_PER_MILLISECOND / 2);
    format!("{rounded}ms")
}

macro_rules! scalar_to_json {
    (json_bool, $value:expr, $from:expr) => {
        Ok(JsonValue::Bool(*$value))
    };
    (json_number, $value:expr, $from:expr) => {
        serde_number($value, $from)
    };
    (json_float, $value:expr, $from:expr) => {
        finite_float(*$value as f64, $from)
    };
    (json_string, $value:expr, $from:expr) => {
        Ok(JsonValue::String($value.to_string()))
    };
    (json_duration, $value:expr, $from:expr) => {
        Ok(JsonValue::String(duration_string($value)))
    };
    (json_object, $value:expr, $from:expr) => {
        Ok(JsonValue::Object(
            $value
                .iter()
                .map(|(key, value)| (key.clone(), JsonValue::String(value.clone())))
                .collect::<Map<String, JsonValue>>(),
        ))
    };
    (json_identity, $value:expr, $from:expr) => {
        Ok($value.clone())
    };
}

macro_rules! value_to_json_match {
    ($value:expr; $(([$($cfg:meta),*], [$($value_attr:meta),*], [$($multi_attr:meta),*], $variant:ident, $type:ty, $data_type:expr, $ownership:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            Value::Empty(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* Value::$variant(value) => {
                scalar_to_json!($json_class, value, $data_type)
            },)+
        }
    };
}

/// Projects a concrete vector according to the natural JSON cardinality rule.
fn collection_to_json<T, F>(values: &[T], mut project: F) -> ValueResult<JsonValue>
where
    F: FnMut(&T) -> ValueResult<JsonValue>,
{
    let mut projected = Vec::with_capacity(values.len());
    for (source_index, value) in values.iter().enumerate() {
        match project(value) {
            Ok(value) => projected.push(value),
            Err(ValueError::DataConversion(source)) => {
                return Err(DataListConversionError {
                    source_index,
                    source,
                }
                .into());
            }
            Err(error) => return Err(error),
        }
    }

    match projected.len() {
        0 => Ok(JsonValue::Array(projected)),
        1 => Ok(projected.pop().expect("one projected value must exist")),
        _ => Ok(JsonValue::Array(projected)),
    }
}

macro_rules! multi_values_to_json_match {
    ($value:expr; $(([$($cfg:meta),*], [$($value_attr:meta),*], [$($multi_attr:meta),*], $variant:ident, $type:ty, $data_type:expr, $ownership:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Empty(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* MultiValues::$variant(values) => {
                collection_to_json(values, |value| {
                    scalar_to_json!($json_class, value, $data_type)
                })
            },)+
        }
    };
}

impl Value {
    /// Projects this typed value to its natural JSON representation.
    ///
    /// This differs from the tagged [`Serialize`] representation: for example,
    /// `Value::Int32(42)` projects to the JSON number `42`.
    ///
    /// # Errors
    ///
    /// Returns a structured conversion error for values JSON cannot represent,
    /// including non-finite floating-point values.
    pub fn to_json_value(&self) -> ValueResult<JsonValue> {
        for_each_value_type!(value_to_json_match, self)
    }
}

impl MultiValues {
    /// Projects this collection to its natural JSON representation.
    ///
    /// Unset is `null`, a concrete empty collection is `[]`, one item is a
    /// scalar or object, and multiple items form an array.
    ///
    /// # Errors
    ///
    /// Returns a list conversion error containing the zero-based source index
    /// when an item cannot be represented as JSON.
    pub fn to_json_value(&self) -> ValueResult<JsonValue> {
        for_each_value_type!(multi_values_to_json_match, self)
    }
}
