/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! # MultiValues v0.3.0 新增类型单元测试
//!
//! 覆盖 v0.3.0 新增的以下类型在 MultiValues 中的支持：
//! - `isize` / `usize`
//! - `std::time::Duration`
//! - `url::Url`
//! - `HashMap<String, String>`
//! - `serde_json::Value` (Json escape hatch)
//!

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    Value,
    ValueError,
};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;

fn make_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ============================================================================
// isize MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_intsize_creation() {
    let mv = MultiValues::IntSize(vec![1isize, -2, 3]);
    assert_eq!(mv.data_type(), DataType::IntSize);
    assert_eq!(mv.count(), 3);
    assert!(!mv.is_empty());
}

#[test]
fn test_multi_values_intsize_get_all() {
    let mv = MultiValues::IntSize(vec![10isize, 20, 30]);
    assert_eq!(mv.get_intsizes().unwrap(), &[10isize, 20, 30]);
}

#[test]
fn test_multi_values_intsize_get_first() {
    let mv = MultiValues::IntSize(vec![42isize, 100]);
    assert_eq!(mv.get_first_intsize().unwrap(), 42isize);
}

#[test]
fn test_multi_values_intsize_get_first_empty_error() {
    let mv = MultiValues::Empty(DataType::IntSize);
    assert!(matches!(mv.get_first_intsize(), Err(ValueError::NoValue)));
}

#[test]
fn test_multi_values_intsize_set_single() {
    let mut mv = MultiValues::Empty(DataType::IntSize);
    mv.set_intsize(-5isize).unwrap();
    assert_eq!(mv.get_intsizes().unwrap(), &[-5isize]);
}

#[test]
fn test_multi_values_intsize_set_all() {
    let mut mv = MultiValues::IntSize(vec![1isize]);
    mv.set_intsizes(vec![10isize, 20, 30]).unwrap();
    assert_eq!(mv.get_intsizes().unwrap(), &[10isize, 20, 30]);
}

#[test]
fn test_multi_values_intsize_add_single() {
    let mut mv = MultiValues::IntSize(vec![1isize]);
    mv.add_intsize(2isize).unwrap();
    assert_eq!(mv.get_intsizes().unwrap(), &[1isize, 2]);
}

#[test]
fn test_multi_values_intsize_add_multiple() {
    let mut mv = MultiValues::IntSize(vec![1isize]);
    mv.add_intsizes(vec![2isize, 3]).unwrap();
    assert_eq!(mv.get_intsizes().unwrap(), &[1isize, 2, 3]);
}

