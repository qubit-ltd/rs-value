// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded deterministic generators and shared Eq/Hash identity assertions.

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use serde_json::Map;
use serde_json::Number;
use serde_json::Value as Json;

const INPUT_LIMIT: usize = 4096;

/// Exercises every identity family with parameters read from a bounded prefix.
pub fn check_identity_properties(data: &[u8]) {
    let mut bytes = [0_u8; INPUT_LIMIT];
    let copied = data.len().min(INPUT_LIMIT);
    bytes[..copied].copy_from_slice(&data[..copied]);
    let mut cursor = Cursor {
        bytes: &bytes,
        offset: 0,
    };

    check_float32(cursor.next_u32());
    check_float64(cursor.next_u64());
    check_decimal(&mut cursor);
    check_string_map(&mut cursor);
    check_json(&mut cursor);
}

/// Checks one equivalent triple and collection lookups for any Eq + Hash type.
pub(crate) fn assert_equivalent_triplet<T>(a: &T, b: &T, c: &T, context: &str)
where
    T: Clone + Eq + Hash + std::fmt::Debug,
{
    assert!(a.eq(a), "{context}: reflexivity");
    assert_eq!(a, b, "{context}: a == b");
    assert_eq!(b, a, "{context}: symmetry");
    assert_eq!(b, c, "{context}: b == c");
    assert_eq!(a, c, "{context}: transitivity");
    assert_eq!(hash(a), hash(b), "{context}: hash(a) == hash(b)");
    assert_eq!(hash(b), hash(c), "{context}: hash(b) == hash(c)");

    let mut set = HashSet::new();
    set.insert(a.clone());
    assert!(
        set.contains(b),
        "{context}: HashSet lookup across equivalent representations"
    );
    let mut map = HashMap::new();
    map.insert(a.clone(), 1_u8);
    assert_eq!(map.insert(b.clone(), 2_u8), Some(1), "{context}: HashMap replacement");
    assert_eq!(map.len(), 1, "{context}: HashMap retains one equivalent key");
}

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn check_wrappers<T, V, M>(a: T, b: T, c: T, value: V, multi: M, context: &str)
where
    T: Clone + std::fmt::Debug,
    V: Fn(T) -> Value,
    M: Fn(Vec<T>) -> MultiValues,
{
    let values = [a, b, c];
    let wrapped = values.clone().map(value);
    assert_equivalent_triplet(&wrapped[0], &wrapped[1], &wrapped[2], &format!("{context}/Value"));

    for length in [0, 1, 3] {
        let collections = values.clone().map(|item| multi(vec![item; length]));
        assert_equivalent_triplet(
            &collections[0],
            &collections[1],
            &collections[2],
            &format!("{context}/MultiValues/{length}"),
        );
        let containers = collections.clone().map(ValueContainer::Collection);
        assert_equivalent_triplet(
            &containers[0],
            &containers[1],
            &containers[2],
            &format!("{context}/ValueContainer/Collection/{length}"),
        );
    }
    let scalars = wrapped.clone().map(ValueContainer::Scalar);
    assert_equivalent_triplet(
        &scalars[0],
        &scalars[1],
        &scalars[2],
        &format!("{context}/ValueContainer/Scalar"),
    );
    assert_ne!(scalars[0], ValueContainer::Collection(multi(vec![values[0].clone()])));
}

fn check_value_wrappers<F>(values: [Value; 3], multi: F, context: &str)
where
    F: Fn(usize, usize) -> MultiValues,
{
    assert_equivalent_triplet(&values[0], &values[1], &values[2], &format!("{context}/Value"));
    for length in [0, 1, 3] {
        let collections = [multi(0, length), multi(1, length), multi(2, length)];
        assert_equivalent_triplet(
            &collections[0],
            &collections[1],
            &collections[2],
            &format!("{context}/MultiValues/{length}"),
        );
        let containers = collections.map(ValueContainer::Collection);
        assert_equivalent_triplet(
            &containers[0],
            &containers[1],
            &containers[2],
            &format!("{context}/ValueContainer/Collection/{length}"),
        );
    }
    let scalars = values.map(ValueContainer::Scalar);
    assert_equivalent_triplet(
        &scalars[0],
        &scalars[1],
        &scalars[2],
        &format!("{context}/ValueContainer/Scalar"),
    );
}

