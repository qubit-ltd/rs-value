// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic, bounded structured values for Wire V1 fuzzing.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::Days;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::TimeZone;
use chrono::Utc;
use num_bigint::BigInt;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use serde_json::json;
use url::Url;

const TAG_COUNT: u8 = 52;
const MAX_INPUT: usize = 4096;
const MAX_NODES: usize = 96;
const MAX_TEXT: usize = 24;

const FLOAT32_BOUNDARIES: [u32; 10] = [
    0x0000_0000,
    0x8000_0000,
    0x0000_0001,
    0x8000_0001,
    0x007f_ffff,
    0x0080_0000,
    0x7f7f_ffff,
    0xff7f_ffff,
    0x3f80_0001,
    0x3f7f_ffff,
];
const FLOAT64_BOUNDARIES: [u64; 10] = [
    0x0000_0000_0000_0000,
    0x8000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x8000_0000_0000_0001,
    0x000f_ffff_ffff_ffff,
    0x0010_0000_0000_0000,
    0x7fef_ffff_ffff_ffff,
    0xffef_ffff_ffff_ffff,
    0x3ff0_0000_0000_0001,
    0x3fefffffffffffff,
];

/// Selects one finite f32 path: mode byte 0 keeps the legacy arithmetic value,
/// mode 1 reads bytes 2..6 as little-endian IEEE bits (zero-padded), and mode
/// 2 selects a fixed boundary bit pattern using byte 2. Non-finite bit patterns
/// fall back to the same-sign maximum finite value.
#[must_use]
fn float32(data: &[u8], arithmetic: i32) -> f32 {
    match data.get(1).copied().unwrap_or_default() % 3 {
        0 => arithmetic as f32 / 3.0,
        1 => {
            let bits = u32::from_le_bytes(padded_bytes(data.get(2..).unwrap_or_default()));
            let value = f32::from_bits(bits);
            if value.is_finite() {
                value
            } else {
                f32::from_bits((bits & 0x8000_0000) | 0x7f7f_ffff)
            }
        }
        _ => f32::from_bits(
            FLOAT32_BOUNDARIES[usize::from(data.get(2).copied().unwrap_or_default()) % FLOAT32_BOUNDARIES.len()],
        ),
    }
}

/// Selects one finite f64 path: mode byte 0 keeps the legacy arithmetic value,
/// mode 1 reads bytes 2..10 as little-endian IEEE bits (zero-padded), and mode
/// 2 selects a fixed boundary bit pattern using byte 2. Non-finite bit patterns
/// fall back to the same-sign maximum finite value.
#[must_use]
fn float64(data: &[u8], arithmetic: i64) -> f64 {
    match data.get(1).copied().unwrap_or_default() % 3 {
        0 => arithmetic as f64 / 3.0,
        1 => {
            let bits = u64::from_le_bytes(padded_bytes(data.get(2..).unwrap_or_default()));
            let value = f64::from_bits(bits);
            if value.is_finite() {
                value
            } else {
                f64::from_bits((bits & 0x8000_0000_0000_0000) | 0x7fef_ffff_ffff_ffff)
            }
        }
        _ => f64::from_bits(
            FLOAT64_BOUNDARIES[usize::from(data.get(2).copied().unwrap_or_default()) % FLOAT64_BOUNDARIES.len()],
        ),
    }
}

/// Asserts Wire V1 rejects input-controlled NaN payloads and both infinities
/// for f32/f64 scalar values and non-empty collections.
pub fn check_non_finite(data: &[u8]) {
    let sign32 = if data.first().copied().unwrap_or_default() & 1 == 0 {
        0
    } else {
        0x8000_0000
    };
    let sign64 = if data.first().copied().unwrap_or_default() & 1 == 0 {
        0
    } else {
        0x8000_0000_0000_0000
    };
    let payload32 = u32::from_le_bytes(padded_bytes(data.get(1..5).unwrap_or_default())) & 0x003f_ffff;
    let payload64 = u64::from_le_bytes(padded_bytes(data.get(1..9).unwrap_or_default())) & 0x0007_ffff_ffff_ffff;
    let nan32 = f32::from_bits(sign32 | 0x7fc0_0000 | payload32);
    let nan64 = f64::from_bits(sign64 | 0x7ff8_0000_0000_0000 | payload64);
    for value in [f32::INFINITY, f32::NEG_INFINITY, nan32] {
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Scalar(Value::Float32(value))),
            Err(qubit_value::ValueWireEncodeError::NonFiniteFloat { .. })
        ));
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Collection(MultiValues::Float32(vec![value]))),
            Err(qubit_value::ValueWireEncodeError::NonFiniteFloat { .. })
        ));
    }
    for value in [f64::INFINITY, f64::NEG_INFINITY, nan64] {
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Scalar(Value::Float64(value))),
            Err(qubit_value::ValueWireEncodeError::NonFiniteFloat { .. })
        ));
        assert!(matches!(
            ValueWireV1::try_from(ValueContainer::Collection(MultiValues::Float64(vec![value]))),
            Err(qubit_value::ValueWireEncodeError::NonFiniteFloat { .. })
        ));
    }
}

