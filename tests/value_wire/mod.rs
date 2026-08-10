// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for public bounded wire-decoding types.

mod value_wire_decode_error_tests;
mod value_wire_payload_ref_v1_tests;
mod value_wire_ref_v1_tests;
mod value_wire_v1_tests;
mod internal {
    mod collection_wire_owned_tests;
    mod collection_wire_ref_tests;
    mod scalar_wire_owned_tests;
    mod scalar_wire_ref_tests;
    mod wire_data_type_v1_tests;
    mod wire_envelope_owned_tests;
    mod wire_envelope_ref_tests;
    mod wire_shape_owned_tests;
    mod wire_shape_ref_tests;
}
