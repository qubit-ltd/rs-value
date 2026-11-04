// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::json::JsonEncodeLimits;
use qubit_value_fuzz::preflight_sequence::run_sequence;
use qubit_value_fuzz::preflight_sequence::run_sequence_with_limits;

fn limits(nodes: usize, payload: usize, output: usize) -> JsonEncodeLimits {
    JsonEncodeLimits::builder()
        .max_output_bytes(output)
        .max_depth(80)
        .max_nodes(nodes)
        .max_sequence_items(40)
        .max_map_entries(40)
        .max_key_bytes(4096)
        .max_string_bytes(4096)
        .max_number_bytes(4096)
        .max_payload_bytes(payload)
        .build()
}

fn sequence(selectors: &[u8]) -> Vec<u8> {
    let mut input = vec![0; 4096];
    for (index, selector) in selectors.iter().enumerate() {
        input[1 + index * 512] = *selector;
    }
    input
}

#[test]
fn sequence_is_bounded_deterministic_and_empty_input_checks_once() {
    for input in [vec![], vec![0x5a], vec![0x5a; 4096], vec![0x5a; 4097]] {
        let first = run_sequence(&input);
        let second = run_sequence(&input);
        assert_eq!(first, second, "sequence must be deterministic");
        assert!(first.operation_count > 0, "empty input still checks one operation");
        assert!(first.operation_count <= 8);
        assert!(first.checked_entries > 0, "checker entry path must be reached");
    }
    assert_eq!(run_sequence(&vec![0x5a; 4096]), run_sequence(&vec![0x5a; 4097]));
}

#[test]
fn sequence_dispatches_all_legal_checker_entries_and_array_oracle() {
    let report = run_sequence_with_limits(&sequence(&[0, 1, 2, 3, 0, 1, 2, 3]), limits(4096, 4096, 4096));
    assert_eq!(report.operation_count, 8);
    assert_eq!(report.checked_entries, 0b111);
    assert_eq!(report.failures, 0);
    assert!(report.array_oracles > 0, "direct array encoding path must execute");
}

#[test]
fn sequence_exercises_nodes_payload_and_output_budget_failures() {
    let inputs = [
        (sequence(&[0, 0]), limits(0, 4096, 4096)),
        (sequence(&[3, 0]), limits(4096, 0, 4096)),
        (sequence(&[0, 0]), limits(4096, 4096, 0)),
    ];
    let nodes = run_sequence_with_limits(&inputs[0].0, inputs[0].1);
    let payload = run_sequence_with_limits(&inputs[1].0, inputs[1].1);
    let output = run_sequence_with_limits(&inputs[2].0, inputs[2].1);
    assert!(nodes.node_failures > 0);
    assert!(payload.payload_failures > 0);
    assert!(output.output_failures > 0);
    assert!(nodes.rollback_probes > 0 && payload.rollback_probes > 0 && output.rollback_probes > 0);
}

#[test]
fn sequence_can_succeed_after_a_failed_operation() {
    let input = sequence(&[2, 3, 2]);
    let report = (1..512)
        .map(|output| run_sequence_with_limits(&input, limits(4096, 4096, output)))
        .find(|report| report.success_after_failure)
        .expect("a later operation after a failed operation must be accepted");
    assert!(report.failures > 0);
    assert!(report.successes > 0);
    assert!(report.rollback_probes > 0);
}
