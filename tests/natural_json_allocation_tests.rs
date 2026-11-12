// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Allocation evidence for public Natural JSON projection of wide arrays.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;
use std::collections::HashMap;

use qubit_value::Value;
use serde_json::Value as JsonValue;

struct CountingAllocator;

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static TOTAL: Cell<usize> = const { Cell::new(0) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_alloc(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        record_dealloc(layout.size());
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let result = unsafe { System.realloc(pointer, layout, new_size) };
        if !result.is_null() {
            record_realloc(layout.size(), new_size);
        }
        result
    }
}

fn record_alloc(size: usize) {
    ACTIVE.with(|active| {
        if !active.get() {
            return;
        }
        BUSY.with(|busy| {
            if busy.replace(true) {
                return;
            }
            TOTAL.with(|v| v.set(v.get().saturating_add(size)));
            LIVE.with(|v| {
                let live = v.get().saturating_add(size);
                v.set(live);
                PEAK.with(|peak| peak.set(peak.get().max(live)));
            });
            busy.set(false);
        });
    });
}

fn record_dealloc(size: usize) {
    ACTIVE.with(|active| {
        if !active.get() {
            return;
        }
        BUSY.with(|busy| {
            if busy.replace(true) {
                return;
            }
            LIVE.with(|v| v.set(v.get().saturating_sub(size)));
            busy.set(false);
        });
    });
}

fn record_realloc(old_size: usize, new_size: usize) {
    ACTIVE.with(|active| {
        if !active.get() {
            return;
        }
        BUSY.with(|busy| {
            if busy.replace(true) {
                return;
            }
            TOTAL.with(|v| v.set(v.get().saturating_add(new_size)));
            LIVE.with(|v| {
                let live = v.get().saturating_sub(old_size).saturating_add(new_size);
                v.set(live);
                PEAK.with(|peak| peak.set(peak.get().max(live)));
            });
            busy.set(false);
        });
    });
}

#[derive(Clone, Copy, Debug)]
struct Reading {
    cumulative: usize,
    peak_live: usize,
}

struct Measurement;
impl Measurement {
    fn start() -> Self {
        ACTIVE.with(|v| v.set(false));
        TOTAL.with(|v| v.set(0));
        LIVE.with(|v| v.set(0));
        PEAK.with(|v| v.set(0));
        ACTIVE.with(|v| v.set(true));
        Self
    }
    fn stop(self) -> Reading {
        ACTIVE.with(|v| v.set(false));
        Reading {
            cumulative: TOTAL.with(Cell::get),
            peak_live: PEAK.with(Cell::get),
        }
    }
}
impl Drop for Measurement {
    fn drop(&mut self) {
        ACTIVE.with(|v| v.set(false));
    }
}

fn measured(operation: impl FnOnce() -> JsonValue) -> (JsonValue, Reading) {
    let guard = Measurement::start();
    let result = operation();
    let reading = guard.stop();
    (result, reading)
}

fn fixture(size: usize) -> Value {
    Value::Json(JsonValue::Array(vec![JsonValue::Bool(true); size]))
}

fn direct_output_reference(size: usize) -> (JsonValue, Reading) {
    measured(|| JsonValue::Array(vec![JsonValue::Bool(true); size]))
}

fn object_backend_is_ordered() -> bool {
    let mut map = serde_json::Map::new();
    map.insert("z".to_owned(), JsonValue::Null);
    map.insert("a".to_owned(), JsonValue::Null);
    map.keys().next().is_some_and(|key| key == "a")
}

fn object_fixture(size: usize) -> (JsonValue, Value) {
    let mut source = serde_json::Map::with_capacity(size);
    for index in 0..size {
        source.insert(format!("key-{index:04}"), JsonValue::Bool(true));
    }
    let value = Value::Json(JsonValue::Object(source.clone()));
    (JsonValue::Object(source), value)
}

fn string_map_fixture(size: usize) -> HashMap<String, String> {
    (0..size)
        .map(|index| (format!("key-{index:04}"), format!("value-{index:04}")))
        .collect()
}

fn direct_string_map_reference(source: &HashMap<String, String>) -> (JsonValue, Reading) {
    measured(|| {
        let mut output = serde_json::Map::with_capacity(source.len());
        for (key, value) in source {
            output.insert(key.clone(), JsonValue::String(value.clone()));
        }
        output.sort_keys();
        JsonValue::Object(output)
    })
}

