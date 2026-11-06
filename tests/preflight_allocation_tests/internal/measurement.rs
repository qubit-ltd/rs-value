// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Thread-local allocation accounting and measured operations.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use qubit_budget::json::JsonEncodeLimits;
use qubit_value::Value;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWirePayloadRefV1;

/// Delegates to the system allocator while counting successful allocation bytes
/// on the current thread only when a measurement is enabled. Frees are not
/// deducted.
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

    #[inline]
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

/// Counts unbounded preflight allocations, checking acceptance after counting
/// stops.
pub(crate) fn measured_preflight_bytes(value: &Value) -> usize {
    let (result, bytes) =
        measured_allocations(|| ValueWireEncodePreflight::new(JsonEncodeLimits::new()).check_value(value));
    result.expect("an unbounded preflight must accept the fixture");
    bytes
}

/// Measures bytes allocated by one synchronous operation on the current thread.
/// Measurements must not nest; the guard disables counting even if the
/// operation panics.
pub(crate) fn measured_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    let guard = AllocationGuard::start();
    let result = operation();
    let allocated = guard.stop();
    (result, allocated)
}

/// Disables thread-local allocation counting on normal return and unwinding.
struct AllocationGuard;

impl AllocationGuard {
    #[inline]
    fn start() -> Self {
        ALLOCATED_BYTES.with(|bytes| bytes.set(0));
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(true));
        Self
    }

    #[inline]
    fn stop(self) -> usize {
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
        ALLOCATED_BYTES.with(Cell::get)
    }
}

impl Drop for AllocationGuard {
    #[inline]
    fn drop(&mut self) {
        COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    }
}

/// Runs two unmeasured payload constructions before sampling a third
/// construction. Warm-up allocations are excluded so the sample isolates
/// steady-state validation; construction results are checked after counting has
/// stopped.
pub(crate) fn measured_json_payload_construction_bytes(value: &Value) -> usize {
    let _ = ValueWirePayloadRefV1::from_value(value).expect("number fixture must be accepted");
    let _ = ValueWirePayloadRefV1::from_value(value).expect("warm-up fixture must be accepted");
    let (result, bytes) = measured_allocations(|| ValueWirePayloadRefV1::from_value(value));
    let _ = result.expect("measured number fixture must be accepted");
    bytes
}
