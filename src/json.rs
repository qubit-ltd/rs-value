// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection for value containers.

use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConverter,
    DataListConversionError,
    DataType,
    InvalidValueReason,
};
use serde_json::{
    Map,
    Number,
    Value as JsonValue,
};

use crate::{
    MultiValues,
    Value,
    ValueContainer,
    ValueError,
    ValueResult,
};

/// Converts a finite float to a JSON number.
fn finite_float(
    value: f64,
    from: DataType,
) -> Result<JsonValue, DataConversionError> {
    Number::from_f64(value).map(JsonValue::Number).ok_or(
        DataConversionError::invalid(
            from,
            DataType::Json,
            InvalidValueReason::NonFinite,
        ),
    )
}

macro_rules! scalar_to_json {
    (json_bool, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::Bool(*$value))
    };
    (json_number, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::from(*$value))
    };
    (json_float, $value:expr, $from:expr, $options:expr) => {
        finite_float(*$value as f64, $from)
    };
    (json_string, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::String($value.to_string()))
    };
    (json_duration, $value:expr, $from:expr, $options:expr) => {
        DataConverter::from(*$value)
            .to_with::<String>($options)
            .map(JsonValue::String)
    };
    (json_object, $value:expr, $from:expr, $options:expr) => {
        Ok(JsonValue::Object(
            $value
                .iter()
                .map(|(key, value)| {
                    (key.clone(), JsonValue::String(value.clone()))
                })
                .collect::<Map<String, JsonValue>>(),
        ))
    };
    (json_identity, $value:expr, $from:expr, $options:expr) => {
        Ok($value.clone())
    };
}

macro_rules! value_to_json_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {{
        let result: Result<JsonValue, DataConversionError> = match $value {
            Value::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* Value::$variant(value) => {
                scalar_to_json!($json_class, value, $data_type, $options)
            },)+
        };
        result.map_err(ValueError::from)
    }};
}

/// Projects a concrete vector according to the natural JSON cardinality rule.
fn collection_to_json<T, F>(
    values: &[T],
    mut project: F,
) -> ValueResult<JsonValue>
where
    F: FnMut(&T) -> Result<JsonValue, DataConversionError>,
{
    let mut projected = Vec::with_capacity(values.len());
    for (source_index, value) in values.iter().enumerate() {
        match project(value) {
            Ok(value) => projected.push(value),
            Err(source) => {
                return Err(
                    DataListConversionError::new(source_index, source).into()
                );
            }
        }
    }

    Ok(JsonValue::Array(projected))
}

macro_rules! multi_values_to_json_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* MultiValues::$variant(values) => {
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
    /// This differs from the tagged [`serde::Serialize`] representation: for
    /// example,
    /// `Value::Int32(42)` projects to the JSON number `42`.
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
    /// # Arguments
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
    pub fn to_json_value_with(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<JsonValue> {
        for_each_value_type!(value_to_json_match, self, options)
    }
}

impl MultiValues {
    /// Projects this collection to its natural JSON representation.
    ///
    /// Unset is `null`; every concrete collection is an array, including empty
    /// and one-item collections.
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
    /// # Arguments
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
    pub fn to_json_value_with(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<JsonValue> {
        for_each_value_type!(multi_values_to_json_match, self, options)
    }
}

impl ValueContainer {
    /// Projects this container while preserving its explicit shape.
    ///
    /// Scalar storage uses the natural scalar projection; concrete collection
    /// storage always uses a JSON array.
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
    /// # Arguments
    ///
    /// * `options` - Controls duration units and precision-loss behavior.
    ///
    /// # Returns
    ///
    /// The natural JSON representation while preserving container shape.
    ///
    /// # Errors
    ///
    /// Returns the same structured projection error as the contained value.
    #[inline(always)]
    pub fn to_json_value_with(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<JsonValue> {
        match self {
            Self::Scalar(value) => value.to_json_value_with(options),
            Self::Collection(values) => values.to_json_value_with(options),
        }
    }
}
