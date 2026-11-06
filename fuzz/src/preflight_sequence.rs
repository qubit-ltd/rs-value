// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded cumulative-checker sequences for preflight fuzzing.

use qubit_budget::MeasuredBudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonEncodeSession;
use qubit_budget::json::JsonResource;
use qubit_json::encode::JsonEncoder;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;

use crate::preflight_input::MAX_INPUT_BYTES;
use crate::preflight_input::MAX_ITEMS;
use crate::preflight_input::MAX_OUTPUT_BYTES;
use crate::preflight_input::bounded_input;
use crate::preflight_input::value_from_bytes;
use crate::structured_wire_input::container_from_bytes;

const MAX_OPERATIONS: usize = 8;
const OPERATION_BYTES: usize = MAX_INPUT_BYTES / MAX_OPERATIONS;

/// Observable path counts from one bounded sequence execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SequenceReport {
    /// Number of operations sent to the checker.
    pub operation_count: usize,
    /// Bit 0/1/2 records `check_value`/`check_values`/`check_container` use.
    pub checked_entries: u8,
    /// Number of successful checker operations.
    pub successes: usize,
    /// Number of rejected checker operations.
    pub failures: usize,
    /// Whether a successful checker operation ran after a rejection.
    pub success_after_failure: bool,
    /// Number of rejected operations followed by rollback probes.
    pub rollback_probes: usize,
    /// Number of node-budget failures.
    pub node_failures: usize,
    /// Number of resource failures for payload bytes.
    pub payload_failures: usize,
    /// Number of resource failures for output bytes.
    pub output_failures: usize,
    /// Number of accepted prefixes whose direct array encoding succeeded.
    pub array_oracles: usize,
}

/// One decoded checker call, retaining its entry-point shape for replay.
#[derive(Debug, Clone)]
enum Operation {
    Value(Value),
    Values(MultiValues),
    Container(ValueContainer),
}

impl Operation {
    /// Clones the payload into the common shape used by the encoding oracle.
    fn container(&self) -> ValueContainer {
        match self {
            Self::Value(value) => ValueContainer::Scalar(value.clone()),
            Self::Values(values) => ValueContainer::Collection(values.clone()),
            Self::Container(value) => value.clone(),
        }
    }

    /// Identifies the exercised checker entry point in the sequence report.
    #[must_use]
    #[inline]
    fn entry_bit(&self) -> u8 {
        match self {
            Self::Value(_) => 1,
            Self::Values(_) => 2,
            Self::Container(_) => 4,
        }
    }

    /// Dispatches through the same public entry point selected by the input.
    #[inline]
    fn invoke(&self, checker: &mut ValueWireEncodePreflight) -> Result<(), MeasuredBudgetError<JsonResource, usize>> {
        match self {
            Self::Value(value) => checker.check_value(value),
            Self::Values(values) => checker.check_values(values),
            Self::Container(value) => checker.check_container(value),
        }
    }
}

/// Runs one deterministic, bounded sequence with limits derived from its input.
#[must_use]
pub fn run_sequence(data: &[u8]) -> SequenceReport {
    let input = bounded_input(data);
    let mode = input.first().copied().unwrap_or_default() % 4;
    let limits = limits_for_mode(mode, input.get(1).copied().unwrap_or_default());
    run_sequence_with_limits(input, limits)
}