/// Reads bounded fuzz bytes, zero-filling exhausted scalar reads and advancing
/// monotonically even when a requested slice extends past the input.
struct Cursor<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    #[must_use]
    #[inline]
    fn new(data: &'a [u8]) -> Self {
        Self {
            data: &data[..data.len().min(MAX_INPUT)],
            at: 0,
        }
    }
    #[inline]
    fn byte(&mut self) -> u8 {
        let b = self.data.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        b
    }
    #[inline]
    fn take(&mut self, n: usize) -> &'a [u8] {
        let start = self.at.min(self.data.len());
        let end = start.saturating_add(n).min(self.data.len());
        self.at = self.at.saturating_add(n);
        &self.data[start..end]
    }
    /// Consumes a length selector and charges available input bytes to the
    /// shared text budget before replacing invalid UTF-8 sequences.
    fn text(&mut self, budget: &mut usize) -> String {
        let requested = usize::from(self.byte()) % (MAX_TEXT + 1);
        let bytes = self.take(requested.min(*budget));
        *budget -= bytes.len();
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// Generates JSON with a shared node and input-text budget, limiting nesting
/// to eight levels without parsing or validating production wire syntax.
fn json_value(cursor: &mut Cursor<'_>, depth: usize, nodes: &mut usize, text_bytes_left: &mut usize) -> JsonValue {
    if *nodes == 0 {
        return JsonValue::Null;
    }
    *nodes -= 1;
    match cursor.byte() % if depth >= 8 { 5 } else { 7 } {
        0 => JsonValue::Null,
        1 => JsonValue::Bool(cursor.byte() & 1 != 0),
        2 => json!(i64::from(cursor.byte()) - 128),
        3 => json!((i16::from(cursor.byte()) - 128) as f64 / 10.0),
        4 => JsonValue::String(cursor.text(text_bytes_left)),
        5 if depth < 8 => {
            let len = usize::from(cursor.byte() % 9);
            let mut out = Vec::new();
            for _ in 0..len {
                if *nodes == 0 {
                    break;
                }
                out.push(json_value(cursor, depth + 1, nodes, text_bytes_left));
            }
            JsonValue::Array(out)
        }
        6 if depth < 8 => {
            let len = usize::from(cursor.byte() % 5);
            let mut out = JsonMap::new();
            for i in 0..len {
                if *nodes == 0 {
                    break;
                }
                let key = format!("{}-{i}", cursor.text(text_bytes_left));
                out.insert(key, json_value(cursor, depth + 1, nodes, text_bytes_left));
            }
            JsonValue::Object(out)
        }
        _ => JsonValue::Null,
    }
}

/// Copies at most `N` fuzz bytes into a fixed-width little-endian buffer.
#[must_use]
#[inline]
fn padded_bytes<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut bytes = [0_u8; N];
    let copy_len = data.len().min(N);
    bytes[..copy_len].copy_from_slice(&data[..copy_len]);
    bytes
}

/// Derives a deterministic scalar seed from the first fuzz bytes.
#[must_use]
#[inline]
fn seed(data: &[u8]) -> u64 {
    u64::from_le_bytes(padded_bytes(data))
}

/// Converts a bounded suffix of fuzz input into valid UTF-8 text.
#[must_use]
fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data.get(1..data.len().min(25)).unwrap_or_default()).into_owned()
}

/// Maps a seed to a date inside a fixed one-year fuzzing window.
#[must_use]
#[inline]
fn date(seed: u64) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 1)
        .expect("fixed fuzz date must be valid")
        .checked_add_days(Days::new(seed % 365))
        .expect("fuzz date offset must remain in range")
}

/// Maps a seed to a valid time-of-day value.
#[must_use]
#[inline]
fn time(seed: u64) -> NaiveTime {
    NaiveTime::from_hms_nano_opt(
        (seed % 24) as u32,
        ((seed / 24) % 60) as u32,
        ((seed / (24 * 60)) % 60) as u32,
        (seed % 1_000_000_000) as u32,
    )
    .expect("fuzz time components must be valid")
}

/// Builds a normalized URL from bounded selectors over known valid components.
#[must_use]
fn url(data: &[u8]) -> Url {
    let pick = |index: usize| data.get(index).copied().unwrap_or_default();
    let scheme = if pick(9) & 1 == 0 { "http" } else { "https" };
    let host = ["example.com", "sub.example.com", "localhost"][usize::from(pick(10) % 3)];
    let port = match pick(11) % 4 {
        0 => "",
        1 => ":8080",
        2 if scheme == "http" => ":80",
        2 => ":443",
        _ => ":3000",
    };
    let path = ["fuzz", "a/b", "", "items/1"][usize::from(pick(12) % 4)];
    Url::parse(&format!("{scheme}://{host}{port}/{path}")).expect("generated URL components form a valid URL")
}

