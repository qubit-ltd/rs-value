// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests the public wire facade.

#[cfg(feature = "all")]
use std::collections::HashMap;
#[cfg(all(feature = "chrono", feature = "json"))]
use std::str::from_utf8;
use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "all")]
use chrono::DateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "all")]
use chrono::NaiveDateTime;
#[cfg(feature = "all")]
use chrono::NaiveTime;
#[cfg(feature = "chrono")]
use chrono::TimeZone;
#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "big-decimal")]
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueContainer;
#[cfg(feature = "all")]
use qubit_value::ValueWireDecodeError;
#[cfg(feature = "chrono")]
use qubit_value::ValueWireEncodeError;
#[cfg(feature = "chrono")]
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
#[cfg(feature = "json")]
use qubit_value::ValueWirePayloadV1Seed;
#[cfg(feature = "chrono")]
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use ron::Deserializer as RonDeserializer;
use ron::Error as RonError;
use ron::from_str as from_ron_str;
use ron::ser::PrettyConfig;
use ron::ser::to_string_pretty;
use ron::to_string as to_ron_string;
use serde::Deserialize;
use serde::Deserializer as SerdeDeserializer;
use serde::de::DeserializeSeed;
#[cfg(feature = "json")]
use serde::de::IntoDeserializer;
use serde::de::value::SeqDeserializer;
use serde::forward_to_deserialize_any;
#[cfg(feature = "all")]
use serde_json::from_slice;
use serde_json::json;
use serde_json::to_value;
#[cfg(feature = "chrono")]
use serde_json::to_vec;
#[cfg(feature = "all")]
use url::Url;

use crate::decode_value_wire_str;
use crate::decode_value_wire_value;

#[cfg(feature = "json")]
#[test]
fn test_named_and_versioned_wire_reject_wrong_top_level_json_type() {
    assert!(NamedValue::decode_json_slice(b"null").is_err());
    assert!(NamedMultiValues::decode_json_slice(b"null").is_err());
    assert!(ValueWireV1::decode_json_slice(b"null").is_err());
}

#[cfg(feature = "json")]
#[test]
fn test_named_collection_and_wire_writers_accept_sink() {
    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![80, 443]));
    let mut sink = std::io::sink();
    named
        .to_json_writer(&mut sink)
        .expect("named collection writes to a sink");

    let wire = ValueWireV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 wire");
    wire.to_json_writer(&mut sink).expect("V1 wire writes to a sink");
}

#[test]
fn test_ron_named_scalar_roundtrip() {
    let original = NamedValue::new("port", Value::Int32(42));
    let text = to_ron_string(&original).expect("serialize named scalar");
    let restored: NamedValue = from_ron_str(&text).expect("decode named scalar");
    assert_eq!(restored, original);
}

#[test]
fn test_ron_named_collection_roundtrip() {
    let original = NamedMultiValues::new("ports", MultiValues::Int32(vec![42]));
    let text = to_ron_string(&original).expect("serialize named collection");
    let restored: NamedMultiValues = from_ron_str(&text).expect("decode named collection");
    assert_eq!(restored, original);
}

/// Exercises positional records accepted from non-human-readable formats.
#[test]
fn test_non_human_readable_named_and_wire_records_accept_exact_sequences() {
    let named_scalar = serde_json::json!([
        "port",
        {"version": 1, "value": {"scalar": {"int32": 42}}}
    ]);
    let decoded = NamedValue::deserialize(NonHumanReadableSequence(
        named_scalar.as_array().expect("fixture is an array").clone(),
    ))
    .expect("named scalar sequence should decode");
    assert_eq!(decoded, NamedValue::new("port", Value::Int32(42)));

    let named_values = serde_json::json!([
        "ports",
        {"version": 1, "value": {"collection": {"int32": [42, 43]}}}
    ]);
    let decoded = NamedMultiValues::deserialize(NonHumanReadableSequence(
        named_values.as_array().expect("fixture is an array").clone(),
    ))
    .expect("named collection sequence should decode");
    assert_eq!(
        decoded,
        NamedMultiValues::new("ports", MultiValues::Int32(vec![42, 43]))
    );

    let wire = serde_json::json!([1, {"scalar": {"int32": 7}}]);
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableSequence(
            wire.as_array().expect("fixture is an array").clone(),
        ))
        .expect("wire envelope sequence should decode");
    assert_eq!(decoded.container(), &ValueContainer::from(7_i32));
}

