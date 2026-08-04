// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Fuzzes the structured V1 encode/decode path from generated runtime values.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    NaiveDate,
    NaiveTime,
    TimeZone,
    Utc,
};
use libfuzzer_sys::fuzz_target;
use num_bigint::BigInt;
use qubit_value::{
    MultiValues,
    Value,
    ValueContainer,
    ValueWireV1,
};
use url::Url;

const TAG_COUNT: u8 = 52;

/// Copies at most `N` fuzz bytes into a fixed-width little-endian buffer.
fn padded_bytes<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut bytes = [0_u8; N];
    let copy_len = data.len().min(N);
    bytes[..copy_len].copy_from_slice(&data[..copy_len]);
    bytes
}

/// Derives a deterministic scalar seed from the first fuzz bytes.
fn seed(data: &[u8]) -> u64 {
    u64::from_le_bytes(padded_bytes(data))
}

/// Converts a bounded suffix of fuzz input into valid UTF-8 text.
fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data.get(1..data.len().min(65)).unwrap_or_default())
        .into_owned()
}

/// Maps a seed to a date inside a fixed one-year fuzzing window.
fn date(seed: u64) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("fixed fuzz date must be valid")
        .checked_add_days(chrono::Days::new(seed % 365))
        .expect("fuzz date offset must remain in range")
}

/// Maps a seed to a valid time-of-day value.
fn time(seed: u64) -> NaiveTime {
    NaiveTime::from_hms_nano_opt(
        (seed % 24) as u32,
        ((seed / 24) % 60) as u32,
        ((seed / (24 * 60)) % 60) as u32,
        (seed % 1_000_000_000) as u32,
    )
    .expect("fuzz time components must be valid")
}

