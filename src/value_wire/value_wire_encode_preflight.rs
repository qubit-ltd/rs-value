// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Conservative preparation checks for bounded V1 JSON encoding.

use std::time::Duration;

use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::Observation;
use qubit_budget::QuantityConversionError;
use qubit_budget::QuantityMeasurement;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeLimits as JsonEncodeLimitsU64;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueLimits;

use super::internal::WireDataTypeV1;
use super::internal::WirePreflightStrategy;
use crate::MultiValues;
use crate::Value;
use crate::ValueContainer;
use crate::ValueRef;

mod internal;

use self::internal::json_length_writer::JsonLengthWriter;
use self::internal::wire_scalar_measurement::decimal_len;
use self::internal::wire_scalar_measurement::json_float32_len;
use self::internal::wire_scalar_measurement::json_float64_len;
use self::internal::wire_scalar_measurement::unsigned_decimal_len;

macro_rules! define_value_ref_preflight_strategy {
    (
        $($arg:expr),*;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $_data_type:expr,
                $_materialization:ident,
                $_json_class:ident,
                $_number_projection:ident,
                $_value_doc:literal,
                $_multi_doc:literal,
                [$($scalar_attr:meta),*],
                [$($collection_attr:meta),*],
                $_tag:literal,
                $_wire_preflight:ident
            )
        ),+ $(,)?
    ) => {
        fn value_ref_preflight_strategy(value: ValueRef<'_>) -> WirePreflightStrategy {
            match value {
                ValueRef::Unset(_) => WirePreflightStrategy::UnsetText,
                $(
                    $(#[$cfg])*
                    ValueRef::$variant(_) => WireDataTypeV1::$variant.preflight_strategy(),
                )+
            }
        }
    };
}

for_each_value_type!(define_value_ref_preflight_strategy);

/// Performs conservative resource checks before Wire V1 sorting and formatting.
///
/// The checker accumulates conservative lower bounds across successful calls.
/// Each public check is atomic: if it returns an error, all counters are
/// restored to their values before that call. The checker does not serialize,
/// retain a sorting index, or replace the authoritative final
/// `JsonEncodeSession` checks.
///
/// # Examples
///
/// ```
/// use qubit_budget::MeasuredBudgetError;
/// use qubit_budget::json::JsonEncodeLimits;
/// use qubit_budget::json::JsonResource;
/// use qubit_value::Value;
/// use qubit_value::ValueWireEncodePreflight;
///
/// # fn main() -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
/// let limits = JsonEncodeLimits::builder().max_nodes(2_usize).build();
/// let mut preflight = ValueWireEncodePreflight::new(limits);
/// preflight.check_value(&Value::from(1_i32))?;
/// preflight.check_value(&Value::from(2_i32))?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ValueWireEncodePreflight {
    limits: JsonEncodeLimits,
    nodes: usize,
    payload_bytes: usize,
    output_bytes: usize,
}

/// Converts a `u64` limit to the current platform's native budget quantity.
///
/// Values outside the `usize` range saturate at [`usize::MAX`].
fn saturating_u64_to_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

impl ValueWireEncodePreflight {
    /// Creates a checker with zero accumulated usage from one limit profile.
    #[must_use]
    pub fn new(limits: JsonEncodeLimits) -> Self {
        Self {
            limits,
            nodes: 0,
            payload_bytes: 0,
            output_bytes: 0,
        }
    }

    /// Creates a checker with zero accumulated usage for value-only limits.
    ///
    /// The outer output budget remains unconfigured because another protocol
    /// envelope is expected to account for it.
    #[must_use]
    pub fn new_value_limits(limits: JsonValueLimits) -> Self {
        let mut checker = Self::new(JsonEncodeLimits::new());
        checker.limits = JsonEncodeLimits::builder().value_limits(limits).build();
        checker
    }

