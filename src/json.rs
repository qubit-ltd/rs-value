// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection for value containers.

use std::str::FromStr;

use qubit_datatype::DataConversionError;
use qubit_datatype::DataConversionOptions;
use qubit_datatype::DataConverter;
use qubit_datatype::DataListConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;
use serde_json::Map;
use serde_json::Number;
use serde_json::Value as JsonValue;

use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
use crate::ValueError;
use crate::ValueResult;
use crate::multi_values::MultiValuesRepr;
use crate::value::ValueRepr;

/// Converts a finite float to a JSON number.
///
/// # Parameters
///
/// * `value` - Finite floating-point value to convert.
/// * `from` - Runtime type of `value` for conversion diagnostics.
///
/// # Returns
///
/// The corresponding JSON number.
///
/// # Errors
///
/// Returns [`DataConversionError`] when `value` is NaN or infinite.
fn finite_float64(value: f64, from: DataType) -> Result<JsonValue, DataConversionError> {
    Number::from_f64(value)
        .map(JsonValue::Number)
        .ok_or(DataConversionError::invalid(
            from,
            DataType::Json,
            InvalidValueReason::NonFinite,
        ))
}

fn finite_float32(value: f32, from: DataType) -> Result<JsonValue, DataConversionError> {
    // Use f32 display output as input here to keep float32 textual precision
    // stable. Converting through `f64` first can emit a longer/altered decimal
    // representation, which changes natural JSON bytes for the same `f32`
    // value.
    Number::from_str(&value.to_string())
        .map(JsonValue::Number)
        .map_err(|_| {
            DataConversionError::invalid(from, DataType::Json, InvalidValueReason::NonFinite)
        })
}

macro_rules! scalar_to_json {
    (json_bool, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::Bool(*$value))
    };
    (json_number, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::from(*$value))
    };
    (json_float32, $value:expr, $from:expr, $options:expr) => {
        finite_float32(*$value, $from)
    };
    (json_float64, $value:expr, $from:expr, $options:expr) => {
        finite_float64(*$value as f64, $from)
    };
    (json_string, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::String($value.to_string()))
    };
    (json_duration, $value:expr, $from:expr, $options:expr) => {
        DataConverter::from(*$value)
            .to_with::<String>($options)
            .map(JsonValue::String)
    };
    (json_object, $value:expr, $from:expr, $options:expr) => {{
        let mut entries: Vec<_> = $value.iter().collect();
        entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        let mut object = Map::with_capacity(entries.len());
        for (key, value) in entries {
            object.insert(key.clone(), JsonValue::String(value.clone()));
        }
        Ok(JsonValue::Object(object))
    }};
    (json_identity, $value:expr, $from:expr, $options:expr) => {
        Ok(crate::wire::json::canonicalize_json_value($value))
    };
}

macro_rules! value_to_json_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {{
        let result: Result<JsonValue, DataConversionError> = match &$value.repr {
            ValueRepr::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* ValueRepr::$variant(value) => {
                scalar_to_json!($json_class, value, $data_type, $options)
            },)+
        };
        result.map_err(ValueError::from)
    }};
}

/// Projects a concrete vector according to the natural JSON cardinality rule.
///
/// # Type Parameters
///
/// * `T` - Runtime element type.
/// * `F` - Projection from an element to a JSON value.
///
/// # Parameters
///
/// * `values` - Concrete values to project.
/// * `project` - Element projection that can report conversion failures.
///
/// # Returns
///
/// A JSON array containing the projected values in their original order.
///
/// # Errors
///
/// Returns [`ValueError`] with a [`DataListConversionError`] identifying the
/// first source index whose projection fails.
fn collection_to_json<T, F>(values: &[T], mut project: F) -> ValueResult<JsonValue>
where
    F: FnMut(&T) -> Result<JsonValue, DataConversionError>,
{
    let mut projected = Vec::with_capacity(values.len());
    for (source_index, value) in values.iter().enumerate() {
        match project(value) {
            Ok(value) => projected.push(value),
            Err(source) => {
                return Err(DataListConversionError::new(source_index, source).into());
            }
        }
    }

    Ok(JsonValue::Array(projected))
}

macro_rules! multi_values_to_json_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* MultiValuesRepr::$variant(values) => {
                collection_to_json(values, |value| {
                    scalar_to_json!($json_class, value, $data_type, $options)
                })
            },)+
        }
    };
}

impl Value {
    /// Projects this typed value to its natural JSON representation.
    ///
    /// This differs from the tagged [`crate::ValueWireV1`] representation: for
    /// example,
    /// `Value::Int32(42)` projects to the JSON number `42`.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this value.
    ///
    /// # Errors
    ///
    /// Returns a structured conversion error for values JSON cannot represent,
    /// including non-finite floating-point values and inexact durations.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<JsonValue> {
        self.to_json_value_with(DataConversionOptions::default_ref())
    }

    /// Projects this typed value using explicit conversion options.
    ///
    /// # Parameters
    ///
    /// * `options` - Controls duration units and precision-loss behavior.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this value.
    ///
    /// # Errors
    ///
    /// Returns a structured conversion error when JSON projection or duration
    /// formatting violates the requested options.
    pub fn to_json_value_with(&self, options: &DataConversionOptions) -> ValueResult<JsonValue> {
        for_each_value_type!(value_to_json_match, self, options)
    }
}

impl MultiValues {
    /// Projects this collection to its natural JSON representation.
    ///
    /// Unset is `null`; every concrete collection is an array, including empty
    /// and one-item collections.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this collection.
    ///
    /// # Errors
    ///
    /// Returns a list conversion error containing the zero-based source index
    /// when an item cannot be represented as JSON.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<JsonValue> {
        self.to_json_value_with(DataConversionOptions::default_ref())
    }

    /// Projects this collection using explicit conversion options.
    ///
    /// # Parameters
    ///
    /// * `options` - Controls duration units and precision-loss behavior.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this collection.
    ///
    /// # Errors
    ///
    /// Returns an indexed list conversion error when an item cannot be
    /// represented under the requested options.
    pub fn to_json_value_with(&self, options: &DataConversionOptions) -> ValueResult<JsonValue> {
        for_each_value_type!(multi_values_to_json_match, self, options)
    }
}

impl ValueContainer {
    /// Projects this container while preserving concrete collection shape.
    ///
    /// Scalar storage uses the natural scalar projection; concrete collection
    /// storage always uses a JSON array.
    ///
    /// # Returns
    ///
    /// The natural JSON representation, except scalar and collection unset
    /// values both project to `null`.
    ///
    /// # Errors
    ///
    /// Returns the same structured projection error as the contained value.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<JsonValue> {
        self.to_json_value_with(DataConversionOptions::default_ref())
    }

    /// Projects this container using explicit conversion options.
    ///
    /// # Parameters
    ///
    /// * `options` - Controls duration units and precision-loss behavior.
    ///
    /// # Returns
    ///
    /// The natural JSON representation, except scalar and collection unset
    /// values both project to `null`.
    ///
    /// # Errors
    ///
    /// Returns the same structured projection error as the contained value.
    #[inline(always)]
    pub fn to_json_value_with(&self, options: &DataConversionOptions) -> ValueResult<JsonValue> {
        match self {
            Self::Scalar(value) => value.to_json_value_with(options),
            Self::Collection(values) => values.to_json_value_with(options),
        }
    }
}
