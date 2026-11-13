// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic generated checks for Eq/Hash identity contracts.

use super::internal::identity_properties::check_identity_properties;

#[test]
fn test_identity_property_fixed_inputs() {
    for (case, input) in [
        Vec::new(),
        vec![0; 32],
        vec![u8::MAX; 32],
        (0..=u8::MAX).collect(),
        (0..=u8::MAX)
            .map(|index| if index % 2 == 0 { 0x55 } else { 0xaa })
            .collect(),
        vec![1; 256],
        vec![2; 256],
    ]
    .into_iter()
    .enumerate()
    {
        let result = std::panic::catch_unwind(|| check_identity_properties(&input));
        if let Err(payload) = result {
            let detail = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic");
            panic!("identity property fixed case={case}, input={input:02x?}: {detail}");
        }
    }
}

#[test]
fn test_identity_property_seeded_inputs() {
    for seed in 0_u64..256 {
        let mut state = seed;
        let length = (seed as usize) % 257;
        let input = (0..length)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (state >> 32) as u8
            })
            .collect::<Vec<_>>();
        let result = std::panic::catch_unwind(|| check_identity_properties(&input));
        if let Err(payload) = result {
            let detail = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic");
            panic!("identity property seed={seed}, input={input:02x?}: {detail}");
        }
    }
}

#[test]
fn test_identity_property_rejects_equal_values_with_different_hashes() {
    use std::hash::Hash;
    use std::hash::Hasher;

    #[derive(Clone, Debug)]
    struct BrokenHash(u8);

    impl PartialEq for BrokenHash {
        fn eq(&self, _other: &Self) -> bool {
            true
        }
    }

    impl Eq for BrokenHash {}

    impl Hash for BrokenHash {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state);
        }
    }

    let result = std::panic::catch_unwind(|| {
        super::internal::identity_properties::assert_equivalent_triplet(
            &BrokenHash(7),
            &BrokenHash(8),
            &BrokenHash(9),
            "negative oracle",
        );
    });
    assert!(result.is_err(), "oracle must reject unequal hashes for equal values");
}
