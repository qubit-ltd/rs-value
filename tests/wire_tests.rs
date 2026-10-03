// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests the public wire facade.

use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWireV1;

/// Checks every feature enabled V1 payload adapter through the public wire
/// entry point, including the scalar and collection representations.
#[cfg(feature = "all")]
#[test]
fn test_wire_payload_adapters_cover_every_value_type() {
    use std::collections::HashMap;
    use std::time::Duration;

    use bigdecimal::BigDecimal;
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use chrono::Utc;
    use num_bigint::BigInt;
    use qubit_value::MultiValues;
    use qubit_value::NamedMultiValues;
    use qubit_value::NamedValue;
    use qubit_value::ValueWirePayloadV1;
    use qubit_value::ValueContainer;

    let date = NaiveDate::from_ymd_opt(2025, 1, 2).expect("valid date");
    let time = NaiveTime::from_hms_opt(3, 4, 5).expect("valid time");
    let datetime = NaiveDateTime::new(date, time);
    let instant = DateTime::<Utc>::from_timestamp(1_735_776_000, 0).expect("valid instant");
    let map = HashMap::from([(String::from("key"), String::from("value"))]);
    let values = vec![
        Value::Bool(true),
        Value::Char('x'),
        Value::Int8(-8),
        Value::Int16(-16),
        Value::Int32(-32),
        Value::Int64(-64),
        Value::Int128(-128),
        Value::UInt8(8),
        Value::UInt16(16),
        Value::UInt32(32),
        Value::UInt64(64),
        Value::UInt128(128),
        Value::Float32(1.25),
        Value::Float64(2.5),
        Value::BigInteger(BigInt::from(123)),
        Value::BigDecimal(BigDecimal::from(123)),
        Value::String(String::from("text")),
        Value::Date(date),
        Value::Time(time),
        Value::DateTime(datetime),
        Value::Instant(instant),
        Value::Duration(Duration::from_millis(1234)),
        Value::Url(url::Url::parse("https://example.com/path").expect("valid URL")),
        Value::StringMap(map),
        Value::Json(serde_json::json!({"b": 2, "a": 1})),
    ];

    for value in values {
        let payload = ValueWirePayloadV1::try_from(value.clone()).expect("supported value should encode");
        let bytes = payload.to_json_vec().expect("supported value should serialize");
        let decoded = ValueWirePayloadV1::decode_json_slice(&bytes).expect("supported value should decode");
        assert_eq!(decoded, payload);

        let named = NamedValue::new("field", value.clone());
        let named_bytes = serde_json::to_vec(&named).expect("named value should serialize");
        let decoded_named = serde_json::from_slice::<NamedValue>(&named_bytes).expect("named value should decode");
        assert_eq!(decoded_named, named);

    }

    macro_rules! assert_collection_round_trip {
        ($values:expr) => {{
            let values = $values;
            let payload = ValueWirePayloadV1::try_from(ValueContainer::Collection(values.clone()))
                .expect("supported collection should encode");
            let bytes = payload.to_json_vec().expect("supported collection should serialize");
            assert_eq!(ValueWirePayloadV1::decode_json_slice(&bytes).expect("collection should decode"), payload);

            let named = NamedMultiValues::new("field", values);
            let bytes = serde_json::to_vec(&named).expect("named collection should serialize");
            assert_eq!(
                serde_json::from_slice::<NamedMultiValues>(&bytes).expect("named collection should decode"),
                named
            );
        }};
    }

    assert_collection_round_trip!(MultiValues::Bool(vec![true, false]));
    assert_collection_round_trip!(MultiValues::Char(vec!['a', 'b']));
    assert_collection_round_trip!(MultiValues::Int8(vec![-8, 8]));
    assert_collection_round_trip!(MultiValues::Int16(vec![-16, 16]));
    assert_collection_round_trip!(MultiValues::Int32(vec![-32, 32]));
    assert_collection_round_trip!(MultiValues::Int64(vec![-64, 64]));
    assert_collection_round_trip!(MultiValues::Int128(vec![-128, 128]));
    assert_collection_round_trip!(MultiValues::UInt8(vec![8, 9]));
    assert_collection_round_trip!(MultiValues::UInt16(vec![16, 17]));
    assert_collection_round_trip!(MultiValues::UInt32(vec![32, 33]));
    assert_collection_round_trip!(MultiValues::UInt64(vec![64, 65]));
    assert_collection_round_trip!(MultiValues::UInt128(vec![128, 129]));
    assert_collection_round_trip!(MultiValues::Float32(vec![1.25, 2.5]));
    assert_collection_round_trip!(MultiValues::Float64(vec![1.25, 2.5]));
    assert_collection_round_trip!(MultiValues::BigInteger(vec![BigInt::from(123), BigInt::from(456)]));
    assert_collection_round_trip!(MultiValues::BigDecimal(vec![BigDecimal::from(123), BigDecimal::from(456)]));
    assert_collection_round_trip!(MultiValues::String(vec![String::from("a"), String::from("b")]));
    assert_collection_round_trip!(MultiValues::Date(vec![date]));
    assert_collection_round_trip!(MultiValues::Time(vec![time]));
    assert_collection_round_trip!(MultiValues::DateTime(vec![datetime]));
    assert_collection_round_trip!(MultiValues::Instant(vec![instant]));
    assert_collection_round_trip!(MultiValues::Duration(vec![Duration::from_secs(1)]));
    assert_collection_round_trip!(MultiValues::Url(vec![url::Url::parse("https://example.com").expect("valid URL")]));
    assert_collection_round_trip!(MultiValues::StringMap(vec![HashMap::from([(String::from("key"), String::from("value"))])]));
    assert_collection_round_trip!(MultiValues::Json(vec![serde_json::json!({"a": 1})]));
}

/// Verifies the public wire facade preserves scalar values.
#[test]
fn test_wire_round_trips_scalar_value() {
    let wire = ValueWireV1::try_from(Value::Int32(7)).expect("construct V1 wire");
    let decoded = crate::decode_value_wire_value(serde_json::to_value(wire).unwrap()).unwrap();
    assert_eq!(
        decoded,
        ValueWireV1::try_from(Value::Int32(7)).expect("construct V1 wire")
    );
}

/// Verifies nested protocols receive an unversioned but typed V1 payload.
#[test]
fn test_wire_payload_preserves_shape_without_version() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::from(vec![7_i32])).expect("construct V1 payload");

    assert_eq!(
        serde_json::to_value(payload).expect("serialize V1 payload"),
        serde_json::json!({"collection": {"int32": [7]}}),
    );
}
