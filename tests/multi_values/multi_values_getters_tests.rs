// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::ValueError;

#[test]
fn test_multi_values_getters_return_slices_without_copying() {
    let values = MultiValues::String(vec!["red".to_string(), "blue".to_string()]);
    let strings = values.get_strings().unwrap();
    assert_eq!(strings, &["red", "blue"]);
    assert_eq!(strings.len(), values.len());
}

#[test]
fn test_multi_values_getters_distinguish_unset_from_concrete_empty() {
    let unset = MultiValues::Unset(DataType::Int32);
    assert!(matches!(unset.get::<i32>(), Err(ValueError::Missing(_))));
    assert!(matches!(unset.get_int32s(), Err(ValueError::Missing(_))));

    let empty = MultiValues::Int32(Vec::new());
    assert_eq!(empty.get::<i32>(), Ok(Vec::new()));
    assert_eq!(empty.get_int32s(), Ok(&[][..]));
}

#[test]
fn test_multi_values_typed_getters_cover_builtin_variants() {
    let values = MultiValues::Bool(vec![true, false]);
    assert_eq!(values.get_bools().unwrap(), &[true, false]);
    assert!(values.get_first_bool().unwrap());
    let values = MultiValues::Char(vec!['a', 'b']);
    assert_eq!(values.get_chars().unwrap(), &['a', 'b']);
    assert_eq!(values.get_first_char().unwrap(), 'a');
    macro_rules! check_numeric {
        ($variant:ident, $all:ident, $first:ident, $ty:ty, $a:expr, $b:expr) => {{
            let values = MultiValues::$variant(vec![$a, $b]);
            assert_eq!(values.$all().unwrap(), &[$a, $b]);
            assert_eq!(values.$first().unwrap(), $a);
        }};
    }
    check_numeric!(Int8, get_int8s, get_first_int8, i8, -1, 2);
    check_numeric!(Int16, get_int16s, get_first_int16, i16, -1, 2);
    check_numeric!(Int32, get_int32s, get_first_int32, i32, -1, 2);
    check_numeric!(Int64, get_int64s, get_first_int64, i64, -1, 2);
    check_numeric!(Int128, get_int128s, get_first_int128, i128, -1, 2);
    check_numeric!(UInt8, get_uint8s, get_first_uint8, u8, 1, 2);
    check_numeric!(UInt16, get_uint16s, get_first_uint16, u16, 1, 2);
    check_numeric!(UInt32, get_uint32s, get_first_uint32, u32, 1, 2);
    check_numeric!(UInt64, get_uint64s, get_first_uint64, u64, 1, 2);
    check_numeric!(UInt128, get_uint128s, get_first_uint128, u128, 1, 2);
    check_numeric!(Float32, get_float32s, get_first_float32, f32, 1.5, 2.5);
    check_numeric!(Float64, get_float64s, get_first_float64, f64, 1.5, 2.5);
    let values = MultiValues::String(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(values.get_strings().unwrap(), &["a", "b"]);
    assert_eq!(values.get_first_string().unwrap(), "a");
    let values = MultiValues::Duration(vec![std::time::Duration::from_secs(1)]);
    assert_eq!(values.get_durations().unwrap().len(), 1);
    assert_eq!(
        values.get_first_duration().unwrap(),
        std::time::Duration::from_secs(1)
    );
    let map = std::collections::HashMap::from([("key".to_string(), "value".to_string())]);
    let values = MultiValues::StringMap(vec![map.clone()]);
    assert_eq!(
        values.get_string_maps().unwrap(),
        std::slice::from_ref(&map)
    );
    assert_eq!(values.get_first_string_map().unwrap(), map);

    #[cfg(feature = "chrono")]
    {
        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let time = chrono::NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        let datetime = date.and_time(time);
        let instant =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(datetime, chrono::Utc);
        let values = MultiValues::Date(vec![date]);
        assert_eq!(values.get_dates().unwrap(), &[date]);
        assert_eq!(values.get_first_date().unwrap(), date);
        let values = MultiValues::Time(vec![time]);
        assert_eq!(values.get_times().unwrap(), &[time]);
        assert_eq!(values.get_first_time().unwrap(), time);
        let values = MultiValues::DateTime(vec![datetime]);
        assert_eq!(values.get_datetimes().unwrap(), &[datetime]);
        assert_eq!(values.get_first_datetime().unwrap(), datetime);
        let values = MultiValues::Instant(vec![instant]);
        assert_eq!(values.get_instants().unwrap(), &[instant]);
        assert_eq!(values.get_first_instant().unwrap(), instant);
    }

    #[cfg(feature = "big-integer")]
    {
        let value = num_bigint::BigInt::from(7);
        let values = MultiValues::BigInteger(vec![value.clone()]);
        assert_eq!(
            values.get_bigintegers().unwrap(),
            std::slice::from_ref(&value)
        );
        assert_eq!(values.get_first_biginteger().unwrap(), value);
    }
    #[cfg(feature = "big-decimal")]
    {
        let value = "7.5".parse::<bigdecimal::BigDecimal>().unwrap();
        let values = MultiValues::BigDecimal(vec![value.clone()]);
        assert_eq!(
            values.get_bigdecimals().unwrap(),
            std::slice::from_ref(&value)
        );
        assert_eq!(values.get_first_bigdecimal().unwrap(), value);
    }
    #[cfg(feature = "url")]
    {
        let value = url::Url::parse("https://example.com").unwrap();
        let values = MultiValues::Url(vec![value.clone()]);
        assert_eq!(values.get_urls().unwrap(), std::slice::from_ref(&value));
        assert_eq!(values.get_first_url().unwrap(), value);
    }
    #[cfg(feature = "json")]
    {
        let value = serde_json::json!({"key": 1});
        let values = MultiValues::Json(vec![value.clone()]);
        assert_eq!(values.get_jsons().unwrap(), std::slice::from_ref(&value));
        assert_eq!(values.get_first_json().unwrap(), value);
    }
}
