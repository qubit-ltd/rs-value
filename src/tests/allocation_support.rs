// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Thread-local allocation measurement for crate-internal tests.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;

use serde::Deserialize;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::value::Error as ValueError;
use serde::de::value::StrDeserializer;
use serde::de::value::StringDeserializer;

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
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The pointer and layout are supplied by the allocator caller.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: The pointer and layouts are supplied by the allocator caller.
        let result = unsafe { System.realloc(pointer, layout, size) };
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
        let allocations = ALLOCATIONS.with(Cell::get);
        let bytes = BYTES.with(Cell::get);
        std::mem::forget(self);
        (allocations, bytes)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        ENABLED.with(|enabled| enabled.set(false));
    }
}

fn measure<T>(action: impl FnOnce() -> T) -> (T, usize, usize) {
    let guard = Guard::start();
    let result = action();
    let (allocations, bytes) = guard.stop();
    (result, allocations, bytes)
}

/// Confirms a field identifier consumes common string forms without allocating.
pub(crate) fn assert_string_forms_allocate_nothing<T>(field: &str)
where
    for<'de> T: Deserialize<'de>,
{
    let (from_str, allocations, bytes) = measure(|| T::deserialize(StrDeserializer::<ValueError>::new(field)));
    assert!(from_str.is_ok(), "expected field {field:?} to be recognized");
    assert_eq!((allocations, bytes), (0, 0), "StrDeserializer");

    let (from_borrowed, allocations, bytes) =
        measure(|| T::deserialize(BorrowedStrDeserializer::<ValueError>::new(field)));
    assert!(from_borrowed.is_ok(), "expected field {field:?} to be recognized");
    assert_eq!((allocations, bytes), (0, 0), "BorrowedStrDeserializer");

    let owned_field = field.to_owned();
    let (from_owned, allocations, bytes) =
        measure(|| T::deserialize(StringDeserializer::<ValueError>::new(owned_field)));
    assert!(from_owned.is_ok(), "expected field {field:?} to be recognized");
    assert_eq!((allocations, bytes), (0, 0), "StringDeserializer");

    let json = format!("\"{field}\"");
    let (from_json, allocations, bytes) = measure(|| serde_json::from_str::<T>(&json));
    assert!(from_json.is_ok(), "expected JSON field {field:?} to be recognized");
    assert_eq!((allocations, bytes), (0, 0), "unescaped serde_json field");
}

/// Confirms the field identifier accepts the same non-string forms as String.
pub(crate) fn assert_key_forms_match_string<T>(field: &str)
where
    for<'de> T: Deserialize<'de>,
{
    use serde::de::value::BytesDeserializer;
    use serde::de::value::U64Deserializer;

    let valid_bytes = field.as_bytes();
    let old_valid = String::deserialize(BytesDeserializer::<ValueError>::new(valid_bytes));
    let new_valid = T::deserialize(BytesDeserializer::<ValueError>::new(valid_bytes));
    assert_eq!(old_valid.is_ok(), new_valid.is_ok());

    let invalid_utf8 = [0xff];
    let old_invalid = String::deserialize(BytesDeserializer::<ValueError>::new(&invalid_utf8));
    let new_invalid = T::deserialize(BytesDeserializer::<ValueError>::new(&invalid_utf8));
    assert_eq!(old_invalid.is_ok(), new_invalid.is_ok());
    assert_eq!(
        old_invalid.err().map(|error| error.to_string()),
        new_invalid.err().map(|error| error.to_string())
    );

    let old_integer = String::deserialize(U64Deserializer::<ValueError>::new(7));
    let new_integer = T::deserialize(U64Deserializer::<ValueError>::new(7));
    assert_eq!(old_integer.is_ok(), new_integer.is_ok());
    assert_eq!(
        old_integer.as_ref().err().map(ToString::to_string),
        new_integer.as_ref().err().map(ToString::to_string)
    );
    assert!(old_integer.is_err(), "String key deserialization rejects integers");
}

/// Confirms JSON string escapes are decoded before field classification.
pub(crate) fn assert_escaped_field_is_recognized<T>(json: &str)
where
    for<'de> T: Deserialize<'de>,
{
    assert!(serde_json::from_str::<T>(json).is_ok(), "expected escaped field {json}");
}
