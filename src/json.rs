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
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataConverter;
use qubit_datatype::DataListConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;
use serde_json::Number;
use serde_json::Value as JsonValue;

use crate::MultiValues;
use crate::MultiValuesRef;
use crate::Value;
use crate::ValueContainer;
use crate::ValueError;
use crate::ValueRef;
use crate::ValueResult;

mod json_children;
mod measurement_writer;
mod prepared_projection;
mod prepared_scalar;
mod projection_budget;

use json_children::JsonChildren;
use prepared_projection::PreparedProjection;
use prepared_scalar::PreparedScalar;
use projection_budget::ProjectionBudget;

/// Converts the public big-integer type to the version used by JSON budgeting.
///
/// The conversion preserves the sign and base-2<sup>32</sup> limbs exactly, so
/// numeric limits are evaluated against the original value.
#[cfg(feature = "big-integer")]
fn big_integer_for_budget(value: &num_bigint::BigInt) -> num_bigint_budget::BigInt {
    let (sign, digits) = value.to_u32_digits();
    let sign = match sign {
        num_bigint::Sign::Minus => num_bigint_budget::Sign::Minus,
        num_bigint::Sign::NoSign => num_bigint_budget::Sign::NoSign,
        num_bigint::Sign::Plus => num_bigint_budget::Sign::Plus,
    };
    num_bigint_budget::BigInt::new(sign, digits)
}

/// Checks source big-number limits before decimal formatting can allocate.
macro_rules! check_projection_number {
    (BigInteger, $value:expr, $budget:expr) => {
        $budget
            .limits
            .numeric()
            .big_integer()
            .check(&big_integer_for_budget($value))
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

/// Classifies collections requiring a per-element rendering cache.
macro_rules! cache_projection {
    (String, $class:ident) => {
        false
    };
    ($variant:ident, json_bool) => {
        false
    };
    ($variant:ident, json_number) => {
        false
    };
    ($variant:ident, json_object) => {
        false
    };
    ($variant:ident, json_identity) => {
        false
    };
    ($variant:ident, $class:ident) => {
        true
    };
}

/// Prepares one scalar while retaining only genuinely required allocations.
macro_rules! prepare_payload {
    (String, $class:ident, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        $budget.input($value)?;
        $budget.admit_output_string($value.len(), $depth)?;
        Ok(PreparedScalar::Borrowed($view))
    }};
    ($variant:ident, json_bool, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let _ = $value;
        $budget.admit(JsonMeasurement::Boolean { depth: $depth })?;
        Ok(PreparedScalar::Borrowed($view))
    }};
    ($variant:ident, json_number, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        $budget.display(&$value, $depth, true)?;
        Ok(PreparedScalar::Borrowed($view))
    }};
    ($variant:ident, json_float32, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let number = Number::from_str(&$value.to_string())
            .map_err(|_| DataConversionError::invalid($from, DataType::Json, InvalidValueReason::NonFinite))?;
        $budget.display(&number, $depth, true)?;
        Ok(PreparedScalar::Number(number))
    }};
    ($variant:ident, json_float64, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let number = Number::from_f64($value)
            .ok_or_else(|| DataConversionError::invalid($from, DataType::Json, InvalidValueReason::NonFinite))?;
        $budget.display(&number, $depth, true)?;
        Ok(PreparedScalar::Number(number))
    }};
    ($variant:ident, json_string, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {
        $budget.format(&$value, $depth).map(PreparedScalar::Formatted)
    };
    ($variant:ident, json_duration, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        let text = DataConverter::from($value).to_in::<String>(&mut $budget.conversion)?;
        $budget.admit_output_string(text.len(), $depth)?;
        Ok(PreparedScalar::Formatted(text))
    }};
    ($variant:ident, json_object, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        $budget.admit(JsonMeasurement::Object {
            depth: $depth,
            entries: $value.len(),
        })?;
        for (key, value) in $value {
            $budget.text(key, $depth, true)?;
            $budget.text(value, $depth.saturating_add(1), false)?;
        }
        Ok(PreparedScalar::Borrowed($view))
    }};
    ($variant:ident, json_identity, $value:expr, $view:expr, $from:expr, $budget:expr, $depth:expr) => {{
        admit_json($value, $depth, $budget)?;
        Ok(PreparedScalar::Borrowed($view))
    }};
}

/// Uses the owned type table to prepare borrowed scalar payloads.
macro_rules! prepare_scalar_match {
    ($view:expr, $budget:expr, $depth:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $view {
            ValueRef::Unset(_) => {
                $budget.admit(JsonMeasurement::Null { depth: $depth })?;
                Ok(PreparedScalar::Borrowed($view))
            }
            $($(#[$cfg])* ValueRef::$variant(value) => {
                check_projection_number!($variant, value, $budget);
                prepare_payload!($variant, $json_class, value, $view, $data_type, $budget, $depth)
            },)+
        }
    };
}

/// Admits one scalar before allocating a cached rich rendering.
fn prepare_scalar<'a>(
    view: ValueRef<'a>,
    budget: &mut ProjectionBudget<'_>,
    depth: usize,
) -> ValueResult<PreparedScalar<'a>> {
    budget.item()?;
    for_each_value_type!(prepare_scalar_match, view, budget, depth)
}

/// Determines whether a homogeneous collection needs a cache before iteration.
macro_rules! collection_cache_match {
    ($view:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $view {
            MultiValuesRef::Unset(_) => false,
            $($(#[$cfg])* MultiValuesRef::$variant(_) => cache_projection!($variant, $json_class),)+
        }
    };
}

/// Admits the full collection before materializing any final JSON output.
fn prepare_collection<'a>(
    view: MultiValuesRef<'a>,
    budget: &mut ProjectionBudget<'_>,
) -> ValueResult<PreparedProjection<'a>> {
    if matches!(view, MultiValuesRef::Unset(_)) {
        budget.item()?;
        budget.admit(JsonMeasurement::Null { depth: 1 })?;
        return Ok(PreparedProjection::BorrowedCollection(view));
    }
    budget.admit(JsonMeasurement::Array {
        depth: 1,
        items: view.len(),
    })?;
    let cache = for_each_value_type!(collection_cache_match, view);
    let mut prepared = Vec::new();
    for index in 0..view.len() {
        budget.source_index = Some(index);
        let item = view.get(index).expect("index is inside the source collection");
        let scalar = prepare_scalar(item, budget, 2).map_err(|error| match error {
            ValueError::Conversion(source) => DataListConversionError::new(index, source).into(),
            error => error,
        })?;
        if cache {
            prepared.push(scalar);
        }
    }
    Ok(if cache {
        PreparedProjection::Collection(prepared)
    } else {
        PreparedProjection::BorrowedCollection(view)
    })
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

/// Projects a scalar after complete bounded preparation.
///
/// Returns conversion or resource errors before final JSON allocation.
pub(crate) fn value_to_json_value_with(
    value: &Value,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<JsonValue> {
    let mut budget = ProjectionBudget::new(value.data_type(), policy, limits);
    Ok(PreparedProjection::Scalar(prepare_scalar(value.view(), &mut budget, 1)?).materialize())
}

/// Projects a collection after complete bounded preparation, preserving its
/// shape.
pub(crate) fn multi_values_to_json_value_with(
    values: &MultiValues,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<JsonValue> {
    let mut budget = ProjectionBudget::new(values.data_type(), policy, limits);
    Ok(prepare_collection(values.view(), &mut budget)?.materialize())
}

/// Projects scalar or collection storage without conflating cardinalities.
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
