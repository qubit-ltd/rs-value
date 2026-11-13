// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Measures allocations made while hashing root JSON leaves.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_value::Value;
use serde_json::Value as JsonValue;

struct CountingAllocator;

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, new_size) };
        if !result.is_null() {
            record(new_size);
        }
        result
    }
}

fn record(bytes: usize) {
    ACTIVE.with(|active| {
        if !active.get() {
            return;
        }
        BUSY.with(|busy| {
            if busy.replace(true) {
                return;
            }
            ALLOCATIONS.with(|count| count.set(count.get().saturating_add(1)));
            BYTES.with(|total| total.set(total.get().saturating_add(bytes)));
            busy.set(false);
        });
    });
}

fn allocated_by(operation: impl FnOnce()) -> (usize, usize) {
    ACTIVE.with(|value| value.set(false));
    ALLOCATIONS.with(|value| value.set(0));
    BYTES.with(|value| value.set(0));
    ACTIVE.with(|value| value.set(true));
    operation();
    ACTIVE.with(|value| value.set(false));
    (ALLOCATIONS.with(Cell::get), BYTES.with(Cell::get))
}

#[test]
fn test_root_leaf_hash_allocation_evidence() {
    let values = [
        ("null", Value::Json(JsonValue::Null)),
        ("bool", Value::Json(JsonValue::Bool(true))),
        ("integer", Value::Json(JsonValue::Number(42.into()))),
        (
            "float",
            Value::Json(JsonValue::Number(
                serde_json::Number::from_f64(42.5).expect("finite float"),
            )),
        ),
        ("string_empty", Value::Json(JsonValue::String(String::new()))),
        (
            "string_nonempty",
            Value::Json(JsonValue::String(String::from("json identity"))),
        ),
    ];

    // Warm up allocator TLS and Hash dispatch before collecting readings.
    for (_, value) in &values {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        std::hint::black_box(hasher.finish());
    }

    let mut readings = Vec::new();
    for (name, value) in &values {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let reading = allocated_by(|| value.hash(&mut hasher));
        std::hint::black_box(hasher.finish());
        eprintln!("json_root/{name}: allocations={} bytes={}", reading.0, reading.1);
        readings.push((*name, reading));
    }
    let direct_number = serde_json::Number::from(42);
    let mut direct_hasher = std::collections::hash_map::DefaultHasher::new();
    let direct_reading = allocated_by(|| direct_number.hash(&mut direct_hasher));
    std::hint::black_box(direct_hasher.finish());
    eprintln!(
        "direct_number: allocations={} bytes={}",
        direct_reading.0, direct_reading.1
    );

    let lexical_number: serde_json::Number =
        serde_json::from_str("12345678901234567890.123456789").expect("valid large JSON number");
    let lexical_value = Value::Json(JsonValue::Number(lexical_number.clone()));
    let mut direct_lexical_hasher = std::collections::hash_map::DefaultHasher::new();
    let direct_lexical_reading = allocated_by(|| lexical_number.hash(&mut direct_lexical_hasher));
    std::hint::black_box(direct_lexical_hasher.finish());
    let mut value_lexical_hasher = std::collections::hash_map::DefaultHasher::new();
    let value_lexical_reading = allocated_by(|| lexical_value.hash(&mut value_lexical_hasher));
    std::hint::black_box(value_lexical_hasher.finish());
    eprintln!(
        "large_number: Value allocations={} bytes={} direct Number allocations={} bytes={}",
        value_lexical_reading.0, value_lexical_reading.1, direct_lexical_reading.0, direct_lexical_reading.1,
    );

    for (name, reading) in readings {
        if matches!(name, "null" | "bool" | "string_empty" | "string_nonempty") {
            assert_eq!(reading, (0, 0), "{name} root hashing should allocate nothing");
        }
    }
    let mut integer_hasher = std::collections::hash_map::DefaultHasher::new();
    let integer = allocated_by(|| values[2].1.hash(&mut integer_hasher));
    std::hint::black_box(integer_hasher.finish());
    assert_eq!(
        integer, direct_reading,
        "integer allocations should be Number's own Hash cost"
    );
    assert_eq!(
        value_lexical_reading, direct_lexical_reading,
        "large Number allocations should be its own Hash cost"
    );
}
