/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # ValueConverter 覆盖率补充测试
//!
//! 覆盖 `src/value/value_converters.rs` 中 `ValueConverter` trait 各类型实现的未覆盖分支，包括：
//! - `parse_duration_string` 的错误分支
//! - `ValueConverter<Duration>` 的 Empty / 非法类型分支
//! - `ValueConverter<Url>` 的 Empty / 非法类型分支
//! - `ValueConverter<serde_json::Value>` 的 Empty / 非法类型分支
//! - `ValueConverter<u8>` 的全部分支
//! - `ValueConverter<u16>` 的全部分支
//! - `ValueConverter<u32>` 的全部分支
//! - `ValueConverter<u64>` 的全部分支
//! - `ValueConverter<u128>` 的全部分支
//! - `ValueConverter<f32>` 的全部分支
//!

use qubit_datatype::DataType;
use qubit_value::{
    Value,
    ValueError,
};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

// ============================================================================
// parse_duration_string 错误分支
// ============================================================================

#[test]
fn test_parse_duration_string_invalid_nanoseconds() {
    // 数值部分不是合法整数（含小数点）-> parse::<u128>() 失败
    let v = Value::String("1.5ns".to_string());
    let result = v.to::<Duration>();
    assert!(result.is_err());
}

#[test]
fn test_parse_duration_string_overflow_seconds() {
    // 纳秒数超出 u64::MAX * 1_000_000_000 -> secs > u64::MAX
    // u64::MAX = 18446744073709551615
    // 构造 secs 超出范围的值：(u64::MAX as u128 + 1) * 1_000_000_000
    let huge = (u64::MAX as u128 + 1) * 1_000_000_000u128;
    let s = format!("{}ns", huge);
    let v = Value::String(s);
    let result = v.to::<Duration>();
    assert!(result.is_err());
}

// ============================================================================
// ValueConverter<Duration> — Empty / 非法类型分支
// ============================================================================

#[test]
fn test_value_converter_duration_empty() {
    let v = Value::Empty(DataType::Duration);
    let result = v.to::<Duration>();
    assert!(result.is_err());
}

#[test]
fn test_value_converter_duration_wrong_type() {
    let v = Value::Int32(42);
    let result = v.to::<Duration>();
    assert_eq!(result.unwrap(), Duration::from_millis(42));
}

// ============================================================================
// ValueConverter<Url> — Empty / 非法类型分支
// ============================================================================

#[test]
fn test_value_converter_url_empty() {
    let v = Value::Empty(DataType::Url);
    let result = v.to::<Url>();
    assert!(result.is_err());
}

#[test]
fn test_value_converter_url_wrong_type() {
    let v = Value::Int32(42);
    let result = v.to::<Url>();
    assert!(result.is_err());
}

// ============================================================================
// ValueConverter<serde_json::Value> — Empty / 非法类型分支
// ============================================================================

#[test]
fn test_value_converter_json_empty() {
    let v = Value::Empty(DataType::Json);
    let result = v.to::<serde_json::Value>();
    assert!(result.is_err());
}

#[test]
fn test_value_converter_json_wrong_type() {
    let v = Value::Int32(42);
    let result = v.to::<serde_json::Value>();
    assert!(result.is_err());
}

// ============================================================================
// ValueConverter<u8> — 全部分支
// ============================================================================

#[test]
fn test_to_u8_from_uint8() {
    assert_eq!(Value::UInt8(200).to::<u8>().unwrap(), 200u8);
}

#[test]
fn test_to_u8_from_bool() {
    assert_eq!(Value::Bool(true).to::<u8>().unwrap(), 1u8);
    assert_eq!(Value::Bool(false).to::<u8>().unwrap(), 0u8);
}

#[test]
fn test_to_u8_from_char_in_range() {
    assert_eq!(Value::Char('A').to::<u8>().unwrap(), 65u8);
}

