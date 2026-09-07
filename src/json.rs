// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection for value containers.

use std::str::FromStr;

use qubit_budget::json::JsonMeasurement;

mod json_children;
mod projection_budget;

use json_children::JsonChildren;
use projection_budget::ProjectionBudget;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataConversionError;
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

/// Converts a finite 32-bit float to a JSON number without widening its text.
///
/// # Parameters
///
/// * `value` - Finite floating-point value to convert.
/// * `from` - Runtime type of `value` for conversion diagnostics.
///
/// # Returns
///
/// The corresponding JSON number with the source `f32` textual precision.
///
/// # Errors
///
/// Returns [`DataConversionError`] when `value` is NaN or infinite.
fn finite_float32(value: f32, from: DataType) -> Result<JsonValue, DataConversionError> {
    // Use f32 display output as input here to keep float32 textual precision
    // stable. Converting through `f64` first can emit a longer/altered decimal
    // representation, which changes natural JSON bytes for the same `f32`
    // value.
    Number::from_str(&value.to_string())
        .map(JsonValue::Number)
        .map_err(|_| DataConversionError::invalid(from, DataType::Json, InvalidValueReason::NonFinite))
}

/// Projects one scalar storage payload into its natural JSON representation.
macro_rules! scalar_to_json {
    (json_bool, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        Ok(JsonValue::Bool(*$value))
    };
    (json_number, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        Ok(JsonValue::from(*$value))
    };
    (json_float32, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        finite_float32(*$value, $from)
    };
    (json_float64, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        finite_float64(*$value as f64, $from)
    };
    (json_string, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        Ok(JsonValue::String($value.to_string()))
    };
    (json_duration, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        DataConverter::from(*$value)
            .to_with::<String>($policy, $limits)
            .map(JsonValue::String)
    };
    (json_object, $value:expr, $from:expr, $policy:expr, $limits:expr) => {{
        let mut entries: Vec<_> = $value.iter().collect();
        entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        let mut object = Map::with_capacity(entries.len());
        for (key, value) in entries {
            object.insert(key.clone(), JsonValue::String(value.clone()));
        }
        Ok(JsonValue::Object(object))
    }};
    (json_identity, $value:expr, $from:expr, $policy:expr, $limits:expr) => {
        Ok(crate::wire::json::canonicalize_json_value($value))
    };
}

/// Expands the shared value table into a natural JSON projection match.
macro_rules! value_to_json_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {{
        let result: Result<JsonValue, DataConversionError> = match &$value.repr {
            ValueRepr::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* ValueRepr::$variant(value) => {
                scalar_to_json!($json_class, value, $data_type, $policy, $limits)
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

/// Expands the shared value table into a collection JSON projection match.
macro_rules! multi_values_to_json_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(_) => Ok(JsonValue::Null),
            $($(#[$cfg])* MultiValuesRepr::$variant(values) => {
                collection_to_json(values, |value| {
                    scalar_to_json!($json_class, value, $data_type, $policy, $limits)
                })
            },)+
        }
    };
}

/// Checks source big-number limits before decimal formatting can allocate.
macro_rules! check_projection_number {
    (BigInteger, $value:expr, $budget:expr) => {
        $budget
            .limits
            .numeric()
            .big_integer()
            .check($value)
            .map_err(|error| $budget.error(error))?
    };
    (BigDecimal, $value:expr, $budget:expr) => {
        $budget
            .limits
            .numeric()
            .big_decimal()
            .check($value)
            .map_err(|error| $budget.error(error))?
    };
    ($variant:ident, $value:expr, $budget:expr) => {};
}

/// Admits one projected payload while preserving natural JSON type semantics.
macro_rules! admit_projection {
    (json_bool, $value:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let _ = $value;
        $budget.admit(JsonMeasurement::Boolean { depth: $depth })
    }};
    (json_number, $value:expr, $from:expr, $budget:expr, $depth:expr) => {
        $budget.display($value, $depth, true)
    };
    (json_float32, $value:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let projected = finite_float32(*$value, $from)?;
        $budget.display(&projected, $depth, true)
    }};
    (json_float64, $value:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let projected = finite_float64(*$value, $from)?;
        $budget.display(&projected, $depth, true)
    }};
    (json_string, $value:expr, $from:expr, $budget:expr, $depth:expr) => {
        $budget.display($value, $depth, false)
    };
    (json_duration, $value:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let text = DataConverter::from(*$value).to_in::<String>(&mut $budget.conversion)?;
        $budget.display(&text, $depth, false)
    }};
    (json_object, $value:expr, $from:expr, $budget:expr, $depth:expr) => {{
        $budget.admit(JsonMeasurement::Object {
            depth: $depth,
            entries: $value.len(),
        })?;
        for (key, value) in $value {
            $budget.text(key, $depth, true)?;
            $budget.text(value, $depth.saturating_add(1), false)?;
        }
        Ok(())
    }};
    (json_identity, $value:expr, $from:expr, $budget:expr, $depth:expr) => {
        admit_json($value, $depth, &mut $budget)
    };
}

