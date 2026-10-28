// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Allocation regression tests for Wire V1 preflight and payload construction.

#![cfg(all(feature = "big-decimal", feature = "json"))]

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::BudgetError;
use qubit_budget::MeasuredBudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWireV1;

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATED_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The layout comes from the caller of the global allocator.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            COUNT_ALLOCATIONS.with(|enabled| {
                if enabled.get() {
                    ALLOCATED_BYTES.with(|bytes| bytes.set(bytes.get().saturating_add(layout.size())));
                }
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The pointer and layout come from the caller of the global allocator.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: The pointer and layouts come from the caller of the global allocator.
        let result = unsafe { System.realloc(pointer, layout, new_size) };
        if !result.is_null() {
            COUNT_ALLOCATIONS.with(|enabled| {
                if enabled.get() {
                    ALLOCATED_BYTES.with(|bytes| bytes.set(bytes.get().saturating_add(new_size)));
                }
            });
        }
        result
    }
}

fn measured_preflight_bytes(value: &Value) -> usize {
    let (result, bytes) =
        measured_allocations(|| ValueWireEncodePreflight::new(JsonEncodeLimits::new()).check_value(value));
    result.expect("an unbounded preflight must accept the fixture");
    bytes
}

fn measured_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    let guard = AllocationGuard::start();
    let result = operation();
    let allocated = guard.stop();
    (result, allocated)
}

struct AllocationGuard;

impl AllocationGuard {
    fn start() -> Self {
        ALLOCATED_BYTES.with(|bytes| bytes.set(0));
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(true));
        Self
    }

    fn stop(self) -> usize {
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
        ALLOCATED_BYTES.with(Cell::get)
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    }
}

#[test]
fn test_big_decimal_preflight_and_payload_construction_do_not_copy_coefficient() {
    let small = Value::BigDecimal(BigDecimal::new(BigInt::from(123_i32), 2));
    let coefficient = "9".repeat(128 * 1024);
    let large = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(coefficient.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    ));

    let small_bytes = measured_preflight_bytes(&small);
    let large_bytes = measured_preflight_bytes(&large);

    assert!(
        large_bytes <= small_bytes.saturating_add(4096),
        "preflight allocation grew with the coefficient: small={small_bytes}, large={large_bytes}",
    );

    assert_big_decimal_payload_construction_does_not_copy_coefficient();
}

#[test]
fn test_big_decimal_preflight_rejection_allocation_does_not_scale_with_coefficient() {
    let small_digits = "9".repeat(4096);
    let large_digits = "9".repeat(128 * 1024);
    let small = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(small_digits.as_bytes(), 10).expect("small coefficient must parse"),
        2,
    ));
    let large = Value::BigDecimal(BigDecimal::new(
        BigInt::parse_bytes(large_digits.as_bytes(), 10).expect("large coefficient must parse"),
        2,
    ));
    let base = ValueWireV1::default_json_encode_limits();
    let profiles = [
        (
            base.into_builder().max_string_bytes(64).build(),
            JsonResource::StringBytes,
        ),
        (
            base.into_builder().max_payload_bytes(128).build(),
            JsonResource::PayloadBytes,
        ),
        (
            base.into_builder().max_output_bytes(256).build(),
            JsonResource::OutputBytes,
        ),
    ];

    for (limits, resource) in profiles {
        let (small_error, small_bytes) =
            measured_allocations(|| ValueWireEncodePreflight::new(limits).check_value(&small));
        let (large_error, large_bytes) =
            measured_allocations(|| ValueWireEncodePreflight::new(limits).check_value(&large));
        assert_expected_limit(
            small_error.expect_err("small fixture must exceed this profile"),
            resource,
        );
        assert_expected_limit(
            large_error.expect_err("large fixture must exceed this profile"),
            resource,
        );
        assert!(
            large_bytes <= small_bytes.saturating_add(4096),
            "rejected coefficient preflight allocation grew: resource={resource:?}, 4096 digits={small_bytes}, 128 Ki digits={large_bytes}",
        );
    }
}

fn assert_expected_limit(error: MeasuredBudgetError<JsonResource, usize>, expected: JsonResource) {
    assert!(
        matches!(
            error,
            MeasuredBudgetError::Budget(BudgetError::LimitExceeded { resource, .. }) if resource == expected
        ),
        "expected a {expected:?} rejection, got {error:?}",
    );
}

fn assert_big_decimal_payload_construction_does_not_copy_coefficient() {
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

    let (invalid_small_borrowed_scalar, _) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&invalid_small_scalar));
    let (invalid_borrowed_scalar, _) =
        measured_allocations(|| ValueWirePayloadRefV1::from_value(&invalid_large_scalar));
    let (invalid_small_borrowed_values, _) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&invalid_small_values));
    let (invalid_borrowed_values, _) =
        measured_allocations(|| ValueWirePayloadRefV1::from_values(&invalid_large_values));
    let invalid_small_owned_scalar = invalid_small_scalar.clone();
    let invalid_owned_scalar = invalid_large_scalar.clone();
    let invalid_small_owned_values = invalid_small_values.clone();
    let invalid_owned_values = invalid_large_values.clone();
    let (invalid_small_owned_scalar_result, _) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_small_owned_scalar));
    let (invalid_owned_scalar_result, _) = measured_allocations(|| ValueWirePayloadV1::try_from(invalid_owned_scalar));
    let (invalid_small_owned_values_result, _) =
        measured_allocations(|| ValueWirePayloadV1::try_from(invalid_small_owned_values));
    let (invalid_owned_values_result, _) = measured_allocations(|| ValueWirePayloadV1::try_from(invalid_owned_values));

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
