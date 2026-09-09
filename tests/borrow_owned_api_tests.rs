// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueError;
use qubit_value::ValueRef;

#[test]
fn test_collection_view_borrows_indexed_text_and_preserves_shape() {
    let values = MultiValues::String(vec![String::from("123")]);
    let expected = values.get_slice::<String>().unwrap()[0].as_ptr();
    let view = values.view();
    assert_eq!(view.data_type(), DataType::String);
    assert_eq!(view.len(), 1);
    assert!(!view.is_empty());
    let Some(ValueRef::String(text)) = view.get(0) else {
        panic!("string view")
    };
    assert_eq!(text.as_ptr(), expected);
    assert!(view.get(1).is_none());
    assert!(view.get(usize::MAX).is_none());
    for values in [MultiValues::Unset(DataType::Int32), MultiValues::Int32(Vec::new())] {
        let view = values.view();
        assert_eq!(view.data_type(), DataType::Int32);
        assert_eq!(view.len(), 0);
        assert!(view.is_empty());
        assert!(view.get(0).is_none());
    }
}

#[cfg(feature = "converter")]
#[test]
fn test_borrowed_view_converts_without_materializing_value() {
    use qubit_datatype::DataConverter;
    let value = Value::from("123");
    assert_eq!(DataConverter::from(value.view()).to::<i32>().unwrap(), 123);
    let values = MultiValues::UInt64(vec![42]);
    assert_eq!(
        DataConverter::from(values.view().get(0).unwrap()).to::<i32>().unwrap(),
        42
    );
    let duration = Value::Duration(std::time::Duration::from_secs(2));
    assert_eq!(
        DataConverter::from(duration.view())
            .to::<std::time::Duration>()
            .unwrap(),
        std::time::Duration::from_secs(2)
    );
    let unset = Value::Unset(DataType::String);
    assert!(DataConverter::from(unset.view()).to::<i32>().unwrap_err().is_missing());
}

#[cfg(feature = "converter")]
#[test]
fn test_rich_collection_views_keep_original_payloads() {
    use qubit_datatype::DataConverter;
    use qubit_value::ValueRef;
    let map = std::collections::HashMap::from([("key".to_owned(), "value".to_owned())]);
    let values = MultiValues::StringMap(vec![map]);
    let item = values.view().get(0).unwrap();
    let ValueRef::StringMap(map) = item else {
        panic!("map view")
    };
    assert!(std::ptr::eq(
        map,
        &values.get_slice::<std::collections::HashMap<String, String>>().unwrap()[0]
    ));
    assert_eq!(
        DataConverter::from(item)
            .to::<std::collections::HashMap<String, String>>()
            .unwrap(),
        *map
    );
    #[cfg(feature = "json")]
    {
        let values = MultiValues::Json(vec![serde_json::json!({"items": [1, 2]})]);
        let item = values.view().get(0).unwrap();
        let ValueRef::Json(json) = item else {
            panic!("JSON view")
        };
        assert!(std::ptr::eq(json, &values.get_slice::<serde_json::Value>().unwrap()[0]));
        assert_eq!(DataConverter::from(item).to::<serde_json::Value>().unwrap(), *json);
    }
    #[cfg(feature = "url")]
    {
        let values = MultiValues::Url(vec![url::Url::parse("https://example.com/").unwrap()]);
        let item = values.view().get(0).unwrap();
        let ValueRef::Url(url) = item else { panic!("URL view") };
        assert!(std::ptr::eq(url, &values.get_slice::<url::Url>().unwrap()[0]));
        assert_eq!(DataConverter::from(item).to::<url::Url>().unwrap(), *url);
        assert_eq!(
            DataConverter::from(Value::Url(url.clone()).view())
                .to::<url::Url>()
                .unwrap(),
            *url
        );
    }
    #[cfg(feature = "chrono")]
    {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap();
        let values = MultiValues::Date(vec![date]);
        let item = values.view().get(0).unwrap();
        let ValueRef::Date(borrowed) = item else {
            panic!("date view")
        };
        assert!(std::ptr::eq(
            borrowed,
            &values.get_slice::<chrono::NaiveDate>().unwrap()[0]
        ));
        assert_eq!(DataConverter::from(item).to::<chrono::NaiveDate>().unwrap(), date);
    }
    #[cfg(feature = "big-integer")]
    {
        let values = MultiValues::BigInteger(vec![num_bigint::BigInt::from(123)]);
        let item = values.view().get(0).unwrap();
        let ValueRef::BigInteger(number) = item else {
            panic!("big integer view")
        };
        assert!(std::ptr::eq(
            number,
            &values.get_slice::<num_bigint::BigInt>().unwrap()[0]
        ));
        assert_eq!(DataConverter::from(item).to::<i32>().unwrap(), 123);
    }
    #[cfg(feature = "big-decimal")]
    {
        let values = MultiValues::BigDecimal(vec![bigdecimal::BigDecimal::from(123)]);
        let item = values.view().get(0).unwrap();
        let ValueRef::BigDecimal(number) = item else {
            panic!("decimal view")
        };
        assert!(std::ptr::eq(
            number,
            &values.get_slice::<bigdecimal::BigDecimal>().unwrap()[0]
        ));
        assert_eq!(DataConverter::from(item).to::<i32>().unwrap(), 123);
    }
}

#[test]
fn test_borrowed_scalar_and_collection_reads_keep_storage() {
    let text = String::from("hello");
    let ptr = text.as_ptr();
    let value = Value::String(text);
    let text: &str = value.get_ref().unwrap();
    assert_eq!(text, "hello");
    assert_eq!(text.as_ptr(), ptr);

    let values = MultiValues::String(vec![String::from("a"), String::from("b")]);
    let vec_ptr = values.get_slice::<String>().unwrap().as_ptr();
    let first: &String = values.get_first_ref().unwrap();
    let all: &[String] = values.get_slice().unwrap();
    assert_eq!(first, "a");
    assert_eq!(all.len(), 2);
    assert_eq!(all.as_ptr(), vec_ptr);

    let container = ValueContainer::from(vec![1_i32, 2, 3]);
    assert_eq!(container.get_slice::<i32>().unwrap(), &[1, 2, 3]);
}

#[test]
fn test_borrowed_reads_preserve_missing_and_mismatch_errors() {
    let unset = Value::new_unset(DataType::String);
    assert!(matches!(unset.get_ref::<String>(), Err(ValueError::Missing(_))));

    let empty = MultiValues::String(Vec::new());
    assert!(matches!(empty.get_first_ref::<String>(), Err(ValueError::Missing(_))));
    let empty_slice: &[String] = empty.get_slice().unwrap();
    assert!(empty_slice.is_empty());

    let wrong = Value::Int32(1);
    assert!(matches!(
        wrong.get_ref::<String>(),
        Err(ValueError::TypeMismatch { .. })
    ));
}

#[test]
fn test_consuming_reads_move_storage() {
    let text = String::from("owned");
    let ptr = text.as_ptr();
    let value = Value::String(text);
    let text = String::try_from(value).unwrap();
    assert_eq!(text, "owned");
    assert_eq!(text.as_ptr(), ptr);

    let source = vec![1, 2];
    let ptr = source.as_ptr();
    let values = MultiValues::Int32(source);
    let items = Vec::<i32>::try_from(values).unwrap();
    assert_eq!(items, vec![1, 2]);
    assert_eq!(items.as_ptr(), ptr);
}