/// Builds a small, valid runtime shape from the fuzz input bytes. Keeping the
/// shape construction typed means the fuzz target exercises the wire encoder
/// and decoder independently from arbitrary JSON syntax handling.
fn container_from_bytes(data: &[u8]) -> ValueContainer {
    let tag = data.first().copied().unwrap_or_default() % TAG_COUNT;
    let seed = seed(data);
    let number = i32::from_le_bytes([
        data.get(1).copied().unwrap_or_default(),
        data.get(2).copied().unwrap_or_default(),
        data.get(3).copied().unwrap_or_default(),
        data.get(4).copied().unwrap_or_default(),
    ]);
    let number64 = i64::from_le_bytes(padded_bytes(data));
    let number128 = i128::from_le_bytes(padded_bytes(data));
    let unsigned_number = u64::from_le_bytes(padded_bytes(data));
    let unsigned_number128 = u128::from_le_bytes(padded_bytes(data));
    let text = text(data);
    let date = date(seed);
    let time = time(seed);
    let date_time = date.and_time(time);
    let instant = Utc.from_utc_datetime(&date_time);
    let duration =
        Duration::new(seed % 1_000_000, (seed % 1_000_000_000) as u32);
    let big_integer = BigInt::from(i128::from_le_bytes(padded_bytes(data)));
    let big_decimal = BigDecimal::new(big_integer.clone(), (seed % 18) as i64);
    let url = Url::parse("https://example.com/fuzz")
        .expect("fixed fuzz URL must be valid");
    let string_map = HashMap::from([
        ("key".to_owned(), text.clone()),
        ("seed".to_owned(), seed.to_string()),
    ]);
    let json = serde_json::json!({
        "text": text,
        "seed": seed,
        "items": [true, null, number],
    });
    let character = char::from_u32((seed % 0x11_0000) as u32).unwrap_or('🦀');
    let float32 = number as f32 / 3.0;
    let float64 = number as f64 / 3.0;

    match tag {
        0 => ValueContainer::Scalar(Value::Bool(
            data.get(1).copied().unwrap_or_default() & 1 == 1,
        )),
        1 => ValueContainer::Scalar(Value::Char(character)),
        2 => ValueContainer::Scalar(Value::Int8(number as i8)),
        3 => ValueContainer::Scalar(Value::Int16(number as i16)),
        4 => ValueContainer::Scalar(Value::Int32(number)),
        5 => ValueContainer::Scalar(Value::Int64(number64)),
        6 => ValueContainer::Scalar(Value::Int128(number128)),
        7 => ValueContainer::Scalar(Value::UInt8(seed as u8)),
        8 => ValueContainer::Scalar(Value::UInt16(seed as u16)),
        9 => ValueContainer::Scalar(Value::UInt32(seed as u32)),
        10 => ValueContainer::Scalar(Value::UInt64(unsigned_number)),
        11 => ValueContainer::Scalar(Value::UInt128(unsigned_number128)),
        12 => ValueContainer::Scalar(Value::Float32(float32)),
        13 => ValueContainer::Scalar(Value::Float64(float64)),
        14 => ValueContainer::Scalar(Value::BigInteger(big_integer.clone())),
        15 => ValueContainer::Scalar(Value::BigDecimal(big_decimal.clone())),
        16 => ValueContainer::Scalar(Value::String(text.clone())),
        17 => ValueContainer::Scalar(Value::Date(date)),
        18 => ValueContainer::Scalar(Value::Time(time)),
        19 => ValueContainer::Scalar(Value::DateTime(date_time)),
        20 => ValueContainer::Scalar(Value::Instant(instant)),
        21 => ValueContainer::Scalar(Value::Duration(duration)),
        22 => ValueContainer::Scalar(Value::Url(url.clone())),
        23 => ValueContainer::Scalar(Value::StringMap(string_map.clone())),
        24 => ValueContainer::Scalar(Value::Json(json.clone())),
        25 => ValueContainer::Scalar(Value::new_unset(
            Value::Int32(0).data_type(),
        )),
        26 => ValueContainer::Collection(MultiValues::Bool(vec![true, false])),
        27 => {
            ValueContainer::Collection(MultiValues::Char(vec![character, '🦀']))
        }
        28 => ValueContainer::Collection(MultiValues::Int8(vec![
            number as i8,
            (number as i8).wrapping_add(1),
        ])),
        29 => ValueContainer::Collection(MultiValues::Int16(vec![
            number as i16,
            (number as i16).wrapping_add(1),
        ])),
        30 => ValueContainer::Collection(MultiValues::Int32(vec![
            number,
            number.wrapping_add(1),
        ])),
        31 => ValueContainer::Collection(MultiValues::Int64(vec![
            number64,
            number64.wrapping_add(1),
        ])),
        32 => ValueContainer::Collection(MultiValues::Int128(vec![
            number128,
            number128.wrapping_add(1),
        ])),
        33 => ValueContainer::Collection(MultiValues::UInt8(vec![
            seed as u8,
            (seed as u8).wrapping_add(1),
        ])),
        34 => ValueContainer::Collection(MultiValues::UInt16(vec![
            seed as u16,
            (seed as u16).wrapping_add(1),
        ])),
        35 => ValueContainer::Collection(MultiValues::UInt32(vec![
            seed as u32,
            (seed as u32).wrapping_add(1),
        ])),
        36 => ValueContainer::Collection(MultiValues::UInt64(vec![
            unsigned_number,
            unsigned_number.wrapping_add(1),
        ])),
        37 => ValueContainer::Collection(MultiValues::UInt128(vec![
            unsigned_number128,
            unsigned_number128.wrapping_add(1),
        ])),
        38 => ValueContainer::Collection(MultiValues::Float32(vec![
            float32,
            float32 + 0.5,
        ])),
        39 => ValueContainer::Collection(MultiValues::Float64(vec![
            float64,
            float64 + 0.5,
        ])),
        40 => ValueContainer::Collection(MultiValues::BigInteger(vec![
            big_integer,
        ])),
        41 => ValueContainer::Collection(MultiValues::BigDecimal(vec![
            big_decimal,
        ])),
        42 => ValueContainer::Collection(MultiValues::String(vec![text])),
        43 => ValueContainer::Collection(MultiValues::Date(vec![date])),
        44 => ValueContainer::Collection(MultiValues::Time(vec![time])),
        45 => {
            ValueContainer::Collection(MultiValues::DateTime(vec![date_time]))
        }
        46 => ValueContainer::Collection(MultiValues::Instant(vec![instant])),
        47 => ValueContainer::Collection(MultiValues::Duration(vec![duration])),
        48 => ValueContainer::Collection(MultiValues::Url(vec![url])),
        49 => {
            ValueContainer::Collection(MultiValues::StringMap(vec![string_map]))
        }
        50 => ValueContainer::Collection(MultiValues::Json(vec![json])),
        _ => ValueContainer::Collection(MultiValues::new_unset(
            MultiValues::Int32(Vec::new()).data_type(),
        )),
    }
}

fuzz_target!(|data: &[u8]| {
    let container = container_from_bytes(data);
    let wire = ValueWireV1::try_from(container.clone());

    match wire {
        Ok(wire) => {
            let encoded = serde_json::to_vec(&wire)
                .expect("validated V1 values serialize");
            let decoded: ValueWireV1 = serde_json::from_slice(&encoded)
                .expect("serialized V1 values deserialize");
            assert_eq!(decoded, wire);
            assert_eq!(decoded.into_container(), container);
        }
        Err(error) => {
            panic!("generated structured value must be V1 encodable: {error}")
        }
    }
});
