// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared admission for one complete natural JSON projection.

use std::error::Error;
use std::fmt::Display;
use std::fmt::Write;

use qubit_budget::BudgetError;
use qubit_budget::BudgetedStringError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::QuantityConversionError;
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
use qubit_datatype::DataConversionErrorKind;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;

use crate::ValueError;
use crate::ValueResult;
use crate::json::internal::MeasurementWriter;

/// Private accounting shared by every scalar and nested node in a projection.
pub(in crate::json) struct ProjectionBudget<'a> {
    /// Per-value and cumulative limits borrowed for this operation.
    pub(in crate::json) limits: &'a ConversionLimits,
    /// Shared conversion session for duration formatting.
    pub(in crate::json) conversion: ConversionSession<'a>,
    /// Original runtime type retained in structured failures.
    pub(in crate::json) data_type: DataType,
    /// Current collection index, absent for scalars and outer shape checks.
    pub(in crate::json) source_index: Option<usize>,
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
    ///
    /// # Parameters
    ///
    /// * `data_type` - Original runtime type attached to failures.
    /// * `policy` - Conversion policy borrowed for this operation.
    /// * `limits` - Resource limits borrowed for this operation.
    ///
    /// # Returns
    ///
    /// A fresh budget sharing the supplied policy and limits.
    #[must_use]
    pub(in crate::json) fn new(
        data_type: DataType,
        policy: &'a ConversionPolicy,
        limits: &'a ConversionLimits,
    ) -> Self {
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
    ///
    /// # Parameters
    ///
    /// * `source` - Native measured failure to attach to the current source
    ///   context.
    ///
    /// # Returns
    ///
    /// A projection failure preserving the resource facts.
    #[inline]
    pub(in crate::json) fn error(
        &self,
        source: MeasuredBudgetError<ConversionResource, u64>,
    ) -> ValueError {
        ValueError::JsonProjectionLimit {
            data_type: self.data_type,
            source_index: self.source_index,
            source,
        }
    }

    /// Admits one scalar before formatting or traversing its payload.
    ///
    /// # Returns
    ///
    /// `Ok(())` after charging one item.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error when scalar capacity is exhausted.
    #[inline]
    pub(in crate::json) fn item(&mut self) -> ValueResult<()> {
        self.items
            .try_consume(1)
            .map_err(|error| self.error(error.into()))
    }

    /// Admits one complete structure measurement before allocating output.
    ///
    /// # Parameters
    ///
    /// * `measurement` - Complete node or key measurement to charge
    ///   transactionally.
    ///
    /// # Returns
    ///
    /// `Ok(())` after committing the measurement.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error if the measurement cannot be admitted.
    pub(in crate::json) fn admit(&mut self, measurement: JsonMeasurement) -> ValueResult<()> {
        let result = {
            let mut transaction = self.structure.transaction();
            transaction
                .try_admit(measurement)
                .and_then(|()| transaction.commit())
        };
        result.map_err(|error| self.error(error))
    }

    /// Accounts borrowed UTF-8 input before inspecting or copying it.
    ///
    /// # Parameters
    ///
    /// * `text` - Borrowed UTF-8 source whose byte length is charged.
    ///
    /// # Returns
    ///
    /// `Ok(())` after charging source bytes.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error on byte conversion or capacity failure.
    #[inline]
    pub(in crate::json) fn input(&mut self, text: &str) -> ValueResult<()> {
        self.input
            .try_consume_usize(text.len())
            .map_err(|error| self.error(error))
    }

    /// Accounts a borrowed string or key before the materialization pass.
    ///
    /// # Parameters
    ///
    /// * `text` - Borrowed decoded string or key text.
    /// * `depth` - Root-inclusive node depth, unused for keys.
    /// * `key` - Whether the text is an object key rather than a string value.
    ///
    /// # Returns
    ///
    /// `Ok(())` after charging input, structure and output.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error on a text, quantity or cumulative limit
    /// failure. Earlier successful charges remain applied.
    pub(in crate::json) fn text(&mut self, text: &str, depth: usize, key: bool) -> ValueResult<()> {
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

    /// Admits output text without charging source input a second time.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Decoded UTF-8 output length.
    /// * `depth` - Root-inclusive depth of the string node.
    ///
    /// # Returns
    ///
    /// `Ok(())` after charging the string and output bytes.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error on a text, quantity or cumulative limit
    /// failure. Earlier successful charges remain applied.
    pub(in crate::json) fn admit_output_string(
        &mut self,
        bytes: usize,
        depth: usize,
    ) -> ValueResult<()> {
        let amount = u64::try_from_usize(bytes).map_err(|source| {
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
        self.admit(JsonMeasurement::String { depth, bytes })?;
        self.output
            .try_consume_usize(bytes)
            .map_err(|error| self.error(error))
    }

    /// Measures formatted numbers without allocating or retaining their text.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Possibly unsized value implementing `Display`.
    ///
    /// # Parameters
    ///
    /// * `value` - Value formatted only to measure its UTF-8 length.
    /// * `depth` - Root-inclusive depth of the resulting node.
    /// * `number` - Whether the resulting node is a number rather than a
    ///   string.
    ///
    /// # Returns
    ///
    /// `Ok(())` after measuring and admitting the formatted node.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error on resource rejection, or a conversion
    /// error if formatting fails.
    pub(in crate::json) fn display<T: Display + ?Sized>(
        &mut self,
        value: &T,
        depth: usize,
        number: bool,
    ) -> ValueResult<()> {
        let mut writer = MeasurementWriter::new(self.rendering_budget(depth, number)?);
        let formatted = write!(&mut writer, "{value}");
        let bytes = writer.finish().map_err(|error| self.error(error))?;
        formatted.map_err(|_| self.format_error())?;
        let bytes = usize::try_from_u64(bytes).map_err(|source| {
            self.error(MeasuredBudgetError::quantity(
                ConversionResource::OutputBytes,
                source,
            ))
        })?;
        self.admit(if number {
            JsonMeasurement::Number { depth, bytes }
        } else {
            JsonMeasurement::String { depth, bytes }
        })?;
        self.output
            .try_consume_usize(bytes)
            .map_err(|error| self.error(error))
    }

    /// Renders rich text exactly once, retaining the admitted result for
    /// materialization.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Possibly unsized value implementing `Display`.
    ///
    /// # Parameters
    ///
    /// * `value` - Value whose formatted text is retained.
    /// * `depth` - Root-inclusive depth of the resulting string node.
    ///
    /// # Returns
    ///
    /// The admitted, owned UTF-8 text.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error on resource rejection, or a conversion
    /// error if formatting fails.
    pub(in crate::json) fn format<T: Display + ?Sized>(
        &mut self,
        value: &T,
        depth: usize,
    ) -> ValueResult<String> {
        let text = self
            .rendering_budget(depth, false)?
            .try_write_string(|writer| write!(writer.as_fmt(), "{value}"))
            .map_err(|error| match error {
                BudgetedStringError::Budget(error) => self.error(error.into()),
                BudgetedStringError::Quantity { resource, source } => {
                    self.error(MeasuredBudgetError::quantity(resource, source))
                }
                _ => self.format_error(),
            })?;
        self.admit_output_string(text.len(), depth)?;
        Ok(text)
    }

    /// Adapts Duration conversion resource failures to projection failures.
    /// Ordinary conversion errors remain conversions; resource causes retain
    /// their original budget or quantity facts.
    ///
    /// # Parameters
    ///
    /// * `error` - Owned duration conversion failure to classify.
    ///
    /// # Returns
    ///
    /// A projection failure for matching resource causes, otherwise the
    /// original conversion failure.
    pub(in crate::json) fn duration_conversion_error(
        &self,
        error: DataConversionError,
    ) -> ValueError {
        let measured = match error.kind() {
            DataConversionErrorKind::LimitExceeded => {
                let source =
                    find_error_source::<BudgetError<ConversionResource, u64>>(&error).cloned();
                source.map(MeasuredBudgetError::Budget)
            }
            DataConversionErrorKind::Quantity => {
                let Some(resource) = error.quantity_resource() else {
                    return ValueError::Conversion(error);
                };
                let source = find_error_source::<QuantityConversionError>(&error).cloned();
                source.map(|source| MeasuredBudgetError::quantity(resource, source))
            }
            _ => return ValueError::Conversion(error),
        };
        match measured {
            Some(source) if Some(*source.resource()) == error.resource() => self.error(source),
            _ => ValueError::Conversion(error),
        }
    }

    /// Probes node capacity before creating a renderer and limits its remaining
    /// bytes.
    ///
    /// # Parameters
    ///
    /// * `depth` - Root-inclusive depth of the prospective node.
    /// * `number` - Whether to probe a number node instead of a string node.
    ///
    /// # Returns
    ///
    /// A temporary budget bounded by remaining output, payload and text
    /// capacity.
    ///
    /// # Errors
    ///
    /// Returns a projection limit error if the node probe or existing usage
    /// exceeds limits.
    fn rendering_budget(
        &mut self,
        depth: usize,
        number: bool,
    ) -> ValueResult<ResourceBudget<ConversionResource, u64>> {
        let probe = if number {
            JsonMeasurement::Number { depth, bytes: 0 }
        } else {
            JsonMeasurement::String { depth, bytes: 0 }
        };
        let result = self.structure.transaction().try_admit(probe);
        result.map_err(|error| self.error(error))?;
        let mut rendering =
            ResourceBudget::from_limit(*self.limits.operation().output_bytes_limit());
        rendering
            .try_consume(self.output.used())
            .map_err(|error| self.error(error.into()))?;
        let mut payload =
            ResourceBudget::from_limit(*self.limits.operation().structured_payload_bytes_limit());
        payload
            .try_consume(self.structure.used_payload_bytes().unwrap_or(0))
            .map_err(|error| self.error(error.into()))?;
        let text_limit =
            ResourceBudget::from_limit(*self.limits.structured().max_text_bytes_limit());
        for candidate in [payload, text_limit] {
            if candidate.remaining() < rendering.remaining() {
                rendering = candidate;
            }
        }
        Ok(rendering)
    }

    /// Reports a formatting failure without exposing source payload text.
    ///
    /// # Returns
    ///
    /// A conversion failure classified as an out-of-range JSON value.
    #[inline]
    fn format_error(&self) -> ValueError {
        ValueError::Conversion(DataConversionError::invalid(
            self.data_type,
            DataType::Json,
            InvalidValueReason::OutOfRange,
        ))
    }
}

/// Finds the first matching cause below the supplied error, without allocation.
///
/// # Type Parameters
///
/// * `T` - Concrete error type to locate in the source chain.
///
/// # Parameters
///
/// * `error` - Outer error whose sources are inspected; the outer error is
///   skipped.
///
/// # Returns
///
/// `Some` borrows the first matching cause; `None` means no cause matched.
#[must_use]
fn find_error_source<T: Error + 'static>(error: &dyn Error) -> Option<&T> {
    let mut source = error.source();
    while let Some(current) = source {
        if let Some(source) = current.downcast_ref::<T>() {
            return Some(source);
        }
        source = current.source();
    }
    None
}
