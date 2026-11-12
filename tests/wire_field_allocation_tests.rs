// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Allocation and key-representation checks for fixed Wire record fields.

#![cfg(feature = "json")]

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use qubit_value::ValueWireV1Seed;
use serde::Deserialize;
use serde::de::DeserializeSeed;
use serde::de::value::Error as ValueError;
use serde::de::value::StrDeserializer;

struct CountingAllocator;

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The layout is supplied by the allocator caller.
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The pointer and layout are supplied by the allocator caller.
        unsafe { System.dealloc(ptr, layout) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: The pointer and layouts are supplied by the allocator caller.
        let result = unsafe { System.realloc(ptr, layout, size) };
        if !result.is_null() {
            record(size);
        }
        result
    }
}

fn record(size: usize) {
    ENABLED.with(|enabled| {
        if enabled.get() {
            ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
            BYTES.with(|bytes| bytes.set(bytes.get().saturating_add(size)));
        }
    });
}

struct Guard;

impl Guard {
    fn start() -> Self {
        ALLOCATIONS.with(|count| count.set(0));
        BYTES.with(|bytes| bytes.set(0));
        ENABLED.with(|enabled| enabled.set(true));
        Self
    }

    fn stop(self) -> (usize, usize) {
        ENABLED.with(|enabled| enabled.set(false));
        let counts = ALLOCATIONS.with(Cell::get);
        let bytes = BYTES.with(Cell::get);
        std::mem::forget(self);
        (counts, bytes)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        ENABLED.with(|enabled| enabled.set(false));
    }
}

fn measured<T>(f: impl FnOnce() -> T) -> (T, usize, usize) {
    let guard = Guard::start();
    let value = f();
    let (count, bytes) = guard.stop();
    (value, count, bytes)
}

#[test]
fn test_old_string_classifier_is_detected() {
    let key = "secs";
    let (old, allocations, bytes) =
        measured(|| String::deserialize(StrDeserializer::<ValueError>::new(key)));
    assert_eq!(old.as_deref(), Ok(key));
    assert!(
        allocations > 0 && bytes > 0,
        "old String path must allocate"
    );
}

#[test]
fn test_public_seed_duration_batch_allocation_counts() {
    for size in [1, 32, 256] {
        let records = std::iter::repeat_n(r#"{"secs":1,"nanos":2}"#, size)
            .collect::<Vec<_>>()
            .join(",");
        let input =
            format!(r#"{{"version":1,"value":{{"collection":{{"duration":[{records}]}}}}}}"#);
        let decode = || {
            let mut deserializer = serde_json::Deserializer::from_str(&input);
            ValueWireV1Seed::new().deserialize(&mut deserializer)
        };
        let warm = decode();
        assert!(warm.is_ok());
        drop(warm);
        let (result, allocations, bytes) = measured(decode);
        assert!(result.is_ok());
        eprintln!("public_duration_batch size={size} allocations={allocations} bytes={bytes}");
        drop(result);
    }
}
