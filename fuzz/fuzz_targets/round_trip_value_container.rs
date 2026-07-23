// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Fuzzes bounded JSON round trips for [`ValueContainer`].

use libfuzzer_sys::fuzz_target;
use qubit_value::ValueContainer;

/// Mirrors the CI input budget while retaining every checked-in corpus seed.
const MAX_INPUT_BYTES: usize = 4096;

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    if let Ok(value) = serde_json::from_slice::<ValueContainer>(data) {
        let encoded = serde_json::to_vec(&value)
            .expect("a valid ValueContainer must serialize");
        let decoded = serde_json::from_slice::<ValueContainer>(&encoded)
            .expect("serialized ValueContainer must deserialize");
        assert_eq!(decoded, value);
    }
});
