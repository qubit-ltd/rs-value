// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Processing Module Tests
//!
//! Provides integration tests for the value processing framework.
mod contracts;
mod doc_examples_tests;
mod finite_float_tests;
mod finite_float {
    mod internal {
        mod finite_float_tests;
    }
}
mod identity;
mod into_value_default_tests;
mod json_tests;
mod multi_values;
mod named_multi_values_tests;
mod named_multi_values {
    mod internal {
        mod named_multi_values_wire_owned_tests;
        mod named_multi_values_wire_ref_tests;
    }
}
mod named_value_tests;
mod named_value {
    mod internal {
        mod named_value_wire_owned_tests;
        mod named_value_wire_ref_tests;
    }
}
mod numeric_comparison_error_tests;
mod strict_json_tests;
mod strict_value_read_tests;
mod value;
mod value_container_tests;
mod value_error_tests;
mod value_missing_tests;
mod value_type_table_tests;
mod value_wire;
mod value_wire_encode_error_tests;
mod value_wire_payload_v1_tests;
mod value_wire_tests;
mod wide_integer_tests;
mod wide_integer {
    mod internal {
        mod display_integer_tests;
        mod integer_visitor_tests;
        mod parsed_integer_tests;
    }
}
mod wire {
    mod decimal {
        mod internal {
            mod decimal_visitor_tests;
            mod display_decimal_tests;
            mod parsed_decimal_tests;
        }
        mod decimal_tests;
    }
    mod internal {
        mod big_decimal_payload_tests;
        mod duration_payload_tests;
    }
}
mod wire_tests;
