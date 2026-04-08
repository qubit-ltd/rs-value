/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value v0.3.0 新增类型单元测试
//!
//! 覆盖 v0.3.0 新增的以下类型：
//! - `isize` / `usize`
//! - `std::time::Duration`
//! - `url::Url`
//! - `HashMap<String, String>`
//! - `serde_json::Value` (Json escape hatch)
//!
//! # Author
//!
//! Haixing Hu

use qubit_common::lang::DataType;
use qubit_value::{Value, ValueError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

// ============================================================================
// isize 测试
// ============================================================================

#[test]
fn test_value_intsize_creation() {
    let v = Value::IntSize(42isize);
    assert_eq!(v.data_type(), DataType::IntSize);
    assert!(!v.is_empty());
}

#[test]
fn test_value_intsize_get() {
    let v = Value::IntSize(100isize);
    assert_eq!(v.get_intsize().unwrap(), 100isize);
}

#[test]
fn test_value_intsize_negative() {
    let v = Value::IntSize(-999isize);
    assert_eq!(v.get_intsize().unwrap(), -999isize);
}

#[test]
fn test_value_intsize_set() {
    let mut v = Value::Empty(DataType::IntSize);
    v.set_intsize(42isize).unwrap();
    assert_eq!(v.get_intsize().unwrap(), 42isize);
}

#[test]
fn test_value_intsize_type_mismatch() {
    let v = Value::IntSize(1isize);
    assert!(matches!(
        v.get_uintsize(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_intsize_empty_error() {
    let v = Value::Empty(DataType::IntSize);
    assert!(matches!(v.get_intsize(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_intsize_generic_new() {
    let v = Value::new(42isize);
    assert_eq!(v.data_type(), DataType::IntSize);
    assert_eq!(v.get::<isize>().unwrap(), 42isize);
}

#[test]
fn test_value_intsize_generic_set() {
    let mut v = Value::IntSize(0isize);
    v.set(99isize).unwrap();
    assert_eq!(v.get_intsize().unwrap(), 99isize);
}

#[test]
fn test_value_intsize_as_string() {
    let v = Value::IntSize(-42isize);
    assert_eq!(v.to::<String>().unwrap(), "-42");
}

#[test]
fn test_value_intsize_clear() {
    let mut v = Value::IntSize(10isize);
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::IntSize);
}

#[test]
fn test_value_intsize_serde_roundtrip() {
    let original = Value::IntSize(isize::MAX);
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// usize 测试
// ============================================================================

#[test]
fn test_value_uintsize_creation() {
    let v = Value::UIntSize(42usize);
    assert_eq!(v.data_type(), DataType::UIntSize);
    assert!(!v.is_empty());
}

#[test]
fn test_value_uintsize_get() {
    let v = Value::UIntSize(999usize);
    assert_eq!(v.get_uintsize().unwrap(), 999usize);
}

#[test]
fn test_value_uintsize_set() {
    let mut v = Value::Empty(DataType::UIntSize);
    v.set_uintsize(512usize).unwrap();
    assert_eq!(v.get_uintsize().unwrap(), 512usize);
}

#[test]
fn test_value_uintsize_type_mismatch() {
    let v = Value::UIntSize(1usize);
    assert!(matches!(
        v.get_intsize(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_uintsize_generic_new() {
    let v = Value::new(8080usize);
    assert_eq!(v.data_type(), DataType::UIntSize);
    assert_eq!(v.get::<usize>().unwrap(), 8080usize);
}

#[test]
fn test_value_uintsize_as_string() {
    let v = Value::UIntSize(1024usize);
    assert_eq!(v.to::<String>().unwrap(), "1024");
}

#[test]
fn test_value_uintsize_serde_roundtrip() {
    let original = Value::UIntSize(usize::MAX);
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// Duration 测试
// ============================================================================

#[test]
fn test_value_duration_creation() {
    let d = Duration::from_secs(30);
    let v = Value::Duration(d);
    assert_eq!(v.data_type(), DataType::Duration);
    assert!(!v.is_empty());
}

#[test]
fn test_value_duration_get() {
    let d = Duration::from_millis(500);
    let v = Value::Duration(d);
    assert_eq!(v.get_duration().unwrap(), d);
}

#[test]
fn test_value_duration_set() {
    let mut v = Value::Empty(DataType::Duration);
    let d = Duration::from_secs(60);
    v.set_duration(d).unwrap();
    assert_eq!(v.get_duration().unwrap(), d);
}

#[test]
fn test_value_duration_type_mismatch() {
    let v = Value::Duration(Duration::from_secs(1));
    assert!(matches!(
        v.get_intsize(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_duration_empty_error() {
    let v = Value::Empty(DataType::Duration);
    assert!(matches!(v.get_duration(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_duration_generic_new() {
    let d = Duration::from_nanos(1_000_000);
    let v = Value::new(d);
    assert_eq!(v.data_type(), DataType::Duration);
    assert_eq!(v.get::<Duration>().unwrap(), d);
}

#[test]
fn test_value_duration_generic_set() {
    let d1 = Duration::from_secs(1);
    let d2 = Duration::from_secs(2);
    let mut v = Value::Duration(d1);
    v.set(d2).unwrap();
    assert_eq!(v.get_duration().unwrap(), d2);
}

#[test]
fn test_value_duration_as_string_nanoseconds() {
    let d = Duration::from_nanos(1_500_000_000);
    let v = Value::Duration(d);
    assert_eq!(v.to::<String>().unwrap(), "1500000000ns");
}

#[test]
fn test_value_duration_as_duration_from_duration() {
    let d = Duration::from_secs(3);
    let v = Value::Duration(d);
    assert_eq!(v.to::<Duration>().unwrap(), d);
}

#[test]
fn test_value_duration_as_duration_from_string() {
    let v = Value::String("1500000000ns".to_string());
    assert_eq!(
        v.to::<Duration>().unwrap(),
        Duration::from_nanos(1_500_000_000)
    );
}

#[test]
fn test_value_duration_as_duration_invalid_string() {
    let v = Value::String("1.5s".to_string());
    assert!(matches!(
        v.to::<Duration>(),
        Err(ValueError::ConversionError(_))
    ));
}

#[test]
fn test_value_duration_zero() {
    let v = Value::Duration(Duration::ZERO);
    assert_eq!(v.get_duration().unwrap(), Duration::ZERO);
    assert_eq!(v.to::<String>().unwrap(), "0ns");
}

#[test]
fn test_value_duration_clear() {
    let mut v = Value::Duration(Duration::from_secs(10));
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::Duration);
}

#[test]
fn test_value_duration_serde_roundtrip() {
    let original = Value::Duration(Duration::from_secs(3600));
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// Url 测试
// ============================================================================

#[test]
fn test_value_url_creation() {
    let url = Url::parse("https://example.com").unwrap();
    let v = Value::Url(url);
    assert_eq!(v.data_type(), DataType::Url);
    assert!(!v.is_empty());
}

#[test]
fn test_value_url_get() {
    let url = Url::parse("https://example.com/path?q=1").unwrap();
    let v = Value::Url(url.clone());
    assert_eq!(v.get_url().unwrap(), url);
}

#[test]
fn test_value_url_set() {
    let mut v = Value::Empty(DataType::Url);
    let url = Url::parse("http://localhost:8080").unwrap();
    v.set_url(url.clone()).unwrap();
    assert_eq!(v.get_url().unwrap(), url);
}

#[test]
fn test_value_url_type_mismatch() {
    let url = Url::parse("https://example.com").unwrap();
    let v = Value::Url(url);
    assert!(matches!(
        v.get_string(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_url_empty_error() {
    let v = Value::Empty(DataType::Url);
    assert!(matches!(v.get_url(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_url_generic_new() {
    let url = Url::parse("ftp://files.example.com").unwrap();
    let v = Value::new(url.clone());
    assert_eq!(v.data_type(), DataType::Url);
    assert_eq!(v.get::<Url>().unwrap(), url);
}

#[test]
fn test_value_url_generic_set() {
    let url1 = Url::parse("https://old.example.com").unwrap();
    let url2 = Url::parse("https://new.example.com").unwrap();
    let mut v = Value::Url(url1);
    v.set(url2.clone()).unwrap();
    assert_eq!(v.get_url().unwrap(), url2);
}

#[test]
fn test_value_url_as_string() {
    let url = Url::parse("https://example.com/path").unwrap();
    let v = Value::Url(url.clone());
    assert_eq!(v.to::<String>().unwrap(), url.to_string());
}

#[test]
fn test_value_url_as_url_from_url() {
    let url = Url::parse("https://example.com/path").unwrap();
    let v = Value::Url(url.clone());
    assert_eq!(v.to::<Url>().unwrap(), url);
}

#[test]
fn test_value_url_as_url_from_string() {
    let url = Url::parse("https://example.com/path?q=1").unwrap();
    let v = Value::String(url.to_string());
    assert_eq!(v.to::<Url>().unwrap(), url);
}

#[test]
fn test_value_url_as_url_invalid_string() {
    let v = Value::String("not-a-url".to_string());
    assert!(matches!(v.to::<Url>(), Err(ValueError::ConversionError(_))));
}

#[test]
fn test_value_url_clear() {
    let mut v = Value::Url(Url::parse("https://example.com").unwrap());
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::Url);
}

#[test]
fn test_value_url_serde_roundtrip() {
    let original = Value::Url(Url::parse("https://example.com/path?key=value#anchor").unwrap());
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// HashMap<String, String> (StringMap) 测试
// ============================================================================

fn make_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn test_value_stringmap_creation() {
    let m = make_map(&[("host", "localhost"), ("port", "8080")]);
    let v = Value::StringMap(m);
    assert_eq!(v.data_type(), DataType::StringMap);
    assert!(!v.is_empty());
}

#[test]
fn test_value_stringmap_get() {
    let m = make_map(&[("key", "value")]);
    let v = Value::StringMap(m.clone());
    assert_eq!(v.get_string_map().unwrap(), m);
}

#[test]
fn test_value_stringmap_set() {
    let mut v = Value::Empty(DataType::StringMap);
    let m = make_map(&[("a", "1"), ("b", "2")]);
    v.set_string_map(m.clone()).unwrap();
    assert_eq!(v.get_string_map().unwrap(), m);
}

#[test]
fn test_value_stringmap_empty_map() {
    let v = Value::StringMap(HashMap::new());
    assert_eq!(v.data_type(), DataType::StringMap);
    assert_eq!(v.get_string_map().unwrap().len(), 0);
}

#[test]
fn test_value_stringmap_type_mismatch() {
    let v = Value::StringMap(HashMap::new());
    assert!(matches!(
        v.get_string(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_stringmap_empty_error() {
    let v = Value::Empty(DataType::StringMap);
    assert!(matches!(v.get_string_map(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_stringmap_generic_new() {
    let m = make_map(&[("x-header", "value")]);
    let v = Value::new(m.clone());
    assert_eq!(v.data_type(), DataType::StringMap);
    let got: HashMap<String, String> = v.get().unwrap();
    assert_eq!(got, m);
}

#[test]
fn test_value_stringmap_generic_set() {
    let m1 = make_map(&[("old", "val")]);
    let m2 = make_map(&[("new", "val2")]);
    let mut v = Value::StringMap(m1);
    v.set(m2.clone()).unwrap();
    assert_eq!(v.get_string_map().unwrap(), m2);
}

#[test]
fn test_value_stringmap_as_string_is_json() {
    let m = make_map(&[("k", "v")]);
    let v = Value::StringMap(m.clone());
    let s = v.to::<String>().unwrap();
    let parsed: HashMap<String, String> = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, m);
}

#[test]
fn test_value_stringmap_clear() {
    let mut v = Value::StringMap(make_map(&[("a", "b")]));
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::StringMap);
}

#[test]
fn test_value_stringmap_serde_roundtrip() {
    let original = Value::StringMap(make_map(&[
        ("Content-Type", "application/json"),
        ("Accept", "*/*"),
    ]));
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// Json (serde_json::Value) 测试
// ============================================================================

#[test]
fn test_value_json_creation() {
    let j = serde_json::json!({"key": "value", "num": 42});
    let v = Value::Json(j);
    assert_eq!(v.data_type(), DataType::Json);
    assert!(!v.is_empty());
}

#[test]
fn test_value_json_get() {
    let j = serde_json::json!([1, 2, 3]);
    let v = Value::Json(j.clone());
    assert_eq!(v.get_json().unwrap(), j);
}

#[test]
fn test_value_json_set() {
    let mut v = Value::Empty(DataType::Json);
    let j = serde_json::json!(true);
    v.set_json(j.clone()).unwrap();
    assert_eq!(v.get_json().unwrap(), j);
}

#[test]
fn test_value_json_type_mismatch() {
    let v = Value::Json(serde_json::json!(null));
    assert!(matches!(
        v.get_string(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_value_json_empty_error() {
    let v = Value::Empty(DataType::Json);
    assert!(matches!(v.get_json(), Err(ValueError::NoValue)));
}

#[test]
fn test_value_json_generic_new() {
    let j = serde_json::json!({"hello": "world"});
    let v = Value::new(j.clone());
    assert_eq!(v.data_type(), DataType::Json);
    let got: serde_json::Value = v.get().unwrap();
    assert_eq!(got, j);
}

#[test]
fn test_value_json_generic_set() {
    let j1 = serde_json::json!(1);
    let j2 = serde_json::json!(2);
    let mut v = Value::Json(j1);
    v.set(j2.clone()).unwrap();
    assert_eq!(v.get_json().unwrap(), j2);
}

#[test]
fn test_value_json_from_json_value() {
    let j = serde_json::json!({"a": 1});
    let v = Value::from_json_value(j.clone());
    assert_eq!(v.data_type(), DataType::Json);
    assert_eq!(v.get_json().unwrap(), j);
}

#[test]
fn test_value_json_as_string() {
    let j = serde_json::json!({"x": 1});
    let v = Value::Json(j.clone());
    let s = v.to::<String>().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert_eq!(parsed, j);
}

#[test]
fn test_value_json_as_json_from_json() {
    let j = serde_json::json!({"x": 1});
    let v = Value::Json(j.clone());
    assert_eq!(v.to::<serde_json::Value>().unwrap(), j);
}

#[test]
fn test_value_json_as_json_from_string() {
    let j = serde_json::json!({"x": 1, "tags": ["a", "b"]});
    let v = Value::String(r#"{"x":1,"tags":["a","b"]}"#.to_string());
    assert_eq!(v.to::<serde_json::Value>().unwrap(), j);
}

#[test]
fn test_value_json_as_json_from_stringmap() {
    let map = make_map(&[("host", "localhost"), ("port", "8080")]);
    let v = Value::StringMap(map.clone());
    let json = v.to::<serde_json::Value>().unwrap();
    assert_eq!(json, serde_json::to_value(map).unwrap());
}

#[test]
fn test_value_json_as_json_invalid_string() {
    let v = Value::String("{invalid json}".to_string());
    assert!(matches!(
        v.to::<serde_json::Value>(),
        Err(ValueError::JsonDeserializationError(_))
    ));
}

#[test]
fn test_value_to_duration() {
    let v = Value::String("42ns".to_string());
    let got: Duration = v.to().unwrap();
    assert_eq!(got, Duration::from_nanos(42));
}

#[test]
fn test_value_to_url() {
    let v = Value::String("https://example.com/path".to_string());
    let got: Url = v.to().unwrap();
    assert_eq!(got, Url::parse("https://example.com/path").unwrap());
}

#[test]
fn test_value_to_json() {
    let v = Value::String(r#"{"name":"qubit"}"#.to_string());
    let got: serde_json::Value = v.to().unwrap();
    assert_eq!(got, serde_json::json!({"name": "qubit"}));
}

#[test]
fn test_value_to_string() {
    let v = Value::Duration(Duration::from_nanos(7));
    let got: String = v.to().unwrap();
    assert_eq!(got, "7ns");
}

#[test]
fn test_value_json_clear() {
    let mut v = Value::Json(serde_json::json!(42));
    v.clear();
    assert!(v.is_empty());
    assert_eq!(v.data_type(), DataType::Json);
}

#[test]
fn test_value_json_serde_roundtrip() {
    let original = Value::Json(serde_json::json!({"nested": {"arr": [1, 2, 3]}}));
    let json = serde_json::to_string(&original).unwrap();
    let restored: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

// ============================================================================
// from_serializable / deserialize_json 测试
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
    enabled: bool,
}

#[test]
fn test_value_from_serializable() {
    let cfg = Config {
        host: "localhost".to_string(),
        port: 8080,
        enabled: true,
    };
    let v = Value::from_serializable(&cfg).unwrap();
    assert_eq!(v.data_type(), DataType::Json);
}

#[test]
fn test_value_deserialize_json_roundtrip() {
    let cfg = Config {
        host: "example.com".to_string(),
        port: 443,
        enabled: false,
    };
    let v = Value::from_serializable(&cfg).unwrap();
    let restored: Config = v.deserialize_json().unwrap();
    assert_eq!(cfg, restored);
}

#[test]
fn test_value_deserialize_json_on_non_json_returns_error() {
    let v = Value::Int32(42);
    let result = v.deserialize_json::<Config>();
    assert!(result.is_err());
}

#[test]
fn test_value_deserialize_json_on_empty_returns_error() {
    let v = Value::Empty(DataType::Json);
    let result = v.deserialize_json::<Config>();
    assert!(matches!(result, Err(ValueError::NoValue)));
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Status {
    Active,
    Inactive,
    Pending(String),
}

#[test]
fn test_value_custom_enum_via_json() {
    let s = Status::Pending("review".to_string());
    let v = Value::from_serializable(&s).unwrap();
    let restored: Status = v.deserialize_json().unwrap();
    assert_eq!(s, restored);
}

#[test]
fn test_value_custom_enum_unit_variant_via_json() {
    let s = Status::Active;
    let v = Value::from_serializable(&s).unwrap();
    let restored: Status = v.deserialize_json().unwrap();
    assert_eq!(s, restored);
}

// ============================================================================
// DataType 枚举验证
// ============================================================================

#[test]
fn test_datatype_intsize_display() {
    assert_eq!(DataType::IntSize.as_str(), "intsize");
    assert_eq!(DataType::IntSize.to_string(), "intsize");
}

#[test]
fn test_datatype_uintsize_display() {
    assert_eq!(DataType::UIntSize.as_str(), "uintsize");
}

#[test]
fn test_datatype_duration_display() {
    assert_eq!(DataType::Duration.as_str(), "duration");
}

#[test]
fn test_datatype_url_display() {
    assert_eq!(DataType::Url.as_str(), "url");
}

#[test]
fn test_datatype_stringmap_display() {
    assert_eq!(DataType::StringMap.as_str(), "stringmap");
}

#[test]
fn test_datatype_json_display() {
    assert_eq!(DataType::Json.as_str(), "json");
}

#[test]
fn test_datatype_serde_roundtrip() {
    for dt in [
        DataType::IntSize,
        DataType::UIntSize,
        DataType::Duration,
        DataType::Url,
        DataType::StringMap,
        DataType::Json,
    ] {
        let json = serde_json::to_string(&dt).unwrap();
        let restored: DataType = serde_json::from_str(&json).unwrap();
        assert_eq!(dt, restored);
    }
}