fn check_float32(bits: u32) {
    let payload = (bits & 0x003f_ffff) | 1;
    let a = f32::from_bits(0x7f80_0000 | payload);
    let b = f32::from_bits(0xff80_0000 | payload);
    let c = f32::from_bits(0x7fff_ffff);
    let raw = [a, b, c];
    let make_multi = |index, length| MultiValues::Float32(vec![raw[index]; length]);
    check_value_wrappers(raw.map(Value::Float32), make_multi, "f32 NaN");
    let raw = [0.0_f32, -0.0, 0.0];
    let make_multi = |index, length| MultiValues::Float32(vec![raw[index]; length]);
    check_value_wrappers(raw.map(Value::Float32), make_multi, "f32 zero");
    let finite = [1.25_f32, f32::from_bits(1.25_f32.to_bits()), 1.25_f32];
    let make_multi = |index, length| MultiValues::Float32(vec![finite[index]; length]);
    check_value_wrappers(finite.map(Value::Float32), make_multi, "f32 finite copy");
}

fn check_float64(bits: u64) {
    let payload = (bits & 0x0007_ffff_ffff_ffff) | 1;
    let a = f64::from_bits(0x7ff0_0000_0000_0000 | payload);
    let b = f64::from_bits(0xfff0_0000_0000_0000 | payload);
    let c = f64::from_bits(0x7fff_ffff_ffff_ffff);
    let raw = [a, b, c];
    let make_multi = |index, length| MultiValues::Float64(vec![raw[index]; length]);
    check_value_wrappers(raw.map(Value::Float64), make_multi, "f64 NaN");
    let raw = [0.0_f64, -0.0, 0.0];
    let make_multi = |index, length| MultiValues::Float64(vec![raw[index]; length]);
    check_value_wrappers(raw.map(Value::Float64), make_multi, "f64 zero");
    let finite = [1.25_f64, f64::from_bits(1.25_f64.to_bits()), 1.25_f64];
    let make_multi = |index, length| MultiValues::Float64(vec![finite[index]; length]);
    check_value_wrappers(finite.map(Value::Float64), make_multi, "f64 finite copy");
}

fn check_decimal(cursor: &mut Cursor<'_>) {
    let coefficient = BigInt::from_bytes_be(num_bigint::Sign::Plus, &cursor.take(256));
    let coefficient = if cursor.next_u8() & 1 == 0 {
        coefficient
    } else {
        -coefficient
    };
    let scale = i64::from(cursor.next_u8()) - 128;
    let extra = i64::from(cursor.next_u8() % 33);
    let scaled = &coefficient * BigInt::from(10_u8).pow(extra as u32);
    let a = BigDecimal::new(coefficient.clone(), scale);
    let b = BigDecimal::new(scaled, scale + extra);
    let c = BigDecimal::new(coefficient, scale);
    check_wrappers(
        a,
        b,
        c,
        Value::BigDecimal,
        MultiValues::BigDecimal,
        "decimal bounded scale",
    );

    for (left_scale, right_scale) in [(i64::MIN, i64::MIN + 1), (i64::MAX - 1, i64::MAX)] {
        let left = BigDecimal::new(BigInt::from(0), left_scale);
        let right = BigDecimal::new(BigInt::from(0), right_scale);
        let zero = BigDecimal::new(BigInt::from(0), 0);
        check_wrappers(
            left,
            right,
            zero,
            Value::BigDecimal,
            MultiValues::BigDecimal,
            "decimal extreme zero",
        );
    }
    for (scale, delta) in [(i64::MIN, 32_i64), (i64::MAX - 32, 32_i64)] {
        let left = BigDecimal::new(BigInt::from(-7), scale);
        let right = BigDecimal::new(BigInt::from(-7) * BigInt::from(10_u8).pow(delta as u32), scale + delta);
        let third = left.clone();
        check_wrappers(
            left,
            right,
            third,
            Value::BigDecimal,
            MultiValues::BigDecimal,
            "decimal extreme scale",
        );
    }
    for coefficient in [BigInt::from(7), BigInt::from(-7)] {
        let left = BigDecimal::new(coefficient.clone(), 0);
        let right = BigDecimal::new(&coefficient * BigInt::from(100), 2);
        let third = BigDecimal::new(&coefficient * BigInt::from(10_000), 4);
        check_wrappers(
            left,
            right,
            third,
            Value::BigDecimal,
            MultiValues::BigDecimal,
            "decimal signed trailing zero",
        );
    }
}

