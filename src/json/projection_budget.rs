// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared admission for one complete natural JSON projection.

use std::fmt::Display;
use std::fmt::Write;

use qubit_budget::BudgetedStringError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceBudget;
use qubit_budget::ResourceQuantity;
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionResource;
use qubit_datatype::ConversionSession;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;

use crate::ValueError;
use crate::ValueResult;

/// Private accounting shared by every scalar and nested node in a projection.
pub(super) struct ProjectionBudget<'a> {
    /// Per-value and cumulative limits borrowed for this operation.
    pub(super) limits: &'a ConversionLimits,
    /// Shared conversion session for duration formatting.
    pub(super) conversion: ConversionSession<'a>,
    /// Original runtime type retained in structured failures.
    pub(super) data_type: DataType,
    /// Current collection index, absent for scalars and outer shape checks.
    pub(super) source_index: Option<usize>,
    /// Remaining top-level scalar capacity.
    items: ResourceBudget<ConversionResource, u64>,
    /// Borrowed source text, including JSON strings and map keys.
    input: ResourceBudget<ConversionResource, u64>,
    /// Decoded JSON key, string and number text produced by the projection.
    output: ResourceBudget<ConversionResource, u64>,
    /// Root-inclusive structure and cumulative decoded payload accounting.
    structure: JsonValueBudget<ConversionResource, u64>,
}

impl<'a> ProjectionBudget<'a> {
    /// Creates one budget for the whole scalar or collection operation.
    pub(super) fn new(data_type: DataType, policy: &'a ConversionPolicy, limits: &'a ConversionLimits) -> Self {
        let operation = limits.operation();
        let structure_limits = limits
            .structured()
            .value()
            .structure_limits()
            .to_builder()
            .nodes_limit(*operation.structured_nodes_limit())
            .build();
        let structure = JsonValueBudget::new(
            JsonValueLimits::builder()
                .structure_limits(structure_limits)
                .payload_bytes_limit(*operation.structured_payload_bytes_limit())
                .build(),
        );
        Self {
            limits,
            conversion: ConversionSession::new(policy, limits),
            data_type,
            source_index: None,
            items: ResourceBudget::from_limit(*operation.items_limit()),
            input: ResourceBudget::from_limit(*operation.input_bytes_limit()),
            output: ResourceBudget::from_limit(*operation.output_bytes_limit()),
            structure,
        }
    }

    /// Binds a native budget failure to the source type and collection index.
    pub(super) fn error(&self, source: MeasuredBudgetError<ConversionResource, u64>) -> ValueError {
        ValueError::JsonProjectionLimit {
            data_type: self.data_type,
            source_index: self.source_index,
            source,
        }
    }

    /// Admits one scalar before formatting or traversing its payload.
    pub(super) fn item(&mut self) -> ValueResult<()> {
        self.items.try_consume(1).map_err(|error| self.error(error.into()))
    }

    /// Admits one complete structure measurement before allocating output.
    pub(super) fn admit(&mut self, measurement: JsonMeasurement) -> ValueResult<()> {
        let result = {
            let mut transaction = self.structure.transaction();
            transaction.try_admit(measurement).and_then(|()| transaction.commit())
        };
        result.map_err(|error| self.error(error))
    }

    /// Accounts borrowed UTF-8 input before inspecting or copying it.
    pub(super) fn input(&mut self, text: &str) -> ValueResult<()> {
        self.input
            .try_consume_usize(text.len())
            .map_err(|error| self.error(error))
    }

    /// Accounts a borrowed string or key before the materialization pass.
    pub(super) fn text(&mut self, text: &str, depth: usize, key: bool) -> ValueResult<()> {
        self.input(text)?;
        if !key {
            let amount = u64::try_from_usize(text.len()).map_err(|source| {
                self.error(MeasuredBudgetError::quantity(
                    ConversionResource::StructuredTextBytes,
                    source,
                ))
            })?;
            self.limits
                .structured()
                .max_text_bytes_limit()
                .check(amount)
                .map_err(|error| self.error(error.into()))?;
        }
        let measurement = if key {
            JsonMeasurement::Key { bytes: text.len() }
        } else {
            JsonMeasurement::String {
                depth,
                bytes: text.len(),
            }
        };
        self.admit(measurement)?;
        self.output
            .try_consume_usize(text.len())
            .map_err(|error| self.error(error))
    }

    /// Measures formatted text with a bounded temporary buffer before final
    /// output.
    ///
    /// The temporary renderer stops at the smallest remaining payload bound.
    /// `number` selects numeric JSON measurement instead of string measurement.
    pub(super) fn display<T: Display + ?Sized>(&mut self, value: &T, depth: usize, number: bool) -> ValueResult<()> {
        // Reject unavailable depth/node capacity before invoking Display.
        let probe = if number {
            JsonMeasurement::Number { depth, bytes: 0 }
        } else {
            JsonMeasurement::String { depth, bytes: 0 }
        };
        let result = self.structure.transaction().try_admit(probe);
        result.map_err(|error| self.error(error))?;
        let mut rendering = ResourceBudget::from_limit(*self.limits.operation().output_bytes_limit());
        rendering
            .try_consume(self.output.used())
            .map_err(|error| self.error(error.into()))?;
        let mut payload = ResourceBudget::from_limit(*self.limits.operation().structured_payload_bytes_limit());
        payload
            .try_consume(self.structure.used_payload_bytes().unwrap_or(0))
            .map_err(|error| self.error(error.into()))?;
        let text_limit = ResourceBudget::from_limit(*self.limits.structured().max_text_bytes_limit());
        for candidate in [payload, text_limit] {
            if candidate.remaining() < rendering.remaining() {
                rendering = candidate;
            }
        }
        let text = rendering
            .try_write_string(|writer| write!(writer.as_fmt(), "{value}"))
            .map_err(|error| match error {
                BudgetedStringError::Budget(error) => self.error(error.into()),
                BudgetedStringError::Quantity { resource, source } => {
                    self.error(MeasuredBudgetError::quantity(resource, source))
                }
                _ => ValueError::Conversion(DataConversionError::invalid(
                    self.data_type,
                    DataType::Json,
                    InvalidValueReason::OutOfRange,
                )),
            })?;
        self.admit(if number {
            JsonMeasurement::Number {
                depth,
                bytes: text.len(),
            }
        } else {
            JsonMeasurement::String {
                depth,
                bytes: text.len(),
            }
        })?;
        self.output
            .try_consume_usize(text.len())
            .map_err(|error| self.error(error))
    }
}