/// Rejects short and overlong positional records for each owned visitor.
#[test]
fn test_non_human_readable_owned_record_sequences_require_exact_lengths() {
    for input in [
        serde_json::json!(["port"]),
        serde_json::json!(["port", {"version": 1, "value": {"scalar": {"int32": 1}}}, true]),
    ] {
        let fields = input.as_array().expect("fixture is an array").clone();
        assert!(NamedValue::deserialize(NonHumanReadableSequence(fields)).is_err());
    }
    for input in [
        serde_json::json!(["ports"]),
        serde_json::json!(["ports", {"version": 1, "value": {"collection": {"int32": [1]}}}, true]),
    ] {
        let fields = input.as_array().expect("fixture is an array").clone();
        assert!(NamedMultiValues::deserialize(NonHumanReadableSequence(fields)).is_err());
    }
    for input in [
        serde_json::json!([1]),
        serde_json::json!([1, {"scalar": {"int32": 7}}, true]),
    ] {
        let fields = input.as_array().expect("fixture is an array").clone();
        assert!(ValueWireV1Seed.deserialize(NonHumanReadableSequence(fields)).is_err());
    }
}

/// Exercises borrowed sequence access used by serde_json value references.
#[test]
fn test_non_human_readable_borrowed_named_and_wire_sequences() {
    let named_scalar = serde_json::json!([
        "port",
        {"version": 1, "value": {"scalar": {"int32": 42}}}
    ]);
    let decoded = NamedValue::deserialize(NonHumanReadableValue(&named_scalar))
        .expect("borrowed named scalar sequence should decode");
    assert_eq!(decoded, NamedValue::new("port", Value::Int32(42)));

    let named_values = serde_json::json!([
        "ports",
        {"version": 1, "value": {"collection": {"int32": [42]}}}
    ]);
    let decoded = NamedMultiValues::deserialize(NonHumanReadableValue(&named_values))
        .expect("borrowed named collection sequence should decode");
    assert_eq!(decoded, NamedMultiValues::new("ports", MultiValues::Int32(vec![42])));

    let wire = serde_json::json!([1, {"scalar": {"int32": 7}}]);
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableValue(&wire))
        .expect("borrowed wire envelope sequence should decode");
    assert_eq!(decoded.container(), &ValueContainer::from(7_i32));
}

/// Exercises owned wrapper records with serde_json's value deserializer and
/// the unversioned payload seed.
#[cfg(feature = "json")]
#[test]
fn test_owned_wire_records_decode_from_json_values() {
    let named_scalar: NamedValue = serde_json::from_value(serde_json::json!({
        "name": "port",
        "value": {"version": 1, "value": {"scalar": {"int32": 42}}}
    }))
    .expect("named scalar map should decode");
    assert_eq!(named_scalar, NamedValue::new("port", Value::Int32(42)));

    let named_collection: NamedMultiValues = serde_json::from_value(serde_json::json!({
        "name": "ports",
        "value": {"version": 1, "value": {"collection": {"int32": [42, 43]}}}
    }))
    .expect("named collection map should decode");
    assert_eq!(
        named_collection,
        NamedMultiValues::new("ports", MultiValues::Int32(vec![42, 43]))
    );

    let envelope = ValueWireV1Seed::new()
        .deserialize(serde_json::json!({"version": 1, "value": {"scalar": {"int32": 7}}}).into_deserializer())
        .expect("V1 envelope map should decode");
    assert_eq!(envelope.container(), &ValueContainer::from(7_i32));

    let scalar = ValueWirePayloadV1Seed::new()
        .deserialize(serde_json::json!({"scalar": {"int32": 7}}).into_deserializer())
        .expect("unversioned scalar payload should decode");
    assert_eq!(scalar.into_container(), ValueContainer::from(7_i32));

    let collection = ValueWirePayloadV1Seed::new()
        .deserialize(serde_json::json!({"collection": {"int32": [7, 11]}}).into_deserializer())
        .expect("unversioned collection payload should decode");
    assert_eq!(collection.into_container(), ValueContainer::from(vec![7_i32, 11]));
}

