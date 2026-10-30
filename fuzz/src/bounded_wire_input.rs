// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetError;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

const SMALL_INPUT_BYTES: usize = 94;
const WIDE_INPUT_BYTES: usize = 4096;
const ROUND_TRIP_BYTES: usize = 65_536;

/// Checks raw V1 wire input under small, wide, and varied structural budgets.
///
/// Returns whether the wide default-profile decoder accepts the input.
pub fn check_input(data: &[u8]) -> bool {
    let small = decode_limits(SMALL_INPUT_BYTES, ValueWireV1::default_json_decode_limits());
    let wide = decode_limits(WIDE_INPUT_BYTES, ValueWireV1::default_json_decode_limits());
    let varied = varied_decode_limits(data);
    let _ = check_profile(data, small);
    let wide_succeeded = check_profile(data, wide);
    let _ = check_profile(data, varied);
    wide_succeeded
}

fn decode_limits(max_input_bytes: usize, limits: JsonDecodeLimits) -> JsonDecodeLimits {
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, max_input_bytes))
        .value_limits(limits.into_value_limits())
        .build()
}

fn selector(data: &[u8], offset: usize) -> usize {
    usize::from(
        data.iter()
            .skip(offset)
            .step_by(4)
            .fold(0_u8, |sum, byte| sum.wrapping_add(*byte)),
    )
}

fn varied_decode_limits(data: &[u8]) -> JsonDecodeLimits {
    let depth = [0_usize, 1, 4, 8, 16, 64];
    let nodes = [0_usize, 1, 8, 32, 128, 4096];
    let items = [0_usize, 1, 2, 8, 32, 4096];
    let base = ValueWireV1::default_json_decode_limits();
    let values = base
        .into_value_limits()
        .into_builder()
        .max_depth(depth[selector(data, 0) % depth.len()])
        .max_nodes(nodes[selector(data, 1) % nodes.len()])
        .max_sequence_items(items[selector(data, 2) % items.len()])
        .max_map_entries(items[selector(data, 3) % items.len()])
        .build();
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, WIDE_INPUT_BYTES))
        .value_limits(values)
        .build()
}

fn check_profile(data: &[u8], limits: JsonDecodeLimits) -> bool {
    let max_input_bytes = limits
        .max_input_bytes()
        .expect("fuzz decode profiles always set an input-byte limit");
    match ValueWireV1::decode_json_slice_with_limits(data, limits) {
        Ok(value) => {
            let encode_limits = JsonEncodeLimits::builder()
                .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, ROUND_TRIP_BYTES))
                .value_limits(ValueWireV1::default_json_encode_limits().into_value_limits())
                .build();
            let encoded = value
                .to_json_vec_with_limits(encode_limits)
                .expect("decoded values must round-trip within the output budget");
            let decode_limits = decode_limits(ROUND_TRIP_BYTES, ValueWireV1::default_json_decode_limits());
            let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits)
                .expect("encoded values must round-trip within the input budget");
            assert_eq!(decoded, value);
            true
        }
        Err(ValueWireDecodeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::InputBytes,
            maximum,
            ..
        })) => {
            assert_eq!(maximum, max_input_bytes);
            false
        }
        Err(ValueWireDecodeError::Budget(BudgetError::Insufficient {
            resource: JsonResource::InputBytes,
            limit,
            ..
        })) => {
            assert_eq!(limit, max_input_bytes);
            false
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
    use super::selector;
    use super::varied_decode_limits;

    #[test]
    fn test_each_selector_reaches_all_six_values() {
        let depths = [0, 1, 4, 8, 16, 64];
        let nodes = [0, 1, 8, 32, 128, 4096];
        let items = [0, 1, 2, 8, 32, 4096];
        for offset in 0..4 {
            let selected = [0_u8, 1, 2, 3, 4, 5]
                .iter()
                .map(|value| {
                    let mut fixture = vec![0; offset + 4];
                    fixture[offset] = *value;
                    selector(&fixture, offset) % 6
                })
                .collect::<Vec<_>>();
            assert_eq!(selected, [0, 1, 2, 3, 4, 5]);
            for (choice, expected) in selected.iter().copied().enumerate() {
                let mut fixture = vec![0; offset + 1];
                fixture[offset] = expected as u8;
                let limits = varied_decode_limits(&fixture);
                let profile = limits.value_limits();
                match offset {
                    0 => assert_eq!(profile.max_depth(), Some(depths[choice])),
                    1 => assert_eq!(profile.max_nodes(), Some(nodes[choice])),
                    2 => assert_eq!(profile.max_sequence_items(), Some(items[choice])),
                    3 => assert_eq!(profile.max_map_entries(), Some(items[choice])),
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn test_varied_profile_uses_expected_input_budget() {
        let limits = varied_decode_limits(&[0]);
        assert_eq!(limits.max_input_bytes(), Some(4096));
        let profile = limits.value_limits();
        assert_eq!(profile.max_depth(), Some(0));
        assert_eq!(profile.max_nodes(), Some(0));
        assert_eq!(profile.max_sequence_items(), Some(0));
        assert_eq!(profile.max_map_entries(), Some(0));
    }
}
