// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================

//! Allocation regression tests for the Wire V1 preflight pass.

#![cfg(all(feature = "big-decimal", feature = "json"))]

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_budget::json::JsonEncodeLimits;
use qubit_value::Value;
use qubit_value::ValueWireEncodePreflight;

struct CountingAllocator;

static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The layout comes from the caller of the global allocator.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            ALLOCATED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
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
            ALLOCATED_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
        result
    }
}

fn measured_preflight_bytes(value: &Value) -> usize {
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    ValueWireEncodePreflight::new(JsonEncodeLimits::new())
        .check_value(value)
        .expect("an unbounded preflight must accept the fixture");
    ALLOCATED_BYTES.load(Ordering::Relaxed)
}

#[test]
fn preflight_does_not_copy_big_decimal_coefficient() {
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
}
