/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Central type table for `MultiValues` variant metadata.

macro_rules! for_each_multi_value_type {
    ($macro:ident $(, $arg:expr)*) => {
        $macro! {
            $($arg),*;
            (Bool, bool, ::qubit_datatype::DataType::Bool),
            (Char, char, ::qubit_datatype::DataType::Char),
            (Int8, i8, ::qubit_datatype::DataType::Int8),
            (Int16, i16, ::qubit_datatype::DataType::Int16),
            (Int32, i32, ::qubit_datatype::DataType::Int32),
            (Int64, i64, ::qubit_datatype::DataType::Int64),
            (Int128, i128, ::qubit_datatype::DataType::Int128),
            (UInt8, u8, ::qubit_datatype::DataType::UInt8),
            (UInt16, u16, ::qubit_datatype::DataType::UInt16),
            (UInt32, u32, ::qubit_datatype::DataType::UInt32),
            (UInt64, u64, ::qubit_datatype::DataType::UInt64),
            (UInt128, u128, ::qubit_datatype::DataType::UInt128),
            (IntSize, isize, ::qubit_datatype::DataType::IntSize),
            (UIntSize, usize, ::qubit_datatype::DataType::UIntSize),
            (Float32, f32, ::qubit_datatype::DataType::Float32),
            (Float64, f64, ::qubit_datatype::DataType::Float64),
            (BigInteger, ::num_bigint::BigInt, ::qubit_datatype::DataType::BigInteger),
            (BigDecimal, ::bigdecimal::BigDecimal, ::qubit_datatype::DataType::BigDecimal),
            (String, String, ::qubit_datatype::DataType::String),
            (Date, ::chrono::NaiveDate, ::qubit_datatype::DataType::Date),
            (Time, ::chrono::NaiveTime, ::qubit_datatype::DataType::Time),
            (DateTime, ::chrono::NaiveDateTime, ::qubit_datatype::DataType::DateTime),
            (Instant, ::chrono::DateTime<::chrono::Utc>, ::qubit_datatype::DataType::Instant),
            (Duration, ::std::time::Duration, ::qubit_datatype::DataType::Duration),
            (Url, ::url::Url, ::qubit_datatype::DataType::Url),
            (
                StringMap,
                ::std::collections::HashMap<String, String>,
                ::qubit_datatype::DataType::StringMap
            ),
            (Json, ::serde_json::Value, ::qubit_datatype::DataType::Json),
        }
    };
}