/// Presents a JSON array as a non-human-readable struct sequence.
struct NonHumanReadableSequence(Vec<serde_json::Value>);

impl<'de> SerdeDeserializer<'de> for NonHumanReadableSequence {
    type Error = serde_json::Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::<_, Self::Error>::new(self.0.into_iter()))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

/// Marks a borrowed serde_json value as a non-human-readable representation.
struct NonHumanReadableValue<'de>(&'de serde_json::Value);

impl<'de> SerdeDeserializer<'de> for NonHumanReadableValue<'de> {
    type Error = serde_json::Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        SerdeDeserializer::deserialize_any(self.0, visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        SerdeDeserializer::deserialize_struct(self.0, name, fields, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct
        map enum identifier ignored_any
    }
}

#[test]
fn test_ron_wire_seed_roundtrip() {
    let original = ValueContainer::Scalar(Value::Int32(42));
    let wire = ValueWireV1::try_from(original.clone()).expect("construct wire");
    let text = to_ron_string(&wire).expect("serialize wire");
    let mut decoder = RonDeserializer::from_str(&text).expect("RON input");
    let restored = ValueWireV1Seed::new().deserialize(&mut decoder).expect("decode wire");
    assert_eq!(restored.into_container(), original);
}

/// Decodes a RON envelope through the public seed and extracts its container.
fn decode_ron_wire(input: &str) -> Result<ValueContainer, RonError> {
    let mut decoder = RonDeserializer::from_str(input)?;
    ValueWireV1Seed::new()
        .deserialize(&mut decoder)
        .map(ValueWireV1::into_container)
}

/// Checks the same container with plain and explicitly named RON struct syntax.
fn assert_ron_wire_roundtrip(original: ValueContainer) {
    let wire = ValueWireV1::try_from(original.clone()).expect("construct wire");
    let plain = to_ron_string(&wire).expect("serialize wire");
    assert_eq!(decode_ron_wire(&plain).expect("decode plain RON"), original);

    let named =
        to_string_pretty(&wire, PrettyConfig::default().struct_names(true)).expect("serialize named RON structs");
    assert_eq!(decode_ron_wire(&named).expect("decode named RON structs"), original);
}

#[test]
fn test_ron_wire_roundtrips_scalar_collection_and_typed_unset() {
    for original in [
        ValueContainer::Scalar(Value::Int32(42)),
        ValueContainer::Collection(MultiValues::Int32(vec![42, 43])),
        ValueContainer::new_unset_scalar(DataType::Int32),
        ValueContainer::new_unset_collection(DataType::Int32),
        ValueContainer::Scalar(Value::Duration(Duration::new(7, 123))),
        ValueContainer::Collection(MultiValues::Duration(vec![Duration::new(7, 123)])),
    ] {
        assert_ron_wire_roundtrip(original);
    }
}

#[test]
fn test_ron_named_values_roundtrip_with_explicit_struct_names() {
    let named_scalar = NamedValue::new("port", Value::Int32(42));
    let encoded = to_string_pretty(&named_scalar, PrettyConfig::default().struct_names(true))
        .expect("serialize named scalar with struct names");
    assert_eq!(
        from_ron_str::<NamedValue>(&encoded).expect("decode named scalar with struct names"),
        named_scalar
    );

    let named_unset = NamedValue::new("unset", Value::Unset(DataType::Int32));
    let encoded = to_string_pretty(&named_unset, PrettyConfig::default().struct_names(true))
        .expect("serialize named typed unset with struct names");
    assert_eq!(
        from_ron_str::<NamedValue>(&encoded).expect("decode named typed unset"),
        named_unset
    );

    for original in [
        NamedMultiValues::new("ports", MultiValues::Int32(vec![42])),
        NamedMultiValues::new("empty", MultiValues::Int32(Vec::new())),
    ] {
        let encoded = to_string_pretty(&original, PrettyConfig::default().struct_names(true))
            .expect("serialize named collection with struct names");
        assert_eq!(
            from_ron_str::<NamedMultiValues>(&encoded).expect("decode named collection"),
            original
        );
    }
}