/// Runs the sequence with explicit limits for deterministic property tests.
#[must_use]
pub fn run_sequence_with_limits(data: &[u8], limits: JsonEncodeLimits) -> SequenceReport {
    let input = bounded_input(data);
    let body = input.get(1..).unwrap_or_default();
    let chunks: Vec<&[u8]> = if body.is_empty() {
        vec![&[]]
    } else {
        body.chunks(OPERATION_BYTES).take(MAX_OPERATIONS).collect()
    };
    let mut checker = ValueWireEncodePreflight::new(limits);
    let mut accepted = Vec::with_capacity(chunks.len());
    let mut report = SequenceReport::default();
    let mut has_failed = false;

    for chunk in chunks {
        let operation = make_operation(chunk);
        report.operation_count += 1;
        report.checked_entries |= operation.entry_bit();
        let before = checker.clone();
        match operation.invoke(&mut checker) {
            Ok(()) => {
                report.successes += 1;
                report.success_after_failure |= has_failed;
                accepted.push(operation);
            }
            Err(error) => {
                report.failures += 1;
                has_failed = true;
                match *error.resource() {
                    JsonResource::Nodes => report.node_failures += 1,
                    JsonResource::PayloadBytes => report.payload_failures += 1,
                    JsonResource::OutputBytes => report.output_failures += 1,
                    _ => {}
                }
                // Reconstruct the successful prefix to compare probe behavior.
                let mut replay = ValueWireEncodePreflight::new(limits);
                for prior in &accepted {
                    prior.invoke(&mut replay).expect("accepted prefix must replay");
                }
                assert_eq!(format!("{checker:?}"), format!("{before:?}"));
                assert_eq!(format!("{checker:?}"), format!("{replay:?}"));
                let snapshot = format!("{checker:?}");
                let probe = [&operation, accepted.first().unwrap_or(&operation)];
                for item in probe {
                    let mut after_failure = checker.clone();
                    let mut from_replay = before.clone();
                    let left = item.invoke(&mut after_failure).map_err(|error| format!("{error:?}"));
                    let right = item.invoke(&mut from_replay).map_err(|error| format!("{error:?}"));
                    assert_eq!(left, right, "failed-state probes must match");
                    assert_eq!(format!("{after_failure:?}"), format!("{from_replay:?}"));
                }
                assert_eq!(format!("{checker:?}"), snapshot);
                report.rollback_probes += 1;
            }
        }

        let mut replay = ValueWireEncodePreflight::new(limits);
        for prior in &accepted {
            prior.invoke(&mut replay).expect("accepted prefix must replay");
        }
        assert_eq!(format!("{checker:?}"), format!("{replay:?}"));

        let mut fresh = ValueWireEncodePreflight::new(limits);
        let mut wires = Vec::with_capacity(accepted.len());
        let mut encodable = true;
        for prior in &accepted {
            let container = prior.container();
            match ValueWireV1::try_from(container) {
                Ok(wire) => wires.push(wire),
                Err(_) => {
                    encodable = false;
                    break;
                }
            }
        }
        if encodable
            && JsonEncoder::new(JsonEncodeSession::from_limits(limits))
                .to_vec(&wires)
                .is_ok()
        {
            for prior in &accepted {
                fresh
                    .check_container(&prior.container())
                    .expect("successful array encoding must pass fresh sequential preflight");
            }
            report.array_oracles += 1;
        }
    }

    report
}

/// Decodes the selector and bounded payload without changing scalar/collection
/// shape.
#[must_use]
fn make_operation(chunk: &[u8]) -> Operation {
    let selector = chunk.first().copied().unwrap_or_default() % 4;
    let source = chunk.get(1..).unwrap_or_default();
    match selector {
        0 => {
            let mut data = vec![0; source.len().saturating_add(1).min(OPERATION_BYTES)];
            data[0] = source.first().copied().unwrap_or_default() % 26;
            let copy_len = data.len() - 1;
            data[1..].copy_from_slice(&source[..copy_len]);
            match container_from_bytes(&data) {
                ValueContainer::Scalar(value) => Operation::Value(value),
                ValueContainer::Collection(_) => {
                    unreachable!("scalar tags produce scalar containers")
                }
            }
        }
        1 => {
            let mut data = vec![0; source.len().saturating_add(1).min(OPERATION_BYTES)];
            data[0] = 26 + source.first().copied().unwrap_or_default() % 26;
            let copy_len = data.len() - 1;
            data[1..].copy_from_slice(&source[..copy_len]);
            match container_from_bytes(&data) {
                ValueContainer::Collection(values) => Operation::Values(values),
                ValueContainer::Scalar(_) => unreachable!("collection tags produce collections"),
            }
        }
        2 => Operation::Container(container_from_bytes(source)),
        _ => Operation::Container(value_from_bytes(source)),
    }
}

/// Selects the generous baseline or one intentionally constrained resource
/// budget.
#[must_use]
fn limits_for_mode(mode: u8, selector: u8) -> JsonEncodeLimits {
    let builder = JsonEncodeLimits::builder()
        .max_output_bytes(MAX_OUTPUT_BYTES)
        .max_depth(80)
        .max_nodes(MAX_ITEMS * MAX_ITEMS)
        .max_sequence_items(MAX_ITEMS + 2)
        .max_map_entries(MAX_ITEMS + 2)
        .max_key_bytes(MAX_INPUT_BYTES)
        .max_string_bytes(MAX_INPUT_BYTES)
        .max_number_bytes(MAX_INPUT_BYTES)
        .max_payload_bytes(MAX_OUTPUT_BYTES);
    match mode {
        0 => builder.build(),
        1 => builder.max_nodes(usize::from(selector) % 16).build(),
        2 => builder.max_payload_bytes(usize::from(selector) % 128).build(),
        _ => builder.max_output_bytes(usize::from(selector) % 128).build(),
    }
}
