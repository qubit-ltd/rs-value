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

const TAG_COUNT: u8 = 32;

fn padded_bytes<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut bytes = [0_u8; N];
    let copy_len = data.len().min(N);
    bytes[..copy_len].copy_from_slice(&data[..copy_len]);
    bytes
}

fn seed(data: &[u8]) -> u64 {
    u64::from_le_bytes(padded_bytes(data))
}

fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data.get(1..data.len().min(65)).unwrap_or_default())
        .into_owned()
}

fn date(seed: u64) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("fixed fuzz date must be valid")
        .checked_add_days(chrono::Days::new(seed % 365))
        .expect("fuzz date offset must remain in range")
}

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

    match tag {
        0 => ValueContainer::Scalar(Value::Bool(
            data.get(1).copied().unwrap_or_default() & 1 == 1,
        )),
        1 => ValueContainer::Scalar(Value::Char(character)),
        2 => ValueContainer::Scalar(Value::Int8(number as i8)),
        3 => ValueContainer::Scalar(Value::Int32(number)),
        4 => ValueContainer::Scalar(Value::Int128(i128::from_le_bytes(
            padded_bytes(data),
        ))),
        5 => ValueContainer::Scalar(Value::UInt64(seed)),
        6 => ValueContainer::Scalar(Value::UInt128(u128::from_le_bytes(
            padded_bytes(data),
        ))),
        7 => ValueContainer::Scalar(Value::Float32(number as f32 / 3.0)),
        8 => ValueContainer::Scalar(Value::Float64(number as f64 / 3.0)),
        9 => ValueContainer::Scalar(Value::BigInteger(big_integer.clone())),
        10 => ValueContainer::Scalar(Value::BigDecimal(big_decimal.clone())),
        11 => ValueContainer::Scalar(Value::String(text.clone())),
        12 => ValueContainer::Scalar(Value::Date(date)),
        13 => ValueContainer::Scalar(Value::Time(time)),
        14 => ValueContainer::Scalar(Value::DateTime(date_time)),
        15 => ValueContainer::Scalar(Value::Instant(instant)),
        16 => ValueContainer::Scalar(Value::Duration(duration)),
        17 => ValueContainer::Scalar(Value::Url(url.clone())),
        18 => ValueContainer::Scalar(Value::StringMap(string_map.clone())),
        19 => ValueContainer::Scalar(Value::Json(json.clone())),
        20 => ValueContainer::Scalar(Value::new_unset(
            Value::Int32(0).data_type(),
        )),
        21 => ValueContainer::Collection(MultiValues::Int32(Vec::new())),
        22 => ValueContainer::Collection(MultiValues::Int32(vec![
            number,
            number.wrapping_add(1),
        ])),
        23 => ValueContainer::Collection(MultiValues::BigInteger(vec![
            big_integer,
        ])),
        24 => ValueContainer::Collection(MultiValues::BigDecimal(vec![
            big_decimal,
        ])),
        25 => ValueContainer::Collection(MultiValues::Date(vec![date])),
        26 => ValueContainer::Collection(MultiValues::Duration(vec![duration])),
        27 => ValueContainer::Collection(MultiValues::Url(vec![url])),
        28 => {
            ValueContainer::Collection(MultiValues::StringMap(vec![string_map]))
        }
        29 => ValueContainer::Collection(MultiValues::Json(vec![json])),
        30 => ValueContainer::Collection(MultiValues::String(vec![text])),
        _ => ValueContainer::Collection(MultiValues::Bool(vec![true, false])),
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