#[test]
fn test_multi_values_intsize_type_mismatch() {
    let mv = MultiValues::IntSize(vec![1isize]);
    assert!(matches!(
        mv.get_uintsizes(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_multi_values_intsize_generic_new() {
    let mv = MultiValues::new(vec![1isize, 2, 3]);
    assert_eq!(mv.data_type(), DataType::IntSize);
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_values_intsize_generic_get() {
    let mv = MultiValues::IntSize(vec![7isize, 8]);
    let got: Vec<isize> = mv.get().unwrap();
    assert_eq!(got, vec![7isize, 8]);
}

#[test]
fn test_multi_values_intsize_generic_get_first() {
    let mv = MultiValues::IntSize(vec![99isize]);
    let first: isize = mv.get_first().unwrap();
    assert_eq!(first, 99isize);
}

#[test]
fn test_multi_values_intsize_serde_roundtrip() {
    let original = MultiValues::IntSize(vec![isize::MIN, 0, isize::MAX]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_intsize_merge() {
    let mut a = MultiValues::IntSize(vec![1isize, 2]);
    let b = MultiValues::IntSize(vec![3isize, 4]);
    a.merge(&b).unwrap();
    assert_eq!(a.get_intsizes().unwrap(), &[1isize, 2, 3, 4]);
}

#[test]
fn test_multi_values_intsize_clear() {
    let mut mv = MultiValues::IntSize(vec![1isize, 2, 3]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::IntSize);
}

#[test]
fn test_multi_values_intsize_from_value() {
    let v = Value::IntSize(42isize);
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::IntSize);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_intsize().unwrap(), 42isize);
}

// ============================================================================
// usize MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_uintsize_creation() {
    let mv = MultiValues::UIntSize(vec![100usize, 200, 300]);
    assert_eq!(mv.data_type(), DataType::UIntSize);
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_values_uintsize_get_all() {
    let mv = MultiValues::UIntSize(vec![1usize, 2, 3]);
    assert_eq!(mv.get_uintsizes().unwrap(), &[1usize, 2, 3]);
}

#[test]
fn test_multi_values_uintsize_get_first() {
    let mv = MultiValues::UIntSize(vec![8080usize]);
    assert_eq!(mv.get_first_uintsize().unwrap(), 8080usize);
}

#[test]
fn test_multi_values_uintsize_add_single() {
    let mut mv = MultiValues::UIntSize(vec![1usize]);
    mv.add_uintsize(2usize).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_uintsize_merge() {
    let mut mv1 = MultiValues::UIntSize(vec![10usize, 20]);
    let mv2 = MultiValues::UIntSize(vec![30usize, 40]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.get_uintsizes().unwrap(), &[10usize, 20, 30, 40]);
}

#[test]
fn test_multi_values_uintsize_clear() {
    let mut mv = MultiValues::UIntSize(vec![1usize, 2, 3]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::UIntSize);
}

#[test]
fn test_multi_values_uintsize_serde_roundtrip() {
    let original = MultiValues::UIntSize(vec![0usize, 1, usize::MAX]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_uintsize_from_value() {
    let v = Value::UIntSize(512usize);
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::UIntSize);
    assert_eq!(mv.get_first_uintsize().unwrap(), 512usize);
}

// ============================================================================
// Duration MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_duration_creation() {
    let mv = MultiValues::Duration(vec![Duration::from_secs(1), Duration::from_secs(2)]);
    assert_eq!(mv.data_type(), DataType::Duration);
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_duration_get_all() {
    let d1 = Duration::from_millis(100);
    let d2 = Duration::from_millis(200);
    let mv = MultiValues::Duration(vec![d1, d2]);
    let got = mv.get_durations().unwrap();
    assert_eq!(got, &[d1, d2]);
}

#[test]
fn test_multi_values_duration_get_first() {
    let d = Duration::from_secs(30);
    let mv = MultiValues::Duration(vec![d, Duration::from_secs(60)]);
    assert_eq!(mv.get_first_duration().unwrap(), d);
}

#[test]
fn test_multi_values_duration_add_single() {
    let mut mv = MultiValues::Duration(vec![Duration::from_secs(1)]);
    mv.add_duration(Duration::from_secs(2)).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_duration_set_all() {
    let mut mv = MultiValues::Empty(DataType::Duration);
    mv.set_durations(vec![Duration::from_secs(5), Duration::from_secs(10)])
        .unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_duration_serde_roundtrip() {
    let original = MultiValues::Duration(vec![
        Duration::ZERO,
        Duration::from_nanos(1),
        Duration::from_secs(3600),
    ]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_duration_merge() {
    let mut a = MultiValues::Duration(vec![Duration::from_secs(1)]);
    let b = MultiValues::Duration(vec![Duration::from_secs(2)]);
    a.merge(&b).unwrap();
    assert_eq!(a.count(), 2);
}

#[test]
fn test_multi_values_duration_clear() {
    let mut mv = MultiValues::Duration(vec![Duration::from_secs(10), Duration::from_secs(20)]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Duration);
}

#[test]
fn test_multi_values_duration_from_value() {
    let d = Duration::from_secs(60);
    let v = Value::Duration(d);
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::Duration);
    assert_eq!(mv.get_first_duration().unwrap(), d);
}

#[test]
fn test_multi_values_duration_generic() {
    let mv = MultiValues::new(vec![Duration::from_secs(1), Duration::from_secs(2)]);
    assert_eq!(mv.data_type(), DataType::Duration);
    let got: Vec<Duration> = mv.get().unwrap();
    assert_eq!(got.len(), 2);
    let first: Duration = mv.get_first().unwrap();
    assert_eq!(first, Duration::from_secs(1));
}

// ============================================================================
// Url MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_url_creation() {
    let mv = MultiValues::Url(vec![
        Url::parse("https://example.com").unwrap(),
        Url::parse("http://localhost:8080").unwrap(),
    ]);
    assert_eq!(mv.data_type(), DataType::Url);
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_url_get_all() {
    let u1 = Url::parse("https://a.com").unwrap();
    let u2 = Url::parse("https://b.com").unwrap();
    let mv = MultiValues::Url(vec![u1.clone(), u2.clone()]);
    let got = mv.get_urls().unwrap();
    assert_eq!(got, &[u1, u2]);
}

#[test]
fn test_multi_values_url_get_first() {
    let u = Url::parse("https://first.example.com").unwrap();
    let mv = MultiValues::Url(vec![
        u.clone(),
        Url::parse("https://second.example.com").unwrap(),
    ]);
    assert_eq!(mv.get_first_url().unwrap(), u);
}

#[test]
fn test_multi_values_url_add_single() {
    let mut mv = MultiValues::Url(vec![Url::parse("https://a.com").unwrap()]);
    mv.add_url(Url::parse("https://b.com").unwrap()).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_url_set_all() {
    let mut mv = MultiValues::Empty(DataType::Url);
    mv.set_urls(vec![
        Url::parse("https://x.com").unwrap(),
        Url::parse("https://y.com").unwrap(),
    ])
    .unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_url_serde_roundtrip() {
    let original = MultiValues::Url(vec![
        Url::parse("https://example.com/path?q=1#frag").unwrap(),
        Url::parse("ftp://files.example.org").unwrap(),
    ]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_url_clear() {
    let mut mv = MultiValues::Url(vec![
        Url::parse("https://a.com").unwrap(),
        Url::parse("https://b.com").unwrap(),
    ]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Url);
}

#[test]
fn test_multi_values_url_merge() {
    let mut mv1 = MultiValues::Url(vec![Url::parse("https://a.com").unwrap()]);
    let mv2 = MultiValues::Url(vec![Url::parse("https://b.com").unwrap()]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 2);
    assert_eq!(mv1.get_urls().unwrap().len(), 2);
}

#[test]
fn test_multi_values_url_from_value() {
    let u = Url::parse("https://example.com").unwrap();
    let v = Value::Url(u.clone());
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::Url);
    assert_eq!(mv.get_first_url().unwrap(), u);
}

#[test]
fn test_multi_values_url_generic() {
    let u1 = Url::parse("https://a.com").unwrap();
    let u2 = Url::parse("https://b.com").unwrap();
    let mv = MultiValues::new(vec![u1.clone(), u2.clone()]);
    assert_eq!(mv.data_type(), DataType::Url);
    let got: Vec<Url> = mv.get().unwrap();
    assert_eq!(got, vec![u1.clone(), u2]);
    let first: Url = mv.get_first().unwrap();
    assert_eq!(first, u1);
}

// ============================================================================
// StringMap MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_stringmap_creation() {
    let m1 = make_map(&[("k1", "v1")]);
    let m2 = make_map(&[("k2", "v2")]);
    let mv = MultiValues::StringMap(vec![m1, m2]);
    assert_eq!(mv.data_type(), DataType::StringMap);
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_stringmap_get_all() {
    let m = make_map(&[("host", "localhost")]);
    let mv = MultiValues::StringMap(vec![m.clone()]);
    let got = mv.get_string_maps().unwrap();
    assert_eq!(got, &[m]);
}

#[test]
fn test_multi_values_stringmap_get_first() {
    let m1 = make_map(&[("a", "1")]);
    let m2 = make_map(&[("b", "2")]);
    let mv = MultiValues::StringMap(vec![m1.clone(), m2]);
    assert_eq!(mv.get_first_string_map().unwrap(), m1);
}

#[test]
fn test_multi_values_stringmap_add_single() {
    let m1 = make_map(&[("x", "1")]);
    let m2 = make_map(&[("y", "2")]);
    let mut mv = MultiValues::StringMap(vec![m1]);
    mv.add_string_map(m2).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_stringmap_set_all() {
    let mut mv = MultiValues::Empty(DataType::StringMap);
    mv.set_string_maps(vec![make_map(&[("a", "1")]), make_map(&[("b", "2")])])
        .unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_stringmap_empty_map() {
    let mv = MultiValues::StringMap(vec![HashMap::new()]);
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_string_map().unwrap().len(), 0);
}

#[test]
fn test_multi_values_stringmap_serde_roundtrip() {
    let original = MultiValues::StringMap(vec![
        make_map(&[("Content-Type", "application/json")]),
        make_map(&[("X-Custom", "value"), ("Accept", "*/*")]),
    ]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_stringmap_clear() {
    let mut mv = MultiValues::StringMap(vec![make_map(&[("a", "1")]), make_map(&[("b", "2")])]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::StringMap);
}

#[test]
fn test_multi_values_stringmap_merge() {
    let mut mv1 = MultiValues::StringMap(vec![make_map(&[("a", "1")])]);
    let mv2 = MultiValues::StringMap(vec![make_map(&[("b", "2")])]);
    mv1.merge(&mv2).unwrap();
    assert_eq!(mv1.count(), 2);
}

#[test]
fn test_multi_values_stringmap_from_value() {
    let m = make_map(&[("key", "val")]);
    let v = Value::StringMap(m.clone());
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::StringMap);
    assert_eq!(mv.get_first_string_map().unwrap(), m);
}

#[test]
fn test_multi_values_stringmap_generic() {
    let m = make_map(&[("a", "b")]);
    let mv = MultiValues::new(vec![m.clone()]);
    assert_eq!(mv.data_type(), DataType::StringMap);
    let got: Vec<HashMap<String, String>> = mv.get().unwrap();
    assert_eq!(got, vec![m.clone()]);
    let first: HashMap<String, String> = mv.get_first().unwrap();
    assert_eq!(first, m);
}

// ============================================================================
// Json MultiValues 测试
// ============================================================================

#[test]
fn test_multi_values_json_creation() {
    let mv = MultiValues::Json(vec![
        serde_json::json!({"a": 1}),
        serde_json::json!([1, 2, 3]),
    ]);
    assert_eq!(mv.data_type(), DataType::Json);
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_json_get_all() {
    let j1 = serde_json::json!(true);
    let j2 = serde_json::json!(null);
    let mv = MultiValues::Json(vec![j1.clone(), j2.clone()]);
    let got = mv.get_jsons().unwrap();
    assert_eq!(got, &[j1, j2]);
}

#[test]
fn test_multi_values_json_get_first() {
    let j = serde_json::json!(42);
    let mv = MultiValues::Json(vec![j.clone(), serde_json::json!(99)]);
    assert_eq!(mv.get_first_json().unwrap(), j);
}

#[test]
fn test_multi_values_json_add_single() {
    let mut mv = MultiValues::Json(vec![serde_json::json!(1)]);
    mv.add_json(serde_json::json!(2)).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_json_set_all() {
    let mut mv = MultiValues::Empty(DataType::Json);
    mv.set_jsons(vec![
        serde_json::json!("hello"),
        serde_json::json!({"key": "val"}),
    ])
    .unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_json_serde_roundtrip() {
    let original = MultiValues::Json(vec![
        serde_json::json!({"nested": {"arr": [1, 2, 3]}}),
        serde_json::json!(null),
        serde_json::json!([true, false]),
    ]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: MultiValues = serde_json::from_str(&json).unwrap();
    assert_eq!(original, restored);
}

#[test]
fn test_multi_values_json_clear() {
    let mut mv = MultiValues::Json(vec![serde_json::json!({"a": 1}), serde_json::json!(2)]);
    mv.clear();
    assert!(mv.is_empty());
    assert_eq!(mv.data_type(), DataType::Json);
}

#[test]
fn test_multi_values_json_from_value() {
    let j = serde_json::json!({"x": 1});
    let v = Value::Json(j.clone());
    let mv = MultiValues::from(v);
    assert_eq!(mv.data_type(), DataType::Json);
    assert_eq!(mv.get_first_json().unwrap(), j);
}

#[test]
fn test_multi_values_json_generic() {
    let j1 = serde_json::json!(1);
    let j2 = serde_json::json!(2);
    let mv = MultiValues::new(vec![j1.clone(), j2.clone()]);
    assert_eq!(mv.data_type(), DataType::Json);
    let got: Vec<serde_json::Value> = mv.get().unwrap();
    assert_eq!(got, vec![j1.clone(), j2]);
    let first: serde_json::Value = mv.get_first().unwrap();
    assert_eq!(first, j1);
}

#[test]
fn test_multi_values_json_merge() {
    let mut a = MultiValues::Json(vec![serde_json::json!(1)]);
    let b = MultiValues::Json(vec![serde_json::json!(2), serde_json::json!(3)]);
    a.merge(&b).unwrap();
    assert_eq!(a.count(), 3);
}

// ============================================================================
// 跨类型错误验证
// ============================================================================

#[test]
fn test_multi_values_type_mismatch_across_new_types() {
    let mv_intsize = MultiValues::IntSize(vec![1isize]);
    assert!(matches!(
        mv_intsize.get_uintsizes(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        mv_intsize.get_durations(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        mv_intsize.get_urls(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        mv_intsize.get_string_maps(),
        Err(ValueError::TypeMismatch { .. })
    ));
    assert!(matches!(
        mv_intsize.get_jsons(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_multi_values_add_type_mismatch() {
    let mut mv = MultiValues::IntSize(vec![1isize]);
    assert!(matches!(
        mv.add_uintsize(1usize),
        Err(ValueError::TypeMismatch { .. })
    ));
}

// ============================================================================
// 通用 add/set 接口测试
// ============================================================================

#[test]
fn test_multi_values_intsize_generic_add() {
    let mut mv = MultiValues::IntSize(vec![1isize]);
    mv.add(2isize).unwrap();
    assert_eq!(mv.count(), 2);
}

#[test]
fn test_multi_values_uintsize_generic_set() {
    let mut mv = MultiValues::UIntSize(vec![1usize]);
    mv.set(vec![10usize, 20, 30]).unwrap();
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_values_duration_generic_add_vec() {
    let mut mv = MultiValues::Duration(vec![Duration::from_secs(1)]);
    mv.add(vec![Duration::from_secs(2), Duration::from_secs(3)])
        .unwrap();
    assert_eq!(mv.count(), 3);
}

#[test]
fn test_multi_values_url_generic_set_single() {
    let u = Url::parse("https://example.com").unwrap();
    let mut mv = MultiValues::Url(vec![]);
    mv.set(u.clone()).unwrap();
    assert_eq!(mv.count(), 1);
    assert_eq!(mv.get_first_url().unwrap(), u);
}
