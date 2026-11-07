// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![no_main]

use libfuzzer_sys::fuzz_target;
use qubit_value_fuzz::natural_json_projection::check_input;

fuzz_target!(|data: &[u8]| check_input(data));