/// Builds a small, valid runtime shape from the fuzz input bytes. Keeping the
/// shape construction typed means the fuzz target exercises the wire encoder
/// and decoder independently from arbitrary JSON syntax handling.
pub fn container_from_bytes(data: &[u8]) -> ValueContainer {
    let bounded = &data[..data.len().min(MAX_INPUT)];
    let tag = bounded.first().copied().unwrap_or_default() % TAG_COUNT;
    let seed = seed(bounded);
    let mut cursor = Cursor::new(bounded);
    cursor.at = 5;
    let collection_len = usize::from(cursor.byte() % 9);
    let object_width = usize::from(cursor.byte() % 5);
    let mut nodes = MAX_NODES;
    let mut text_bytes_left = MAX_TEXT;
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
    let date = match bounded.get(8).copied().unwrap_or_default() % 4 {
        0 => NaiveDate::from_ymd_opt(2024, 2, 29).expect("leap day is valid"),
        1 => NaiveDate::from_ymd_opt(0, 1, 1).expect("year zero is valid"),
        2 => NaiveDate::from_ymd_opt(9999, 12, 31).expect("upper year is valid"),
        _ => date(seed),
    };
    let time = time(seed);
    let date_time = date.and_time(time);
    let instant = Utc.from_utc_datetime(&date_time);
    let duration = Duration::new(seed % 1_000_000, (seed % 1_000_000_000) as u32);
    let big_integer = BigInt::from(i128::from_le_bytes(padded_bytes(data)));
    let scale = match cursor.byte() % 5 {
        0 => -150_000,
        1 => -1,
        2 => 0,
        3 => 1,
        _ => 150_000,
    };
    let big_decimal = BigDecimal::new(big_integer.clone(), scale);
    let url = url(bounded);
    let mut string_map = HashMap::new();
    for i in 0..object_width {
        string_map.insert(format!("key-{i}"), if i == 0 { text.clone() } else { seed.to_string() });
    }
    let json = json_value(&mut cursor, 0, &mut nodes, &mut text_bytes_left);
    let character = char::from_u32((seed % 0x11_0000) as u32).unwrap_or('🦀');
    let float32 = float32(bounded, number);
    let float64 = float64(bounded, number64);

    match tag {
        0 => ValueContainer::Scalar(Value::Bool(data.get(1).copied().unwrap_or_default() & 1 == 1)),
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
        25 => ValueContainer::Scalar(Value::new_unset(Value::Int32(0).data_type())),
        26 => ValueContainer::Collection(MultiValues::Bool(vec![true; collection_len])),
        27 => ValueContainer::Collection(MultiValues::Char(vec![character; collection_len])),
        28 => ValueContainer::Collection(MultiValues::Int8(vec![number as i8; collection_len])),
        29 => ValueContainer::Collection(MultiValues::Int16(vec![number as i16; collection_len])),
        30 => ValueContainer::Collection(MultiValues::Int32(vec![number; collection_len])),
        31 => ValueContainer::Collection(MultiValues::Int64(vec![number64; collection_len])),
        32 => ValueContainer::Collection(MultiValues::Int128(vec![number128; collection_len])),
        33 => ValueContainer::Collection(MultiValues::UInt8(vec![seed as u8; collection_len])),
        34 => ValueContainer::Collection(MultiValues::UInt16(vec![seed as u16; collection_len])),
        35 => ValueContainer::Collection(MultiValues::UInt32(vec![seed as u32; collection_len])),
        36 => ValueContainer::Collection(MultiValues::UInt64(vec![unsigned_number; collection_len])),
        37 => ValueContainer::Collection(MultiValues::UInt128(vec![unsigned_number128; collection_len])),
        38 => ValueContainer::Collection(MultiValues::Float32(vec![float32; collection_len])),
        39 => ValueContainer::Collection(MultiValues::Float64(vec![float64; collection_len])),
        40 => ValueContainer::Collection(MultiValues::BigInteger(vec![big_integer; collection_len])),
        41 => ValueContainer::Collection(MultiValues::BigDecimal(vec![big_decimal; collection_len])),
        42 => ValueContainer::Collection(MultiValues::String(vec![text; collection_len])),
        43 => ValueContainer::Collection(MultiValues::Date(vec![date; collection_len])),
        44 => ValueContainer::Collection(MultiValues::Time(vec![time; collection_len])),
        45 => ValueContainer::Collection(MultiValues::DateTime(vec![date_time; collection_len])),
        46 => ValueContainer::Collection(MultiValues::Instant(vec![instant; collection_len])),
        47 => ValueContainer::Collection(MultiValues::Duration(vec![duration; collection_len])),
        48 => ValueContainer::Collection(MultiValues::Url(vec![url; collection_len])),
        49 => ValueContainer::Collection(MultiValues::StringMap(vec![string_map; collection_len])),
        50 => {
            let mut values = Vec::with_capacity(collection_len);
            for _ in 0..collection_len {
                if nodes == 0 {
                    break;
                }
                values.push(json_value(&mut cursor, 0, &mut nodes, &mut text_bytes_left));
            }
            ValueContainer::Collection(MultiValues::Json(values))
        }
        _ => ValueContainer::Collection(MultiValues::new_unset(MultiValues::Int32(Vec::new()).data_type())),
    }
}