fn check_string_map(cursor: &mut Cursor<'_>) {
    let count = usize::from(cursor.next_u8() % 17);
    let pairs = (0..count)
        .map(|index| {
            let key = format!("k{index:02}");
            let length = usize::from(cursor.next_u8() % 33);
            let value = String::from_utf8_lossy(&cursor.take(length)).into_owned();
            (key, value)
        })
        .collect::<Vec<_>>();
    let make = |order: Vec<usize>| {
        order
            .into_iter()
            .map(|index| pairs[index].clone())
            .collect::<HashMap<_, _>>()
    };
    let forward = make((0..count).collect());
    let reverse = make((0..count).rev().collect());
    let mut permuted_indices = (0..count).collect::<Vec<_>>();
    for index in (1..count).rev() {
        let other = usize::from(cursor.next_u8()) % (index + 1);
        permuted_indices.swap(index, other);
    }
    let permuted = make(permuted_indices);
    check_wrappers(
        forward,
        reverse,
        permuted,
        Value::StringMap,
        MultiValues::StringMap,
        "StringMap",
    );
}

fn check_json(cursor: &mut Cursor<'_>) {
    let tree = make_json(cursor, 0, &mut 0);
    let reordered = reorder_json(&tree, cursor, false);
    let third = reorder_json(&reordered, cursor, true);
    check_wrappers(tree, reordered, third, Value::Json, MultiValues::Json, "JSON/order");
}

fn make_json(cursor: &mut Cursor<'_>, depth: usize, nodes: &mut usize) -> Json {
    *nodes += 1;
    if depth >= 6 || *nodes >= 128 {
        return leaf_json(cursor);
    }
    match cursor.next_u8() % 5 {
        0 => leaf_json(cursor),
        1 => {
            let count = usize::from(cursor.next_u8() % 9);
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                if *nodes >= 128 {
                    break;
                }
                items.push(make_json(cursor, depth + 1, nodes));
            }
            Json::Array(items)
        }
        2 => {
            let count = usize::from(cursor.next_u8() % 9);
            let mut object = Map::new();
            for index in 0..count {
                if *nodes >= 128 {
                    break;
                }
                object.insert(format!("key-{index}"), make_json(cursor, depth + 1, nodes));
            }
            Json::Object(object)
        }
        _ => leaf_json(cursor),
    }
}

fn leaf_json(cursor: &mut Cursor<'_>) -> Json {
    match cursor.next_u8() % 6 {
        0 => Json::Null,
        1 => Json::Bool(cursor.next_u8() & 1 != 0),
        2 => Json::Number(Number::from(i64::from(cursor.next_u8()))),
        3 => Json::Number(Number::from_f64(if cursor.next_u8() & 1 == 0 { 0.0 } else { -0.0 }).unwrap()),
        4 => {
            let length = usize::from(cursor.next_u8() % 33);
            Json::String(String::from_utf8_lossy(&cursor.take(length)).into_owned())
        }
        _ => Json::Array(Vec::new()),
    }
}

fn reorder_json(value: &Json, cursor: &mut Cursor<'_>, rotate: bool) -> Json {
    match value {
        Json::Array(items) => Json::Array(items.iter().map(|item| reorder_json(item, cursor, rotate)).collect()),
        Json::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            let use_rotation = cursor.next_u8() & 1 == if rotate { 0 } else { 1 };
            if entries.len() > 1 {
                if use_rotation {
                    entries.rotate_left(1);
                } else {
                    entries.reverse();
                }
            }
            let mut reordered = Map::new();
            for (key, item) in entries {
                reordered.insert(key.clone(), reorder_json(item, cursor, rotate));
            }
            Json::Object(reordered)
        }
        _ => value.clone(),
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Cursor<'_> {
    fn next_u8(&mut self) -> u8 {
        let result = self.bytes[self.offset % self.bytes.len()];
        self.offset = self.offset.wrapping_add(1);
        result
    }

    fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes([self.next_u8(), self.next_u8(), self.next_u8(), self.next_u8()])
    }

    fn next_u64(&mut self) -> u64 {
        u64::from_le_bytes([
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
            self.next_u8(),
        ])
    }

    fn take(&mut self, length: usize) -> Vec<u8> {
        let mut result = vec![0; length.min(256)];
        for byte in &mut result {
            *byte = self.next_u8();
        }
        result
    }
}