#[test]
fn test_ron_wire_rejects_invalid_envelope_fields_and_payloads() {
    for input in [
        "WireEnvelopeRef(version:2,value:scalar(int32(7)))",
        "WireEnvelopeRef(version:1,version:1,value:scalar(int32(7)))",
        "WireEnvelopeRef(version:1)",
        "WireEnvelopeRef(version:1,value:scalar(int32(7)),extra:0)",
        "WireEnvelopeRef(version:1,value:scalar(duration((secs:7,nanos:1000000000))))",
    ] {
        assert!(decode_ron_wire(input).is_err(), "must reject {input}");
    }
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_ron_big_decimal_roundtrip_preserves_coefficient_and_scale() {
    for original in [
        ValueContainer::Scalar(Value::BigDecimal(BigDecimal::new(BigInt::from(12300), 4))),
        ValueContainer::Collection(MultiValues::BigDecimal(vec![BigDecimal::new(BigInt::from(12300), 4)])),
    ] {
        let wire = ValueWireV1::try_from(original.clone()).expect("construct decimal wire");
        let encoded = to_string_pretty(&wire, PrettyConfig::default().struct_names(true))
            .expect("serialize decimal wire with struct names");
        let restored = decode_ron_wire(&encoded).expect("decode decimal wire");
        let (coefficient, scale) = restored
            .get_first_ref::<BigDecimal>()
            .expect("restored wire should contain a decimal")
            .as_bigint_and_scale();
        assert_eq!(coefficient.as_ref(), &BigInt::from(12300));
        assert_eq!(scale, 4);
        assert_eq!(restored, original);
    }
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_ron_big_decimal_rejects_invalid_payload() {
    for input in [
        "WireEnvelopeRef(version:1,value:scalar(bigdecimal(BigDecimalPayload(coefficient:\"0123\",scale:4))))",
        "WireEnvelopeRef(version:1,value:scalar(bigdecimal(BigDecimalPayload(coefficient:\"123\",scale:150001))))",
    ] {
        assert!(decode_ron_wire(input).is_err(), "must reject {input}");
    }
}

#[cfg(feature = "chrono")]
#[test]
fn test_wire_instant_rejects_extended_years() {
    for year in [-1, 10000] {
        let instant = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let value = Value::Instant(instant);
        assert!(
            matches!(ValueWireV1::try_from(value.clone()), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWireRefV1::try_from(&value), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWirePayloadV1::try_from(value.clone()), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWirePayloadRefV1::try_from(&value), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );

        let values = MultiValues::Instant(vec![instant]);
        assert!(
            matches!(ValueWireV1::try_from(ValueContainer::Collection(values.clone())), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWireRefV1::try_from(&ValueContainer::Collection(values.clone())), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWirePayloadV1::try_from(ValueContainer::Collection(values.clone())), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(
            matches!(ValueWirePayloadRefV1::try_from(&ValueContainer::Collection(values.clone())), Err(ValueWireEncodeError::InstantYearOutOfRange { year: actual }) if actual == year)
        );
        assert!(to_vec(&NamedValue::new("field", value)).is_err());
        assert!(to_vec(&NamedMultiValues::new("field", values)).is_err());
    }
}

#[cfg(feature = "chrono")]
#[test]
fn test_wire_instant_rejects_mixed_out_of_range_collection_and_accepts_empty() {
    let valid = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let invalid = Utc.with_ymd_and_hms(10000, 1, 1, 0, 0, 0).unwrap();
    let values = MultiValues::Instant(vec![valid, invalid]);
    assert!(matches!(
        ValueWireV1::try_from(ValueContainer::Collection(values.clone())),
        Err(ValueWireEncodeError::InstantYearOutOfRange { year: 10000 })
    ));
    assert!(matches!(
        ValueWirePayloadV1::try_from(ValueContainer::Collection(values.clone())),
        Err(ValueWireEncodeError::InstantYearOutOfRange { year: 10000 })
    ));
    let mixed = ValueContainer::Collection(values);
    assert!(matches!(
        ValueWireRefV1::try_from(&mixed),
        Err(ValueWireEncodeError::InstantYearOutOfRange { year: 10000 })
    ));
    assert!(matches!(
        ValueWirePayloadRefV1::try_from(&mixed),
        Err(ValueWireEncodeError::InstantYearOutOfRange { year: 10000 })
    ));

    let empty = ValueContainer::Collection(MultiValues::Instant(Vec::new()));
    assert!(ValueWireV1::try_from(empty).is_ok());
    assert!(ValueWireV1::try_from(ValueContainer::new_unset_scalar(DataType::Instant)).is_ok());
}

#[cfg(all(feature = "chrono", feature = "json"))]
#[test]
fn test_wire_instant_year_boundaries_round_trip() {
    for year in [0, 2026, 9999] {
        let value = Value::Instant(Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap());
        let wire = ValueWireV1::try_from(value).expect("in-range instant should encode");
        let bytes = to_vec(&wire).expect("in-range instant should serialize");
        assert_eq!(
            from_utf8(&bytes).expect("wire JSON should be UTF-8"),
            format!("{{\"version\":1,\"value\":{{\"scalar\":{{\"instant\":\"{year:04}-01-01T00:00:00Z\"}}}}}}")
        );
        let decoded = ValueWireV1::decode_json_slice(&bytes).expect("in-range instant should decode");
        assert_eq!(decoded, wire);
    }
}

#[cfg(feature = "all")]
#[test]
fn test_wire_string_map_rejects_sequence_shape() {
    let input = r#"{"scalar":{"stringmap":[]}}"#;

    let error =
        ValueWirePayloadV1::decode_json_slice(input.as_bytes()).expect_err("a string map must reject a sequence");
    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}

#[cfg(feature = "all")]
#[test]
fn test_wire_instant_rejects_invalid_timestamp() {
    let input = r#"{"scalar":{"instant":"not-an-instant"}}"#;

    let error =
        ValueWirePayloadV1::decode_json_slice(input.as_bytes()).expect_err("an instant must use RFC 3339 syntax");
    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));
}

/// Removes input coordinates so malformed collection fixtures can compare error
/// precedence.
fn collection_payload_error_without_location(payload: &str) -> String {
    crate::decode_value_wire_payload_str(payload)
        .expect_err("invalid collection payload must be rejected")
        .to_string()
        .split(" at line ")
        .next()
        .expect("error text is nonempty")
        .to_owned()
}

#[cfg(feature = "chrono")]
#[test]
fn test_wire_date_collection_reports_first_item_error_before_tail_error() {
    let valid = r#"{"version":1,"value":{"collection":{"date":["2026-07-14"]}}}"#;
    let restored = decode_value_wire_str(valid)
        .expect("valid date collection must reach the typed adapter")
        .into_container();
    assert!(restored.is_collection());
    assert_eq!(restored.data_type(), DataType::Date);
    assert_eq!(restored.len(), 1);

    let first_only = r#"{"collection":{"date":["not-a-date"]}}"#;
    let with_tail = r#"{"collection":{"date":["not-a-date",3]}}"#;
    let first_error = collection_payload_error_without_location(first_only);
    let date_error = "not-a-date"
        .parse::<NaiveDate>()
        .expect_err("invalid date text must fail to parse")
        .to_string();
    assert!(first_error.contains(&date_error), "{first_error}");
    assert!(!first_error.contains("unknown field"));
    assert_eq!(collection_payload_error_without_location(with_tail), first_error);
}

#[test]
fn test_wire_duration_collection_reports_first_item_error_before_tail_error() {
    let valid = r#"{"version":1,"value":{"collection":{"duration":[{"secs":0,"nanos":999999999}]}}}"#;
    let restored = decode_value_wire_str(valid)
        .expect("valid duration collection must reach the typed adapter")
        .into_container();
    assert!(restored.is_collection());
    assert_eq!(restored.data_type(), DataType::Duration);
    assert_eq!(restored.len(), 1);

    let first_only = r#"{"collection":{"duration":[{"secs":0,"nanos":1000000000}]}}"#;
    let with_tail = r#"{"collection":{"duration":[{"secs":0,"nanos":1000000000},3]}}"#;
    let first_error = collection_payload_error_without_location(first_only);
    assert!(
        first_error.contains("duration nanoseconds must be less than 1000000000"),
        "{first_error}"
    );
    assert_eq!(collection_payload_error_without_location(with_tail), first_error);
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_wire_decimal_collection_reports_first_item_error_before_tail_error() {
    let valid = r#"{"version":1,"value":{"collection":{"bigdecimal":[{"coefficient":"1","scale":150000}]}}}"#;
    let restored = decode_value_wire_str(valid)
        .expect("valid decimal collection must reach the typed adapter")
        .into_container();
    assert!(restored.is_collection());
    assert_eq!(restored.data_type(), DataType::BigDecimal);
    assert_eq!(restored.len(), 1);

    let first_only = r#"{"collection":{"bigdecimal":[{"coefficient":"1","scale":150001}]}}"#;
    let with_tail = r#"{"collection":{"bigdecimal":[{"coefficient":"1","scale":150001},3]}}"#;
    let first_error = collection_payload_error_without_location(first_only);
    assert!(
        first_error.contains("decimal scale exceeds the V1 maximum absolute scale"),
        "{first_error}"
    );
    assert_eq!(collection_payload_error_without_location(with_tail), first_error);
}

/// Checks every feature enabled V1 payload adapter through the public wire
/// entry point, including the scalar and collection representations.
#[cfg(feature = "all")]
#[test]
fn test_wire_payload_adapters_cover_every_value_type() {
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
        Value::Url(Url::parse("https://example.com/path").expect("valid URL")),
        Value::StringMap(map),
        Value::Json(json!({"b": 2, "a": 1})),
    ];

    for value in values {
        let payload = ValueWirePayloadV1::try_from(value.clone()).expect("supported value should encode");
        let bytes = payload.to_json_vec().expect("supported value should serialize");
        let decoded = ValueWirePayloadV1::decode_json_slice(&bytes).expect("supported value should decode");
        assert_eq!(decoded, payload);

        let named = NamedValue::new("field", value.clone());
        let named_bytes = to_vec(&named).expect("named value should serialize");
        let decoded_named = from_slice::<NamedValue>(&named_bytes).expect("named value should decode");
        assert_eq!(decoded_named, named);
    }

    macro_rules! assert_collection_round_trip {
        ($values:expr) => {{
            let values = $values;
            let payload = ValueWirePayloadV1::try_from(ValueContainer::Collection(values.clone()))
                .expect("supported collection should encode");
            let bytes = payload.to_json_vec().expect("supported collection should serialize");
            assert_eq!(
                ValueWirePayloadV1::decode_json_slice(&bytes).expect("collection should decode"),
                payload
            );

            let named = NamedMultiValues::new("field", values);
            let bytes = to_vec(&named).expect("named collection should serialize");
            assert_eq!(
                from_slice::<NamedMultiValues>(&bytes).expect("named collection should decode"),
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
    assert_collection_round_trip!(MultiValues::BigDecimal(vec![
        BigDecimal::from(123),
        BigDecimal::from(456)
    ]));
    assert_collection_round_trip!(MultiValues::String(vec![String::from("a"), String::from("b")]));
    assert_collection_round_trip!(MultiValues::Date(vec![date]));
    assert_collection_round_trip!(MultiValues::Time(vec![time]));
    assert_collection_round_trip!(MultiValues::DateTime(vec![datetime]));
    assert_collection_round_trip!(MultiValues::Instant(vec![instant]));
    assert_collection_round_trip!(MultiValues::Duration(vec![Duration::from_secs(1)]));
    assert_collection_round_trip!(MultiValues::Url(vec![
        Url::parse("https://example.com").expect("valid URL")
    ]));
    assert_collection_round_trip!(MultiValues::StringMap(vec![HashMap::from([(
        String::from("key"),
        String::from("value")
    )])]));
    assert_collection_round_trip!(MultiValues::Json(vec![json!({"a": 1})]));
}

/// Verifies the public wire facade preserves scalar values.
#[test]
fn test_wire_round_trips_scalar_value() {
    let wire = ValueWireV1::try_from(Value::Int32(7)).expect("construct V1 wire");
    let decoded = decode_value_wire_value(to_value(wire).expect("wire value should serialize"))
        .expect("wire value should decode");
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
        to_value(payload).expect("serialize V1 payload"),
        json!({"collection": {"int32": [7]}}),
    );
}