fn direct_object_reference(source: &JsonValue) -> (JsonValue, Reading) {
    let JsonValue::Object(entries) = source else {
        unreachable!()
    };
    measured(|| {
        let mut output = serde_json::Map::with_capacity(entries.len());
        for (key, value) in entries {
            output.insert(key.clone(), value.clone());
        }
        JsonValue::Object(output)
    })
}

#[test]
fn test_wide_natural_json_projection_allocation_evidence() {
    // Warm TLS and the projection path before enabling counters.
    let warm = fixture(1);
    let _ = warm.to_json_value().expect("warm projection");

    eprintln!(
        "allocator evidence: package={}, target={}-{}; sizeof(serde_json::Value)={}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::mem::size_of::<JsonValue>()
    );
    let mut readings = Vec::new();
    for size in [16, 1024, 4096] {
        let source = fixture(size);
        let (output, reading) = measured(|| source.to_json_value().expect("public projection"));
        assert_eq!(output.as_array().unwrap().len(), size);
        let (reference, reference_reading) = direct_output_reference(size);
        assert_eq!(reference.as_array().unwrap().len(), size);
        eprintln!(
            "natural_array/{size}: cumulative={} peak_live={} direct_output: cumulative={} peak_live={}",
            reading.cumulative,
            reading.peak_live,
            reference_reading.cumulative,
            reference_reading.peak_live
        );
        readings.push((size, reading, reference_reading));
    }
    // At 4096 items, scratch allocation above the direct output must remain
    // bounded by the 16-item scratch cost plus two Value slots. The baseline
    // exceeded this by hundreds of KiB because pending/completed grew linearly.
    let scratch = |reading: Reading, output: Reading| reading.cumulative - output.cumulative;
    let live_scratch = |reading: Reading, output: Reading| reading.peak_live - output.peak_live;
    let small = (readings[0].1, readings[0].2);
    let wide = (readings[2].1, readings[2].2);
    let fixed_slack = 2 * std::mem::size_of::<JsonValue>();
    assert!(scratch(wide.0, wide.1) <= scratch(small.0, small.1) + fixed_slack);
    assert!(live_scratch(wide.0, wide.1) <= live_scratch(small.0, small.1) + fixed_slack);
}

#[test]
fn test_wide_natural_json_object_allocation_evidence() {
    let warm_source = object_fixture(1).1;
    let _ = warm_source.to_json_value().expect("warm object projection");
    let mut readings = Vec::new();
    for size in [16, 1024, 4096] {
        let (source, value) = object_fixture(size);
        let (projected, projection) =
            measured(|| value.to_json_value().expect("object projection"));
        let (reference, direct) = direct_object_reference(&source);
        assert_eq!(projected, reference);
        eprintln!(
            "natural_object/{size}: cumulative={} peak_live={} direct_map: cumulative={} peak_live={}",
            projection.cumulative, projection.peak_live, direct.cumulative, direct.peak_live
        );
        readings.push((size, projection, direct));
    }
    if object_backend_is_ordered() {
        let scratch = |actual: Reading, output: Reading| actual.cumulative - output.cumulative;
        let live_scratch = |actual: Reading, output: Reading| actual.peak_live - output.peak_live;
        let small = (readings[0].1, readings[0].2);
        let wide = (readings[2].1, readings[2].2);
        let fixed_slack = 2 * std::mem::size_of::<JsonValue>();
        assert!(
            scratch(wide.0, wide.1) <= scratch(small.0, small.1) + fixed_slack,
            "object projection retained a width-sized entry scratch buffer"
        );
        assert!(
            live_scratch(wide.0, wide.1) <= live_scratch(small.0, small.1) + fixed_slack,
            "object projection retained a width-sized entry scratch buffer"
        );
    }
}

#[test]
fn test_wide_natural_json_string_map_allocation_evidence() {
    let warm = Value::StringMap(string_map_fixture(1));
    let _ = warm.to_json_value().expect("warm string map projection");
    for size in [16, 1024, 4096] {
        let source = string_map_fixture(size);
        let value = Value::StringMap(source.clone());
        let (projected, projection) =
            measured(|| value.to_json_value().expect("string map projection"));
        let (reference, direct) = direct_string_map_reference(&source);
        assert_eq!(projected, reference);
        eprintln!(
            "string_map/{size}: cumulative={} peak_live={} direct_map: cumulative={} peak_live={}",
            projection.cumulative, projection.peak_live, direct.cumulative, direct.peak_live
        );
    }
    // StringMap stays on the sorted borrowed-entry path: direct insertion
    // followed by `sort_keys` regressed preserve_order throughput by >20%.
}
