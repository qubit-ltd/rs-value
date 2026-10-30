// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Exercises complete raw Wire V1 inputs under fixed and varied budgets.
//!
//! Each successful decode is checked with an independent bounded round trip.

use qubit_budget::BudgetError;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

const SMALL_INPUT_BYTES: usize = 94;
const WIDE_INPUT_BYTES: usize = 4096;
const ROUND_TRIP_BYTES: usize = 65536;

/// Checks all three profiles without truncating or consuming any input bytes.
///
/// Returns whether the wide default profile decoded successfully. Expected
/// syntax, schema, budget and version rejections return false for that profile;
/// every profile is still exercised. Panics on unexpected errors, incorrect
/// input-budget classification, failed round trips or identity differences.
pub fn check_input(data: &[u8]) -> bool {
    check_profile(data, decode_limits(SMALL_INPUT_BYTES));
    let wide_success = check_profile(data, decode_limits(WIDE_INPUT_BYTES));
    check_profile(data, varied_decode_limits(data));
    wide_success
}

/// Retains the default value profile while replacing the byte budget.
fn decode_limits(max_input_bytes: usize) -> JsonDecodeLimits {
    ValueWireV1::default_json_decode_limits()
        .into_builder()
        .max_input_bytes(max_input_bytes)
        .build()
}

/// Selects a dimension from all input bytes in its interleaved group.
fn selector(data: &[u8], offset: usize) -> usize {
    usize::from(
        data.iter()
            .skip(offset)
            .step_by(4)
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte)),
    )
}

/// Varies structural limits while preserving the remaining default resources.
fn varied_decode_limits(data: &[u8]) -> JsonDecodeLimits {
    let depth = [0_usize, 1, 4, 8, 16, 64];
    let nodes = [0_usize, 1, 8, 32, 128, 4096];
    let items = [0_usize, 1, 2, 8, 32, 4096];
    let values = ValueWireV1::default_json_decode_limits()
        .into_value_limits()
        .into_builder()
        .max_depth(depth[selector(data, 0) % depth.len()])
        .max_nodes(nodes[selector(data, 1) % nodes.len()])
        .max_sequence_items(items[selector(data, 2) % items.len()])
        .max_map_entries(items[selector(data, 3) % items.len()])
        .build();
    JsonDecodeLimits::builder()
        .max_input_bytes(WIDE_INPUT_BYTES)
        .value_limits(values)
        .build()
}

/// Checks one fresh decoding session and round-trips every accepted value.
fn check_profile(data: &[u8], limits: JsonDecodeLimits) -> bool {
    let oversized = limits.max_input_bytes().is_some_and(|maximum| data.len() > maximum);
    let result = ValueWireV1::decode_json_slice_with_limits(data, limits);
    if oversized {
        assert!(
            matches!(
                result,
                Err(ValueWireDecodeError::Budget(
                    BudgetError::LimitExceeded {
                        resource: JsonResource::InputBytes,
                        ..
                    } | BudgetError::Insufficient {
                        resource: JsonResource::InputBytes,
                        ..
                    }
                ))
            ),
            "oversized input must fail its input-byte budget: {result:?}"
        );
        return false;
    }
    match result {
        Ok(value) => {
            let encode_limits = ValueWireV1::default_json_encode_limits()
                .into_builder()
                .max_output_bytes(ROUND_TRIP_BYTES)
                .build();
            let encoded = value
                .to_json_vec_with_limits(encode_limits)
                .expect("a decoded ValueWireV1 must serialize within round-trip limits");
            let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits(ROUND_TRIP_BYTES))
                .expect("a serialized ValueWireV1 must decode within round-trip limits");
            assert_eq!(decoded, value);
            true
        }
        Err(
            ValueWireDecodeError::Syntax(_)
            | ValueWireDecodeError::InvalidJson(_)
            | ValueWireDecodeError::Budget(_)
            | ValueWireDecodeError::UnsupportedVersion { .. },
        ) => false,
        Err(error) => panic!("bounded input returned an unexpected error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use qubit_value::ValueWireV1;

    use crate::bounded_wire_input::selector;
    use crate::bounded_wire_input::varied_decode_limits;

    #[test]
    fn test_all_selector_dimensions_reach_every_profile() {
        let expected = [
            [0, 1, 4, 8, 16, 64],
            [0, 1, 8, 32, 128, 4096],
            [0, 1, 2, 8, 32, 4096],
            [0, 1, 2, 8, 32, 4096],
        ];
        for dimension in 0..4 {
            for choice in 0_u8..6 {
                let mut data = [0_u8; 4];
                data[dimension] = choice;
                let limits = varied_decode_limits(&data);
                let values = limits.value_limits();
                let actual = [
                    values.max_depth(),
                    values.max_nodes(),
                    values.max_sequence_items(),
                    values.max_map_entries(),
                ];
                for other in 0..4 {
                    let selected = if dimension == other { usize::from(choice) } else { 0 };
                    assert_eq!(actual[other], Some(expected[other][selected]));
                }
                assert_eq!(limits.max_input_bytes(), Some(4096));
                let defaults = ValueWireV1::default_json_decode_limits();
                let defaults = defaults.value_limits();
                assert_eq!(values.max_key_bytes(), defaults.max_key_bytes());
                assert_eq!(values.max_string_bytes(), defaults.max_string_bytes());
                assert_eq!(values.max_number_bytes(), defaults.max_number_bytes());
                assert_eq!(values.max_payload_bytes(), defaults.max_payload_bytes());
            }
        }
    }

    #[test]
    fn test_selector_uses_complete_groups_and_wrapping_sum() {
        let data = [255, 254, 253, 252, 2, 4, 6, 8, 3, 3, 3, 3];
        for (offset, expected) in [4, 5, 6, 7].into_iter().enumerate() {
            assert_eq!(selector(&data, offset), expected);
            assert_eq!(selector(&[], offset), 0);
        }
    }
}