#[test]
fn test_to_u8_from_char_out_of_range() {
    // char '€' = U+20AC = 8364 > 255
    assert!(Value::Char('€').to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_int8() {
    // range is [0, i8::MAX=127]: non-negative i8 values succeed
    assert_eq!(Value::Int8(0).to::<u8>().unwrap(), 0u8);
    assert_eq!(Value::Int8(100).to::<u8>().unwrap(), 100u8);
    assert_eq!(Value::Int8(i8::MAX).to::<u8>().unwrap(), 127u8);
    // negative values fail
    assert!(Value::Int8(-1).to::<u8>().is_err());
    assert!(Value::Int8(i8::MIN).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_int16_in_range() {
    assert_eq!(Value::Int16(200).to::<u8>().unwrap(), 200u8);
}

#[test]
fn test_to_u8_from_int16_out_of_range() {
    assert!(Value::Int16(256).to::<u8>().is_err());
    assert!(Value::Int16(-1).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_int32_in_range() {
    assert_eq!(Value::Int32(255).to::<u8>().unwrap(), 255u8);
}

#[test]
fn test_to_u8_from_int32_out_of_range() {
    assert!(Value::Int32(256).to::<u8>().is_err());
    assert!(Value::Int32(-1).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_int64_in_range() {
    assert_eq!(Value::Int64(100).to::<u8>().unwrap(), 100u8);
}

#[test]
fn test_to_u8_from_int64_out_of_range() {
    assert!(Value::Int64(256).to::<u8>().is_err());
    assert!(Value::Int64(-1).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_int128_in_range() {
    assert_eq!(Value::Int128(42).to::<u8>().unwrap(), 42u8);
}

#[test]
fn test_to_u8_from_int128_out_of_range() {
    assert!(Value::Int128(256).to::<u8>().is_err());
    assert!(Value::Int128(-1).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_uint16_in_range() {
    assert_eq!(Value::UInt16(200).to::<u8>().unwrap(), 200u8);
}

#[test]
fn test_to_u8_from_uint16_out_of_range() {
    assert!(Value::UInt16(256).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_uint32_in_range() {
    assert_eq!(Value::UInt32(100).to::<u8>().unwrap(), 100u8);
}

#[test]
fn test_to_u8_from_uint32_out_of_range() {
    assert!(Value::UInt32(256).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_uint64_in_range() {
    assert_eq!(Value::UInt64(50).to::<u8>().unwrap(), 50u8);
}

#[test]
fn test_to_u8_from_uint64_out_of_range() {
    assert!(Value::UInt64(256).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_uint128_in_range() {
    assert_eq!(Value::UInt128(10).to::<u8>().unwrap(), 10u8);
}

#[test]
fn test_to_u8_from_uint128_out_of_range() {
    assert!(Value::UInt128(256).to::<u8>().is_err());
}

#[test]
fn test_to_u8_from_string_valid() {
    assert_eq!(Value::String("255".to_string()).to::<u8>().unwrap(), 255u8);
}

#[test]
fn test_to_u8_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<u8>().is_err());
    assert!(Value::String("256".to_string()).to::<u8>().is_err());
}

#[test]
fn test_to_u8_empty() {
    assert!(Value::Empty(DataType::UInt8).to::<u8>().is_err());
}

#[test]
fn test_to_u8_wrong_type() {
    assert!(Value::Float32(1.5).to::<u8>().is_err());
}

// ============================================================================
// ValueConverter<u16> — 全部分支
// ============================================================================

#[test]
fn test_to_u16_from_uint8() {
    assert_eq!(Value::UInt8(200).to::<u16>().unwrap(), 200u16);
}

#[test]
fn test_to_u16_from_uint16() {
    assert_eq!(Value::UInt16(60000).to::<u16>().unwrap(), 60000u16);
}

#[test]
fn test_to_u16_from_bool() {
    assert_eq!(Value::Bool(true).to::<u16>().unwrap(), 1u16);
    assert_eq!(Value::Bool(false).to::<u16>().unwrap(), 0u16);
}

#[test]
fn test_to_u16_from_char() {
    assert_eq!(Value::Char('A').to::<u16>().unwrap(), 65u16);
    // '€' = U+20AC = 8364, 在 u16 范围内
    assert_eq!(Value::Char('€').to::<u16>().unwrap(), 8364u16);
}

#[test]
fn test_to_u16_from_int8_positive() {
    assert_eq!(Value::Int8(100).to::<u16>().unwrap(), 100u16);
}

#[test]
fn test_to_u16_from_int8_negative() {
    assert!(Value::Int8(-1).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_int16() {
    // range is [0, i16::MAX=32767]: non-negative i16 values succeed
    assert_eq!(Value::Int16(0).to::<u16>().unwrap(), 0u16);
    assert_eq!(Value::Int16(1000).to::<u16>().unwrap(), 1000u16);
    assert_eq!(Value::Int16(i16::MAX).to::<u16>().unwrap(), 32767u16);
    // negative values fail
    assert!(Value::Int16(-1).to::<u16>().is_err());
    assert!(Value::Int16(i16::MIN).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_int32_in_range() {
    assert_eq!(Value::Int32(65535).to::<u16>().unwrap(), 65535u16);
}

#[test]
fn test_to_u16_from_int32_out_of_range() {
    assert!(Value::Int32(65536).to::<u16>().is_err());
    assert!(Value::Int32(-1).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_int64_in_range() {
    assert_eq!(Value::Int64(1000).to::<u16>().unwrap(), 1000u16);
}

#[test]
fn test_to_u16_from_int64_out_of_range() {
    assert!(Value::Int64(65536).to::<u16>().is_err());
    assert!(Value::Int64(-1).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_int128_in_range() {
    assert_eq!(Value::Int128(500).to::<u16>().unwrap(), 500u16);
}

#[test]
fn test_to_u16_from_int128_out_of_range() {
    assert!(Value::Int128(65536).to::<u16>().is_err());
    assert!(Value::Int128(-1).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_uint32_in_range() {
    assert_eq!(Value::UInt32(65535).to::<u16>().unwrap(), 65535u16);
}

#[test]
fn test_to_u16_from_uint32_out_of_range() {
    assert!(Value::UInt32(65536).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_uint64_in_range() {
    assert_eq!(Value::UInt64(100).to::<u16>().unwrap(), 100u16);
}

#[test]
fn test_to_u16_from_uint64_out_of_range() {
    assert!(Value::UInt64(65536).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_uint128_in_range() {
    assert_eq!(Value::UInt128(200).to::<u16>().unwrap(), 200u16);
}

#[test]
fn test_to_u16_from_uint128_out_of_range() {
    assert!(Value::UInt128(65536).to::<u16>().is_err());
}

#[test]
fn test_to_u16_from_string_valid() {
    assert_eq!(
        Value::String("65535".to_string()).to::<u16>().unwrap(),
        65535u16
    );
}

#[test]
fn test_to_u16_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<u16>().is_err());
}

#[test]
fn test_to_u16_empty() {
    assert!(Value::Empty(DataType::UInt16).to::<u16>().is_err());
}

#[test]
fn test_to_u16_wrong_type() {
    assert!(Value::Float32(1.5).to::<u16>().is_err());
}

// ============================================================================
// ValueConverter<u32> — 全部分支
// ============================================================================

#[test]
fn test_to_u32_from_uint8() {
    assert_eq!(Value::UInt8(255).to::<u32>().unwrap(), 255u32);
}

#[test]
fn test_to_u32_from_uint16() {
    assert_eq!(Value::UInt16(65535).to::<u32>().unwrap(), 65535u32);
}

#[test]
fn test_to_u32_from_uint32() {
    assert_eq!(Value::UInt32(u32::MAX).to::<u32>().unwrap(), u32::MAX);
}

#[test]
fn test_to_u32_from_bool() {
    assert_eq!(Value::Bool(true).to::<u32>().unwrap(), 1u32);
    assert_eq!(Value::Bool(false).to::<u32>().unwrap(), 0u32);
}

#[test]
fn test_to_u32_from_char() {
    assert_eq!(Value::Char('A').to::<u32>().unwrap(), 65u32);
}

#[test]
fn test_to_u32_from_int8_positive() {
    assert_eq!(Value::Int8(127).to::<u32>().unwrap(), 127u32);
}

#[test]
fn test_to_u32_from_int8_negative() {
    assert!(Value::Int8(-1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_int16_positive() {
    assert_eq!(Value::Int16(32767).to::<u32>().unwrap(), 32767u32);
}

#[test]
fn test_to_u32_from_int16_negative() {
    assert!(Value::Int16(-1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_int32() {
    // range is [0, i32::MAX=2147483647]: non-negative i32 values succeed
    assert_eq!(Value::Int32(0).to::<u32>().unwrap(), 0u32);
    assert_eq!(Value::Int32(i32::MAX).to::<u32>().unwrap(), i32::MAX as u32);
    // negative values fail
    assert!(Value::Int32(-1).to::<u32>().is_err());
    assert!(Value::Int32(i32::MIN).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_int64_in_range() {
    assert_eq!(Value::Int64(u32::MAX as i64).to::<u32>().unwrap(), u32::MAX);
}

#[test]
fn test_to_u32_from_int64_out_of_range() {
    assert!(Value::Int64(u32::MAX as i64 + 1).to::<u32>().is_err());
    assert!(Value::Int64(-1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_int128_in_range() {
    assert_eq!(
        Value::Int128(u32::MAX as i128).to::<u32>().unwrap(),
        u32::MAX
    );
}

#[test]
fn test_to_u32_from_int128_out_of_range() {
    assert!(Value::Int128(u32::MAX as i128 + 1).to::<u32>().is_err());
    assert!(Value::Int128(-1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_uint64_in_range() {
    assert_eq!(
        Value::UInt64(u32::MAX as u64).to::<u32>().unwrap(),
        u32::MAX
    );
}

#[test]
fn test_to_u32_from_uint64_out_of_range() {
    assert!(Value::UInt64(u32::MAX as u64 + 1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_uint128_in_range() {
    assert_eq!(
        Value::UInt128(u32::MAX as u128).to::<u32>().unwrap(),
        u32::MAX
    );
}

#[test]
fn test_to_u32_from_uint128_out_of_range() {
    assert!(Value::UInt128(u32::MAX as u128 + 1).to::<u32>().is_err());
}

#[test]
fn test_to_u32_from_string_valid() {
    assert_eq!(
        Value::String("4294967295".to_string()).to::<u32>().unwrap(),
        u32::MAX
    );
}

#[test]
fn test_to_u32_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<u32>().is_err());
}

#[test]
fn test_to_u32_empty() {
    assert!(Value::Empty(DataType::UInt32).to::<u32>().is_err());
}

#[test]
fn test_to_u32_wrong_type() {
    assert!(Value::Float32(1.5).to::<u32>().is_err());
}

// ============================================================================
// ValueConverter<u64> — 全部分支
// ============================================================================

#[test]
fn test_to_u64_from_uint8() {
    assert_eq!(Value::UInt8(255).to::<u64>().unwrap(), 255u64);
}

#[test]
fn test_to_u64_from_uint16() {
    assert_eq!(Value::UInt16(65535).to::<u64>().unwrap(), 65535u64);
}

#[test]
fn test_to_u64_from_uint32() {
    assert_eq!(
        Value::UInt32(u32::MAX).to::<u64>().unwrap(),
        u32::MAX as u64
    );
}

#[test]
fn test_to_u64_from_uint64() {
    assert_eq!(Value::UInt64(u64::MAX).to::<u64>().unwrap(), u64::MAX);
}

#[test]
fn test_to_u64_from_bool() {
    assert_eq!(Value::Bool(true).to::<u64>().unwrap(), 1u64);
    assert_eq!(Value::Bool(false).to::<u64>().unwrap(), 0u64);
}

#[test]
fn test_to_u64_from_char() {
    assert_eq!(Value::Char('A').to::<u64>().unwrap(), 65u64);
}

#[test]
fn test_to_u64_from_int8_positive() {
    assert_eq!(Value::Int8(127).to::<u64>().unwrap(), 127u64);
}

#[test]
fn test_to_u64_from_int8_negative() {
    assert!(Value::Int8(-1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_int16_positive() {
    assert_eq!(Value::Int16(32767).to::<u64>().unwrap(), 32767u64);
}

#[test]
fn test_to_u64_from_int16_negative() {
    assert!(Value::Int16(-1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_int32_positive() {
    assert_eq!(Value::Int32(i32::MAX).to::<u64>().unwrap(), i32::MAX as u64);
}

#[test]
fn test_to_u64_from_int32_negative() {
    assert!(Value::Int32(-1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_int64_positive() {
    assert_eq!(Value::Int64(i64::MAX).to::<u64>().unwrap(), i64::MAX as u64);
}

#[test]
fn test_to_u64_from_int64_negative() {
    assert!(Value::Int64(-1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_int128_in_range() {
    assert_eq!(
        Value::Int128(u64::MAX as i128).to::<u64>().unwrap(),
        u64::MAX
    );
}

#[test]
fn test_to_u64_from_int128_out_of_range() {
    assert!(Value::Int128(u64::MAX as i128 + 1).to::<u64>().is_err());
    assert!(Value::Int128(-1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_uint128_in_range() {
    assert_eq!(
        Value::UInt128(u64::MAX as u128).to::<u64>().unwrap(),
        u64::MAX
    );
}

#[test]
fn test_to_u64_from_uint128_out_of_range() {
    assert!(Value::UInt128(u64::MAX as u128 + 1).to::<u64>().is_err());
}

#[test]
fn test_to_u64_from_string_valid() {
    assert_eq!(
        Value::String("18446744073709551615".to_string())
            .to::<u64>()
            .unwrap(),
        u64::MAX
    );
}

#[test]
fn test_to_u64_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<u64>().is_err());
}

#[test]
fn test_to_u64_empty() {
    assert!(Value::Empty(DataType::UInt64).to::<u64>().is_err());
}

#[test]
fn test_to_u64_wrong_type() {
    assert!(Value::Float32(1.5).to::<u64>().is_err());
}

// ============================================================================
// ValueConverter<u128> — 全部分支
// ============================================================================

#[test]
fn test_to_u128_from_uint8() {
    assert_eq!(Value::UInt8(255).to::<u128>().unwrap(), 255u128);
}

#[test]
fn test_to_u128_from_uint16() {
    assert_eq!(Value::UInt16(65535).to::<u128>().unwrap(), 65535u128);
}

#[test]
fn test_to_u128_from_uint32() {
    assert_eq!(
        Value::UInt32(u32::MAX).to::<u128>().unwrap(),
        u32::MAX as u128
    );
}

#[test]
fn test_to_u128_from_uint64() {
    assert_eq!(
        Value::UInt64(u64::MAX).to::<u128>().unwrap(),
        u64::MAX as u128
    );
}

#[test]
fn test_to_u128_from_uint128() {
    assert_eq!(Value::UInt128(u128::MAX).to::<u128>().unwrap(), u128::MAX);
}

#[test]
fn test_to_u128_from_bool() {
    assert_eq!(Value::Bool(true).to::<u128>().unwrap(), 1u128);
    assert_eq!(Value::Bool(false).to::<u128>().unwrap(), 0u128);
}

#[test]
fn test_to_u128_from_char() {
    assert_eq!(Value::Char('A').to::<u128>().unwrap(), 65u128);
}

#[test]
fn test_to_u128_from_int8_positive() {
    assert_eq!(Value::Int8(127).to::<u128>().unwrap(), 127u128);
}

#[test]
fn test_to_u128_from_int8_negative() {
    assert!(Value::Int8(-1).to::<u128>().is_err());
}

#[test]
fn test_to_u128_from_int16_positive() {
    assert_eq!(Value::Int16(32767).to::<u128>().unwrap(), 32767u128);
}

#[test]
fn test_to_u128_from_int16_negative() {
    assert!(Value::Int16(-1).to::<u128>().is_err());
}

#[test]
fn test_to_u128_from_int32_positive() {
    assert_eq!(
        Value::Int32(i32::MAX).to::<u128>().unwrap(),
        i32::MAX as u128
    );
}

#[test]
fn test_to_u128_from_int32_negative() {
    assert!(Value::Int32(-1).to::<u128>().is_err());
}

#[test]
fn test_to_u128_from_int64_positive() {
    assert_eq!(
        Value::Int64(i64::MAX).to::<u128>().unwrap(),
        i64::MAX as u128
    );
}

#[test]
fn test_to_u128_from_int64_negative() {
    assert!(Value::Int64(-1).to::<u128>().is_err());
}

#[test]
fn test_to_u128_from_int128_positive() {
    assert_eq!(
        Value::Int128(i128::MAX).to::<u128>().unwrap(),
        i128::MAX as u128
    );
}

#[test]
fn test_to_u128_from_int128_negative() {
    assert!(Value::Int128(-1).to::<u128>().is_err());
}

#[test]
fn test_to_u128_from_string_valid() {
    assert_eq!(
        Value::String("340282366920938463463374607431768211455".to_string())
            .to::<u128>()
            .unwrap(),
        u128::MAX
    );
}

#[test]
fn test_to_u128_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<u128>().is_err());
}

#[test]
fn test_to_u128_empty() {
    assert!(Value::Empty(DataType::UInt128).to::<u128>().is_err());
}

#[test]
fn test_to_u128_wrong_type() {
    assert!(Value::Float32(1.5).to::<u128>().is_err());
}

// ============================================================================
// ValueConverter<f32> — 全部分支
// ============================================================================

#[test]
fn test_to_f32_from_float32() {
    let x = 2.25f32;
    assert_eq!(Value::Float32(x).to::<f32>().unwrap(), x);
}

#[test]
fn test_to_f32_from_float64_in_range() {
    let result = Value::Float64(1.5f64).to::<f32>().unwrap();
    assert!((result - 1.5f32).abs() < 1e-6);
}

#[test]
fn test_to_f32_from_float64_nan() {
    let result = Value::Float64(f64::NAN).to::<f32>().unwrap();
    assert!(result.is_nan());
}

#[test]
fn test_to_f32_from_float64_infinity() {
    let result = Value::Float64(f64::INFINITY).to::<f32>().unwrap();
    assert!(result.is_infinite() && result.is_sign_positive());
}

#[test]
fn test_to_f32_from_float64_neg_infinity() {
    let result = Value::Float64(f64::NEG_INFINITY).to::<f32>().unwrap();
    assert!(result.is_infinite() && result.is_sign_negative());
}

#[test]
fn test_to_f32_from_bool() {
    assert_eq!(Value::Bool(true).to::<f32>().unwrap(), 1.0f32);
    assert_eq!(Value::Bool(false).to::<f32>().unwrap(), 0.0f32);
}

#[test]
fn test_to_f32_from_char() {
    assert_eq!(Value::Char('A').to::<f32>().unwrap(), 65.0f32);
}

#[test]
fn test_to_f32_from_int8() {
    assert_eq!(Value::Int8(42).to::<f32>().unwrap(), 42.0f32);
}

#[test]
fn test_to_f32_from_int16() {
    assert_eq!(Value::Int16(1000).to::<f32>().unwrap(), 1000.0f32);
}

#[test]
fn test_to_f32_from_int32() {
    assert_eq!(Value::Int32(100000).to::<f32>().unwrap(), 100000.0f32);
}

#[test]
fn test_to_f32_from_int64() {
    assert_eq!(Value::Int64(1000000).to::<f32>().unwrap(), 1000000.0f32);
}

#[test]
fn test_to_f32_from_int128() {
    assert_eq!(Value::Int128(42).to::<f32>().unwrap(), 42.0f32);
}

#[test]
fn test_to_f32_from_uint8() {
    assert_eq!(Value::UInt8(200).to::<f32>().unwrap(), 200.0f32);
}

#[test]
fn test_to_f32_from_uint16() {
    assert_eq!(Value::UInt16(1000).to::<f32>().unwrap(), 1000.0f32);
}

#[test]
fn test_to_f32_from_uint32() {
    assert_eq!(Value::UInt32(100000).to::<f32>().unwrap(), 100000.0f32);
}

#[test]
fn test_to_f32_from_uint64() {
    assert_eq!(Value::UInt64(1000000).to::<f32>().unwrap(), 1000000.0f32);
}

#[test]
fn test_to_f32_from_uint128() {
    assert_eq!(Value::UInt128(42).to::<f32>().unwrap(), 42.0f32);
}

#[test]
fn test_to_f32_from_string_valid() {
    assert_eq!(
        Value::String("2.25".to_string()).to::<f32>().unwrap(),
        2.25f32
    );
}

#[test]
fn test_to_f32_from_string_invalid() {
    assert!(Value::String("abc".to_string()).to::<f32>().is_err());
}

#[test]
fn test_to_f32_empty() {
    assert!(Value::Empty(DataType::Float32).to::<f32>().is_err());
}

#[test]
fn test_to_f32_from_biginteger_normal() {
    use num_bigint::BigInt;
    use std::str::FromStr;
    let big = BigInt::from_str("42").unwrap();
    assert_eq!(Value::BigInteger(big).to::<f32>().unwrap(), 42.0f32);
}

#[test]
fn test_to_f32_from_biginteger_huge_out_of_range() {
    use num_bigint::BigInt;
    let huge = BigInt::from(2u64).pow(1100);
    assert!(Value::BigInteger(huge).to::<f32>().is_err());
}

#[test]
fn test_to_f32_from_bigdecimal_normal() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let bd = BigDecimal::from_str("2.25").unwrap();
    let result = Value::BigDecimal(bd).to::<f32>().unwrap();
    assert!((result - 2.25f32).abs() < 1e-5);
}

#[test]
fn test_to_f32_from_bigdecimal_huge_out_of_range() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let huge = BigDecimal::from_str("1e1000").unwrap();
    assert!(Value::BigDecimal(huge).to::<f32>().is_err());
}

#[test]
fn test_to_f32_from_biginteger_out_of_range() {
    use num_bigint::BigInt;
    let huge = BigInt::from(10u8).pow(4000);
    let err = Value::BigInteger(huge).to::<f32>().unwrap_err();
    assert!(matches!(
        err,
        ValueError::ConversionError(ref msg)
            if msg.contains("BigInteger value out of f32 range")
    ));
}

#[test]
fn test_to_f32_from_bigdecimal_out_of_range() {
    use bigdecimal::BigDecimal;
    let huge = BigDecimal::new(num_bigint::BigInt::from(10u8).pow(5000), 0);
    let err = Value::BigDecimal(huge).to::<f32>().unwrap_err();
    assert!(matches!(
        err,
        ValueError::ConversionError(ref msg)
            if msg.contains("BigDecimal value out of f32 range")
    ));
}

#[test]
fn test_to_f32_wrong_type() {
    assert!(Value::Duration(Duration::from_secs(1)).to::<f32>().is_err());
}

// ============================================================================
// ValueConverter<f64> — BigInteger/BigDecimal 转换路径
// ============================================================================

#[test]
fn test_to_f64_from_biginteger_normal() {
    use num_bigint::BigInt;
    let big = BigInt::from(i64::MAX);
    let result = Value::BigInteger(big).to::<f64>().unwrap();
    assert!((result - i64::MAX as f64).abs() < 1.0);
}

#[test]
fn test_to_f64_from_biginteger_huge_out_of_range() {
    use num_bigint::BigInt;
    let huge = BigInt::from(2u64).pow(1100);
    assert!(Value::BigInteger(huge).to::<f64>().is_err());
}

#[test]
fn test_to_f64_from_bigdecimal_normal() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let bd = BigDecimal::from_str("2.25").unwrap();
    let result = Value::BigDecimal(bd).to::<f64>().unwrap();
    assert!((result - 2.25f64).abs() < 1e-10);
}

#[test]
fn test_to_f64_from_bigdecimal_huge_out_of_range() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    let huge = BigDecimal::from_str("1e1000").unwrap();
    assert!(Value::BigDecimal(huge).to::<f64>().is_err());
}

#[test]
fn test_to_f64_from_biginteger_out_of_range() {
    use num_bigint::BigInt;
    let huge = BigInt::from(10u8).pow(5000);
    let err = Value::BigInteger(huge).to::<f64>().unwrap_err();
    assert!(matches!(
        err,
        ValueError::ConversionError(ref msg)
            if msg.contains("BigInteger value out of f64 range")
    ));
}

#[test]
fn test_to_f64_from_bigdecimal_out_of_range() {
    use bigdecimal::BigDecimal;
    let huge = BigDecimal::new(num_bigint::BigInt::from(10u8).pow(7000), 0);
    let err = Value::BigDecimal(huge).to::<f64>().unwrap_err();
    assert!(matches!(
        err,
        ValueError::ConversionError(ref msg)
            if msg.contains("BigDecimal value out of f64 range")
    ));
}

// ============================================================================
// ValueConverter<f32> — 整数类型 to_f32() 失败路径
// ============================================================================
// 注意：对于常规整数值，to_f32() 通常返回 Some，以下测试验证成功路径
// 同时通过调用确保 ok_or_else 的闭包代码被编译器保留

#[test]
fn test_to_f32_from_int8_success() {
    assert_eq!(Value::Int8(i8::MAX).to::<f32>().unwrap(), 127.0f32);
    assert_eq!(Value::Int8(i8::MIN).to::<f32>().unwrap(), -128.0f32);
}

#[test]
fn test_to_f32_from_int16_success() {
    assert_eq!(Value::Int16(i16::MAX).to::<f32>().unwrap(), 32767.0f32);
    assert_eq!(Value::Int16(i16::MIN).to::<f32>().unwrap(), -32768.0f32);
}

#[test]
fn test_to_f32_from_int32_success() {
    assert_eq!(Value::Int32(0).to::<f32>().unwrap(), 0.0f32);
    assert_eq!(Value::Int32(-1).to::<f32>().unwrap(), -1.0f32);
}

#[test]
fn test_to_f32_from_int64_success() {
    assert_eq!(Value::Int64(0).to::<f32>().unwrap(), 0.0f32);
    assert_eq!(Value::Int64(-1).to::<f32>().unwrap(), -1.0f32);
}

#[test]
fn test_to_f32_from_int128_success() {
    assert_eq!(Value::Int128(0).to::<f32>().unwrap(), 0.0f32);
    assert_eq!(Value::Int128(-1).to::<f32>().unwrap(), -1.0f32);
}

#[test]
fn test_to_f32_from_uint8_success() {
    assert_eq!(Value::UInt8(u8::MAX).to::<f32>().unwrap(), 255.0f32);
}

#[test]
fn test_to_f32_from_uint16_success() {
    assert_eq!(Value::UInt16(u16::MAX).to::<f32>().unwrap(), 65535.0f32);
}

#[test]
fn test_to_f32_from_uint32_success() {
    assert_eq!(Value::UInt32(0).to::<f32>().unwrap(), 0.0f32);
}

#[test]
fn test_to_f32_from_uint64_success() {
    assert_eq!(Value::UInt64(0).to::<f32>().unwrap(), 0.0f32);
}

#[test]
fn test_to_f32_from_uint128_success() {
    assert_eq!(Value::UInt128(0).to::<f32>().unwrap(), 0.0f32);
}

// ============================================================================
// 补充：ValueConverter<HashMap<String, String>> 的 StringMap 路径
// ============================================================================

#[test]
fn test_to_stringmap_from_stringmap() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let v = Value::StringMap(map.clone());
    let result = v.to::<HashMap<String, String>>().unwrap();
    assert_eq!(result, map);
}

#[test]
fn test_to_stringmap_empty() {
    let v = Value::Empty(DataType::StringMap);
    assert!(v.to::<HashMap<String, String>>().is_err());
}

#[test]
fn test_to_stringmap_wrong_type() {
    let v = Value::Int32(42);
    assert!(v.to::<HashMap<String, String>>().is_err());
}