/// Admits one scalar, charging original String bytes before formatting.
macro_rules! admit_scalar_match {
    ($value:expr, $budget:expr, $depth:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {{
        $budget.item()?;
        if let ValueRepr::String(text) = &$value.repr { $budget.input(text)?; }
        match &$value.repr {
            ValueRepr::Unset(_) => $budget.admit(JsonMeasurement::Null { depth: $depth }),
            $($(#[$cfg])* ValueRepr::$variant(stored) => {
                let value = value_storage_ref!($variant, stored);
                check_projection_number!($variant, value, $budget);
                admit_projection!($json_class, value, $data_type, $budget, $depth)
            },)+
        }
    }};
}

/// Charges original string bytes once at the corresponding element index.
macro_rules! admit_projection_input {
    (String, $value:expr, $budget:expr) => {
        $budget.input($value)?;
    };
    ($variant:ident, $value:expr, $budget:expr) => {};
}

/// Admits the explicit array shape and every indexed scalar before allocation.
macro_rules! admit_collection_match {
    ($value:expr, $budget:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {{
        match &$value.repr {
            MultiValuesRepr::Unset(_) => {
                $budget.item()?;
                $budget.admit(JsonMeasurement::Null { depth: 1 })
            },
            $($(#[$cfg])* MultiValuesRepr::$variant(values) => {
                $budget.admit(JsonMeasurement::Array { depth: 1, items: values.len() })?;
                for (index, value) in values.iter().enumerate() {
                    $budget.source_index = Some(index);
                    $budget.item()?;
                    admit_projection_input!($variant, value, $budget);
                    check_projection_number!($variant, value, $budget);
                    let mut admit = || -> ValueResult<()> {
                        admit_projection!($json_class, value, $data_type, $budget, 2_usize)
                    };
                    let result = admit();
                    result.map_err(|error| match error {
                        ValueError::Conversion(source) => ValueError::from(DataListConversionError::new(index, source)),
                        error => error,
                    })?;
                }
                Ok(())
            },)+
        }
    }};
}

/// Traverses nested JSON iteratively, charging keys and leaf text before
/// cloning.
fn admit_json(value: &JsonValue, depth: usize, budget: &mut ProjectionBudget<'_>) -> ValueResult<()> {
    let mut frames = Vec::<JsonChildren<'_>>::new();
    let mut next = Some((None, value, depth));
    while let Some((key, value, depth)) = next.take() {
        if let Some(key) = key {
            budget.text(key, depth, true)?;
        }
        match value {
            JsonValue::Null => budget.admit(JsonMeasurement::Null { depth })?,
            JsonValue::Bool(_) => budget.admit(JsonMeasurement::Boolean { depth })?,
            JsonValue::Number(value) => budget.display(value, depth, true)?,
            JsonValue::String(value) => budget.text(value, depth, false)?,
            JsonValue::Array(values) => {
                budget.admit(JsonMeasurement::Array {
                    depth,
                    items: values.len(),
                })?;
                frames.push(JsonChildren::Array(values.iter(), depth.saturating_add(1)));
            }
            JsonValue::Object(values) => {
                budget.admit(JsonMeasurement::Object {
                    depth,
                    entries: values.len(),
                })?;
                frames.push(JsonChildren::Object(values.iter(), depth.saturating_add(1)));
            }
        }
        while let Some(frame) = frames.last_mut() {
            if let Some(child) = frame.next() {
                next = Some(child);
                break;
            }
            frames.pop();
        }
    }
    Ok(())
}

/// Projects a scalar value using explicit conversion policy and limits.
pub(crate) fn value_to_json_value_with(
    value: &Value,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<JsonValue> {
    let mut budget = ProjectionBudget::new(value.data_type(), policy, limits);
    for_each_value_type!(admit_scalar_match, value, budget, 1_usize)?;
    for_each_value_type!(value_to_json_match, value, policy, limits)
}

/// Projects a collection using explicit conversion policy and limits.
pub(crate) fn multi_values_to_json_value_with(
    values: &MultiValues,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<JsonValue> {
    let mut budget = ProjectionBudget::new(values.data_type(), policy, limits);
    for_each_value_type!(admit_collection_match, values, budget)?;
    for_each_value_type!(multi_values_to_json_match, values, policy, limits)
}

/// Projects a scalar-or-collection container while preserving its shape.
pub(crate) fn value_container_to_json_value_with(
    container: &ValueContainer,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<JsonValue> {
    match container {
        ValueContainer::Scalar(value) => value_to_json_value_with(value, policy, limits),
        ValueContainer::Collection(values) => multi_values_to_json_value_with(values, policy, limits),
    }
}
