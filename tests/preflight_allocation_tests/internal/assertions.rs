// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resource-error and coefficient-independent allocation assertions.

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;

use super::measurement::measured_allocations;

pub(crate) fn assert_expected_encode_limit(
    error: ValueWireEncodeError,
    expected: JsonResource,
    dto: &str,
    method: &str,
) {
    assert!(
        matches!(error, ValueWireEncodeError::Budget(BudgetError::LimitExceeded { resource, .. }) if resource == expected),
        "{dto} {method} expected {expected:?}, got {error:?}",
    );
}

pub(crate) fn assert_bounded_direct_helper_allocations(
    dto: &str,
    method: &str,
    resource: JsonResource,
    small: usize,
    large: usize,
) {
    assert!(
        large <= small.saturating_add(4096),
        "{dto} {method} {resource:?} allocation grew with coefficient: 4096 digits={small}, 128 Ki digits={large}",
    );
}

pub(crate) fn assert_expected_limit(error: MeasuredBudgetError<JsonResource, usize>, expected: JsonResource) {
    assert!(
        matches!(
            error,
            MeasuredBudgetError::Budget(BudgetError::LimitExceeded { resource, .. }) if resource == expected
        ),
        "expected a {expected:?} rejection, got {error:?}",
    );
}

/// Compares borrowed and owned construction for valid and rejected decimal
/// scales. Fixture creation and owned-input cloning stay outside every measured
/// closure.
pub(crate) fn assert_big_decimal_payload_construction_does_not_copy_coefficient() {
    let coefficient = "9".repeat(128 * 1024);
    let large_coefficient = BigInt::parse_bytes(coefficient.as_bytes(), 10).expect("large coefficient must parse");

    let small_scalar = Value::BigDecimal(BigDecimal::new(BigInt::from(123_i32), 2));
    let large_scalar = Value::BigDecimal(BigDecimal::new(large_coefficient.clone(), 2));
    let small_values = MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(123_i32), 2)]);
    let large_values = MultiValues::BigDecimal(vec![BigDecimal::new(large_coefficient.clone(), 2)]);
    let invalid_small_scalar = Value::BigDecimal(BigDecimal::new(BigInt::from(123_i32), i64::MAX));
    let invalid_large_scalar = Value::BigDecimal(BigDecimal::new(large_coefficient.clone(), i64::MAX));
    let invalid_small_values = MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(123_i32), i64::MAX)]);
    let invalid_large_values = MultiValues::BigDecimal(vec![BigDecimal::new(large_coefficient, i64::MAX)]);

    let (small_borrowed_scalar, small_borrowed_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&small_scalar));
    let (large_borrowed_scalar, large_borrowed_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&large_scalar));
    let (small_borrowed_values, small_borrowed_values_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&small_values));
    let (large_borrowed_values, large_borrowed_values_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&large_values));

    let small_owned_scalar = small_scalar.clone();
    let large_owned_scalar = large_scalar.clone();
    let small_owned_values = small_values.clone();
    let large_owned_values = large_values.clone();
    let (small_owned_scalar_result, small_owned_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(small_owned_scalar));
    let (large_owned_scalar_result, large_owned_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(large_owned_scalar));
    let (small_owned_values_result, small_owned_values_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(small_owned_values));
    let (large_owned_values_result, large_owned_values_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(large_owned_values));

    for (succeeded, label) in [
        (small_borrowed_scalar.is_ok(), "small borrowed scalar"),
        (large_borrowed_scalar.is_ok(), "large borrowed scalar"),
        (small_borrowed_values.is_ok(), "small borrowed collection"),
        (large_borrowed_values.is_ok(), "large borrowed collection"),
        (small_owned_scalar_result.is_ok(), "small owned scalar"),
        (large_owned_scalar_result.is_ok(), "large owned scalar"),
        (small_owned_values_result.is_ok(), "small owned collection"),
        (large_owned_values_result.is_ok(), "large owned collection"),
    ] {
        assert!(succeeded, "{label} with a valid scale must construct");
    }

    for (small, large, label) in [
        (
            small_borrowed_scalar_bytes,
            large_borrowed_scalar_bytes,
            "borrowed scalar",
        ),
        (
            small_borrowed_values_bytes,
            large_borrowed_values_bytes,
            "borrowed collection",
        ),
        (small_owned_scalar_bytes, large_owned_scalar_bytes, "owned scalar"),
        (small_owned_values_bytes, large_owned_values_bytes, "owned collection"),
    ] {
        assert!(
            large <= small.saturating_add(4096),
            "{label} construction allocation grew with the coefficient: small={small}, large={large}",
        );
    }

    let (invalid_small_borrowed_scalar, invalid_small_borrowed_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&invalid_small_scalar));
    let (invalid_borrowed_scalar, invalid_borrowed_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&invalid_large_scalar));
    let (invalid_small_borrowed_values, invalid_small_borrowed_values_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&invalid_small_values));
    let (invalid_borrowed_values, invalid_borrowed_values_bytes) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&invalid_large_values));
    let invalid_small_owned_scalar = invalid_small_scalar.clone();
    let invalid_owned_scalar = invalid_large_scalar.clone();
    let invalid_small_owned_values = invalid_small_values.clone();
    let invalid_owned_values = invalid_large_values.clone();
    let (invalid_small_owned_scalar_result, invalid_small_owned_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_small_owned_scalar));
    let (invalid_owned_scalar_result, invalid_owned_scalar_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_owned_scalar));
    let (invalid_small_owned_values_result, invalid_small_owned_values_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_small_owned_values));
    let (invalid_owned_values_result, invalid_owned_values_bytes) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_owned_values));

    for (small, large, label) in [
        (
            invalid_small_borrowed_scalar_bytes,
            invalid_borrowed_scalar_bytes,
            "invalid borrowed scalar",
        ),
        (
            invalid_small_borrowed_values_bytes,
            invalid_borrowed_values_bytes,
            "invalid borrowed collection",
        ),
        (
            invalid_small_owned_scalar_bytes,
            invalid_owned_scalar_bytes,
            "invalid owned scalar",
        ),
        (
            invalid_small_owned_values_bytes,
            invalid_owned_values_bytes,
            "invalid owned collection",
        ),
    ] {
        assert!(
            large <= small.saturating_add(4096),
            "{label} construction allocation grew with the coefficient: small={small}, large={large}",
        );
    }

    for (error, label) in [
        (invalid_small_borrowed_scalar.err(), "small borrowed scalar"),
        (invalid_borrowed_scalar.err(), "borrowed scalar"),
        (invalid_small_borrowed_values.err(), "small borrowed collection"),
        (invalid_borrowed_values.err(), "borrowed collection"),
        (invalid_small_owned_scalar_result.err(), "small owned scalar"),
        (invalid_owned_scalar_result.err(), "owned scalar"),
        (invalid_small_owned_values_result.err(), "small owned collection"),
        (invalid_owned_values_result.err(), "owned collection"),
    ] {
        assert!(
            matches!(error, Some(ValueWireEncodeError::BigDecimalScaleTooLarge { .. })),
            "{label} must preserve the scale error"
        );
    }
}
