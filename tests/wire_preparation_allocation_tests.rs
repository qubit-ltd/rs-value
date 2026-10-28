// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "json")]

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use qubit_budget::BudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATED_BYTES: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The layout is supplied by the global allocator caller.
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
        // SAFETY: The pointer and layout are supplied by the global allocator caller.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: The pointer and layouts are supplied by the global allocator caller.
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

fn measured_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    let guard = AllocationGuard::start();
    let result = operation();
    let allocated = guard.stop();
    (result, allocated)
}

fn object(size: usize) -> Value {
    let entries = (0..size).map(|index| (format!("key-{index:05}"), serde_json::json!(index)));
    Value::Json(serde_json::Value::Object(entries.collect()))
}

fn string_values(size: usize, length: usize) -> MultiValues {
    MultiValues::String(vec!["x".repeat(length); size])
}

#[test]
fn test_borrowed_wire_preparation_allocation_does_not_scale_with_json_object() {
    let small = object(16);
    let large = object(8192);
    let _ = ValueWirePayloadRefV1::from_value(&small).unwrap();
    let _ = ValueWirePayloadRefV1::from_value(&large).unwrap();

    let (small_payload, small_bytes) = measured_allocations(|| ValueWirePayloadRefV1::from_value(&small));
    let (large_payload, large_bytes) = measured_allocations(|| ValueWirePayloadRefV1::from_value(&large));
    assert!(small_payload.is_ok());
    assert!(large_payload.is_ok());
    assert!(
        large_bytes <= small_bytes.saturating_add(4096),
        "JSON tree borrow allocations differ: 16={small_bytes}, 8192={large_bytes}"
    );
}

#[test]
fn test_borrowed_wire_preparation_allocation_does_not_scale_with_string_collection() {
    let short = string_values(16, 8);
    let long = string_values(8192, 8);
    let _ = ValueWirePayloadRefV1::from_values(&short).unwrap();
    let _ = ValueWirePayloadRefV1::from_values(&long).unwrap();

    let (short_payload, short_bytes) = measured_allocations(|| ValueWirePayloadRefV1::from_values(&short));
    let (long_payload, long_bytes) = measured_allocations(|| ValueWirePayloadRefV1::from_values(&long));
    assert!(short_payload.is_ok());
    assert!(long_payload.is_ok());
    assert!(
        long_bytes <= short_bytes.saturating_add(4096),
        "string collection borrow allocations differ: 16={short_bytes}, 8192={long_bytes}"
    );
}

#[test]
fn test_canonical_json_rejection_admits_object_before_sorting_entries() {
    let small_value = object(16);
    let small = ValueWirePayloadRefV1::from_value(&small_value).unwrap();
    let large_value = object(8192);
    let large = ValueWirePayloadRefV1::from_value(&large_value).unwrap();
    let limits = JsonEncodeLimits::builder().max_nodes(2).build();
    let _ = small.to_json_vec_with_limits(limits);
    let _ = large.to_json_vec_with_limits(limits);

    let (small_result, small_bytes) = measured_allocations(|| small.to_json_vec_with_limits(limits));
    let (large_result, large_bytes) = measured_allocations(|| large.to_json_vec_with_limits(limits));
    for result in [small_result, large_result] {
        assert!(
            matches!(
                result,
                Err(ValueWireEncodeError::Budget(BudgetError::Insufficient {
                    resource: qubit_budget::json::JsonResource::Nodes,
                    ..
                }))
            ),
            "expected node budget rejection at object admission"
        );
    }
    assert!(
        large_bytes <= small_bytes.saturating_add(4096),
        "rejected object allocation grew before node admission: 16={small_bytes}, 8192={large_bytes}"
    );
}

#[test]
fn test_bounded_collection_entry_accepts_empty_collection() {
    let limits = JsonEncodeLimits::builder().max_output_bytes(1024).build();
    let values = MultiValues::Int32(Vec::new());
    let wire = ValueWirePayloadRefV1::from_values(&values).unwrap();
    assert!(
        !wire.to_json_vec_with_limits(limits).unwrap().is_empty(),
        "bounded serialization of an empty collection should produce JSON"
    );
}