    /// Creates a checker from the `u64` profile used by configuration wire
    /// limits.
    ///
    /// Values greater than [`usize::MAX`] on the current platform saturate at
    /// [`usize::MAX`] instead of wrapping or truncating.
    #[must_use]
    pub fn new_u64_limits(limits: JsonEncodeLimitsU64<JsonResource, u64>) -> Self {
        let value = limits.value_limits();
        let mut builder = JsonEncodeLimits::builder();
        if let Some(limit) = limits.max_output_bytes() {
            builder = builder.max_output_bytes(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_depth() {
            builder = builder.max_depth(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_nodes() {
            builder = builder.max_nodes(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_sequence_items() {
            builder = builder.max_sequence_items(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_map_entries() {
            builder = builder.max_map_entries(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_key_bytes() {
            builder = builder.max_key_bytes(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_string_bytes() {
            builder = builder.max_string_bytes(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_number_bytes() {
            builder = builder.max_number_bytes(saturating_u64_to_usize(limit));
        }
        if let Some(limit) = value.max_payload_bytes() {
            builder = builder.max_payload_bytes(saturating_u64_to_usize(limit));
        }
        Self::new(builder.build())
    }

    /// Checks and accumulates one scalar value at the root of a payload.
    ///
    /// Returns the first exceeded JSON resource limit. If checking fails, the
    /// accumulated state is restored to its value before this call.
    pub fn check_value(&mut self, value: &Value) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.transaction(|checker| checker.check_value_at(value, 1))
    }

    /// Checks and accumulates one homogeneous collection at the payload root.
    ///
    /// Returns the first exceeded JSON resource limit. If checking fails, the
    /// accumulated state is restored to its value before this call.
    pub fn check_values(&mut self, values: &MultiValues) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.transaction(|checker| checker.check_values_at(values, 1))
    }

    /// Checks and accumulates an explicit scalar-or-collection payload.
    ///
    /// Returns the first exceeded JSON resource limit. If checking fails, the
    /// accumulated state is restored to its value before this call.
    pub fn check_container(&mut self, value: &ValueContainer) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.transaction(|checker| match value {
            ValueContainer::Scalar(value) => checker.check_value_at(value, 1),
            ValueContainer::Collection(values) => checker.check_values_at(values, 1),
        })
    }

    /// Executes one public check as a transaction over cumulative counters.
    ///
    /// Successful checks retain their measurements. An error restores all
    /// counters and returns the original budget error unchanged.
    fn transaction<F>(&mut self, check: F) -> Result<(), MeasuredBudgetError<JsonResource, usize>>
    where
        F: FnOnce(&mut Self) -> Result<(), MeasuredBudgetError<JsonResource, usize>>,
    {
        let mut candidate = self.clone();
        match check(&mut candidate) {
            Ok(()) => {
                *self = candidate;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn check_value_at(&mut self, value: &Value, depth: usize) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.check_view_at(value.view(), depth)
    }

    fn check_view_at(
        &mut self,
        value: ValueRef<'_>,
        depth: usize,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        match value_ref_preflight_strategy(value) {
            WirePreflightStrategy::Boolean => match value {
                ValueRef::Bool(_) => self.admit(JsonMeasurement::Boolean { depth }, 1, 0),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::BorrowedText => match value {
                ValueRef::Char(value) => self.admit_string(depth, value.len_utf8()),
                ValueRef::String(value) => self.admit_string(depth, value.len()),
                #[cfg(feature = "url")]
                ValueRef::Url(value) => self.admit_string(depth, value.as_str().len()),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::JsonInteger => match value {
                ValueRef::Int8(value) => self.admit_number(depth, decimal_len(value as i128)),
                ValueRef::Int16(value) => self.admit_number(depth, decimal_len(value as i128)),
                ValueRef::Int32(value) => self.admit_number(depth, decimal_len(value as i128)),
                ValueRef::Int64(value) => self.admit_number(depth, decimal_len(value as i128)),
                ValueRef::UInt8(value) => self.admit_number(depth, unsigned_decimal_len(value as u128)),
                ValueRef::UInt16(value) => self.admit_number(depth, unsigned_decimal_len(value as u128)),
                ValueRef::UInt32(value) => self.admit_number(depth, unsigned_decimal_len(value as u128)),
                ValueRef::UInt64(value) => self.admit_number(depth, unsigned_decimal_len(value as u128)),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::DecimalText => match value {
                ValueRef::Int128(value) => self.admit_string(depth, decimal_len(value)),
                ValueRef::UInt128(value) => self.admit_string(depth, unsigned_decimal_len(value)),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::UnsetText => match value {
                ValueRef::Unset(data_type) => {
                    let tag = WireDataTypeV1::from(data_type).tag();
                    self.admit_string(depth, tag.len())
                }
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::JsonFloat => match value {
                ValueRef::Float32(value) => {
                    self.admit_number(depth, if value.is_finite() { json_float32_len(value) } else { 1 })
                }
                ValueRef::Float64(value) => {
                    self.admit_number(depth, if value.is_finite() { json_float64_len(value) } else { 1 })
                }
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::BigIntegerText => {
                #[cfg(feature = "big-integer")]
                {
                    match value {
                        ValueRef::BigInteger(value) => self.admit_string(depth, bigint_digits(value)),
                        _ => unreachable!("table strategy and ValueRef variant diverged"),
                    }
                }
                #[cfg(not(feature = "big-integer"))]
                {
                    unreachable!("big-integer strategy is unavailable without its feature")
                }
            }
            WirePreflightStrategy::DecimalObject => {
                #[cfg(feature = "big-decimal")]
                {
                    match value {
                        ValueRef::BigDecimal(value) => {
                            let (coefficient, scale) = value.as_bigint_and_scale();
                            drop(coefficient);
                            self.admit_object_with_keys(
                                depth,
                                [("coefficient", 0, true), ("scale", decimal_len(scale as i128), false)],
                            )
                        }
                        _ => unreachable!("table strategy and ValueRef variant diverged"),
                    }
                }
                #[cfg(not(feature = "big-decimal"))]
                {
                    unreachable!("big-decimal strategy is unavailable without its feature")
                }
            }
            WirePreflightStrategy::TemporalText => match value {
                #[cfg(feature = "chrono")]
                ValueRef::Date(_) | ValueRef::Time(_) | ValueRef::DateTime(_) | ValueRef::Instant(_) => {
                    self.admit_string(depth, 0)
                }
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::DurationObject => match value {
                ValueRef::Duration(value) => self.admit_duration(depth, *value),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            WirePreflightStrategy::StringMap => match value {
                ValueRef::StringMap(value) => self.check_string_map(value, depth),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
            #[cfg(feature = "json")]
            WirePreflightStrategy::JsonTree => match value {
                ValueRef::Json(value) => self.check_json(value, depth),
                _ => unreachable!("table strategy and ValueRef variant diverged"),
            },
        }
    }

    fn check_values_at(
        &mut self,
        values: &MultiValues,
        depth: usize,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.admit(
            JsonMeasurement::Array {
                depth,
                items: values.len(),
            },
            values.len().saturating_add(2),
            2,
        )?;
        let view = values.view();
        for index in 0..view.len() {
            if let Some(value) = view.get(index) {
                self.check_view_at(value, depth + 1)?;
            }
        }
        Ok(())
    }

    fn check_string_map(
        &mut self,
        map: &std::collections::HashMap<String, String>,
        depth: usize,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.admit(
            JsonMeasurement::Object {
                depth,
                entries: map.len(),
            },
            map.len().saturating_mul(2).saturating_add(2),
            2,
        )?;
        for (key, value) in map {
            self.check_point(JsonMeasurement::Key { bytes: key.len() })?;
            self.admit_string(depth + 1, value.len())?;
        }
        Ok(())
    }

    #[cfg(feature = "json")]
    fn check_json(
        &mut self,
        value: &serde_json::Value,
        depth: usize,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        enum Frame<'a> {
            Visit(&'a serde_json::Value, usize),
            Array(std::slice::Iter<'a, serde_json::Value>, usize),
            Object(serde_json::map::Iter<'a>, usize),
        }

        let mut frames = vec![Frame::Visit(value, depth)];
        while let Some(frame) = frames.pop() {
            match frame {
                Frame::Visit(value, depth) => match value {
                    serde_json::Value::Null => self.admit(JsonMeasurement::Null { depth }, 1, 0)?,
                    serde_json::Value::Bool(_) => self.admit(JsonMeasurement::Boolean { depth }, 1, 0)?,
                    serde_json::Value::Number(value) => {
                        let mut writer = JsonLengthWriter::default();
                        serde_json::to_writer(&mut writer, value).expect("JSON Number serialization cannot fail");
                        self.admit_number(depth, writer.len)?;
                    }
                    serde_json::Value::String(value) => self.admit_string(depth, value.len())?,
                    serde_json::Value::Array(values) => {
                        self.admit(
                            JsonMeasurement::Array {
                                depth,
                                items: values.len(),
                            },
                            values.len().saturating_add(2),
                            2,
                        )?;
                        frames.push(Frame::Array(values.iter(), depth + 1));
                    }
                    serde_json::Value::Object(values) => {
                        self.admit(
                            JsonMeasurement::Object {
                                depth,
                                entries: values.len(),
                            },
                            values.len().saturating_mul(2).saturating_add(2),
                            2,
                        )?;
                        frames.push(Frame::Object(values.iter(), depth + 1));
                    }
                },
                Frame::Array(mut values, depth) => {
                    if let Some(value) = values.next() {
                        frames.push(Frame::Array(values, depth));
                        frames.push(Frame::Visit(value, depth));
                    }
                }
                Frame::Object(mut values, depth) => {
                    if let Some((key, value)) = values.next() {
                        self.check_point(JsonMeasurement::Key { bytes: key.len() })?;
                        frames.push(Frame::Object(values, depth));
                        frames.push(Frame::Visit(value, depth));
                    }
                }
            }
        }
        Ok(())
    }

    fn admit_string(&mut self, depth: usize, bytes: usize) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.admit(JsonMeasurement::String { depth, bytes }, bytes.saturating_add(2), 1)
    }

    fn admit_number(&mut self, depth: usize, bytes: usize) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.admit(JsonMeasurement::Number { depth, bytes }, bytes, 1)
    }

    fn admit_duration(
        &mut self,
        depth: usize,
        value: Duration,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.admit_object_with_keys(
            depth,
            [
                ("secs", unsigned_decimal_len(value.as_secs() as u128), false),
                ("nanos", unsigned_decimal_len(value.subsec_nanos() as u128), false),
            ],
        )
    }

    fn admit_object_with_keys<const N: usize>(
        &mut self,
        depth: usize,
        fields: [(&str, usize, bool); N],
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        let output = fields
            .iter()
            .try_fold(2_usize, |total, (key, _, _)| {
                total
                    .checked_add(key.len())
                    .and_then(|total| total.checked_add(3))
                    .ok_or(())
            })
            .ok()
            .and_then(|total| total.checked_add(N.saturating_sub(1)))
            .ok_or_else(|| self.quantity_error(JsonResource::OutputBytes))?;
        self.admit(JsonMeasurement::Object { depth, entries: N }, output, 0)?;
        for (key, value, is_string) in fields {
            self.check_point(JsonMeasurement::Key { bytes: key.len() })?;
            if is_string {
                self.admit_string(depth + 1, value)?;
            } else {
                self.admit_number(depth + 1, value)?;
            }
        }
        Ok(())
    }

    fn admit(
        &mut self,
        measurement: JsonMeasurement,
        output: usize,
        payload: usize,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.check_point(measurement)?;
        self.nodes = self
            .nodes
            .checked_add(1)
            .ok_or_else(|| self.quantity_error(JsonResource::Nodes))?;
        self.payload_bytes = self
            .payload_bytes
            .checked_add(payload)
            .ok_or_else(|| self.quantity_error(JsonResource::PayloadBytes))?;
        self.output_bytes = self
            .output_bytes
            .checked_add(output)
            .ok_or_else(|| self.quantity_error(JsonResource::OutputBytes))?;
        self.check_cumulative(JsonResource::Nodes, self.nodes, self.limits.value_limits().max_nodes())?;
        self.check_cumulative(
            JsonResource::PayloadBytes,
            self.payload_bytes,
            self.limits.value_limits().max_payload_bytes(),
        )?;
        self.check_cumulative(
            JsonResource::OutputBytes,
            self.output_bytes,
            self.limits.max_output_bytes(),
        )
    }

    fn check_point(&self, measurement: JsonMeasurement) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        self.limits.value_limits().check_point(measurement)
    }

    fn check_cumulative(
        &self,
        resource: JsonResource,
        observed: usize,
        maximum: Option<usize>,
    ) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        match maximum {
            Some(maximum) if observed > maximum => Err(self.lower_bound_error(resource, observed, maximum)),
            _ => Ok(()),
        }
    }

    fn lower_bound_error(
        &self,
        resource: JsonResource,
        observed: usize,
        maximum: usize,
    ) -> MeasuredBudgetError<JsonResource, usize> {
        MeasuredBudgetError::Budget(BudgetError::LimitExceeded {
            resource,
            observed: Observation::AtLeast(observed),
            maximum,
        })
    }

    fn quantity_error(&self, resource: JsonResource) -> MeasuredBudgetError<JsonResource, usize> {
        MeasuredBudgetError::quantity(
            resource,
            QuantityConversionError::new(QuantityMeasurement::Usize(usize::MAX), "usize"),
        )
    }
}

#[cfg(any(feature = "big-integer", feature = "big-decimal"))]
fn bigint_digits(value: &num_bigint::BigInt) -> usize {
    if value.sign() == num_bigint::Sign::NoSign {
        1
    } else {
        ((value.bits().saturating_sub(1)) / 4 + 1) as usize + usize::from(value.sign() == num_bigint::Sign::Minus)
    }
}

#[cfg(test)]
mod tests {
    use std::panic::AssertUnwindSafe;

    use qubit_budget::MeasuredBudgetError;
    use qubit_budget::json::JsonEncodeLimits;
    use qubit_budget::json::JsonResource;

    use super::ValueWireEncodePreflight;
    use crate::Value;

    /// Verifies a counter overflow is reported as a quantity error.
    fn assert_overflow(error: MeasuredBudgetError<JsonResource, usize>) {
        assert!(
            matches!(error, MeasuredBudgetError::Quantity { .. }),
            "expected a quantity overflow, got {error:?}",
        );
    }

    #[test]
    fn test_check_value_reports_node_counter_overflow_and_rolls_back() {
        let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::new());
        checker.nodes = usize::MAX;

        let error = checker
            .check_value(&Value::Bool(true))
            .expect_err("the node counter cannot exceed usize::MAX");

        assert_overflow(error);
        assert_eq!(checker.nodes, usize::MAX);
        assert_eq!(checker.payload_bytes, 0);
        assert_eq!(checker.output_bytes, 0);
    }

    #[test]
    fn test_check_value_reports_payload_counter_overflow_and_rolls_back() {
        let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::new());
        checker.payload_bytes = usize::MAX;

        let error = checker
            .check_value(&Value::String("x".to_owned()))
            .expect_err("the payload counter cannot exceed usize::MAX");

        assert_overflow(error);
        assert_eq!(checker.nodes, 0);
        assert_eq!(checker.payload_bytes, usize::MAX);
        assert_eq!(checker.output_bytes, 0);
    }

    #[test]
    fn test_check_value_reports_output_counter_overflow_and_rolls_back() {
        let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::new());
        checker.output_bytes = usize::MAX;

        let error = checker
            .check_value(&Value::Bool(true))
            .expect_err("the output counter cannot exceed usize::MAX");

        assert_overflow(error);
        assert_eq!(checker.nodes, 0);
        assert_eq!(checker.payload_bytes, 0);
        assert_eq!(checker.output_bytes, usize::MAX);
    }

    #[test]
    fn test_transaction_does_not_commit_counters_when_check_panics() {
        let mut checker = ValueWireEncodePreflight::new(JsonEncodeLimits::new());
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            checker.transaction(|candidate| {
                candidate.nodes = 17;
                candidate.payload_bytes = 23;
                panic!("simulated preflight panic");
            })
        }));

        assert!(result.is_err());
        assert_eq!(checker.nodes, 0);
        assert_eq!(checker.payload_bytes, 0);
        assert_eq!(checker.output_bytes, 0);
    }
}
