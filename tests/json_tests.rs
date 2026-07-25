// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, TimeZone, Utc};
use num_bigint::BigInt;
use qubit_datatype::{DataConversionError, DataConversionOptions, DataType, InvalidValueReason};
use qubit_value::{MultiValues, Value, ValueContainer, ValueError};
use serde::de::value::{
    EnumAccessDeserializer, Error as DeError, MapDeserializer, SeqDeserializer, StrDeserializer,
};
use serde::de::{self, DeserializeSeed, EnumAccess, IntoDeserializer, VariantAccess, Visitor};
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[test]
fn test_value_natural_json_projection() {
    assert_eq!(
        Value::Unset(DataType::Int32).to_json_value().unwrap(),
        serde_json::Value::Null
    );
    assert_eq!(Value::Bool(true).to_json_value().unwrap(), json!(true));
    assert_eq!(Value::Int64(-42).to_json_value().unwrap(), json!(-42));
    assert_eq!(
        Value::Int128(i128::MAX).to_json_value().unwrap(),
        json!(i128::MAX.to_string())
    );
    assert_eq!(
        Value::UInt128(u128::MAX).to_json_value().unwrap(),
        json!(u128::MAX.to_string())
    );
    assert_eq!(
        Value::BigInteger(BigInt::from(i128::MAX))
            .to_json_value()
            .unwrap(),
        json!(i128::MAX.to_string())
    );
    assert_eq!(
        Value::BigDecimal(BigDecimal::from_str("123.4500").unwrap())
            .to_json_value()
            .unwrap(),
        json!("123.4500")
    );
    assert_eq!(Value::Char('界').to_json_value().unwrap(), json!("界"));
    assert_eq!(
        Value::String("text".to_string()).to_json_value().unwrap(),
        json!("text")
    );

    let date = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
    assert_eq!(
        Value::Date(date).to_json_value().unwrap(),
        json!("2026-07-14")
    );
    let instant = Utc.with_ymd_and_hms(2026, 7, 14, 1, 2, 3).unwrap();
    assert_eq!(
        Value::Instant(instant).to_json_value().unwrap(),
        json!(instant.to_string())
    );
    assert_eq!(
        Value::Duration(Duration::from_millis(2))
            .to_json_value()
            .unwrap(),
        json!("2ms")
    );
    let url = Url::parse("https://example.com/path").unwrap();
    assert_eq!(
        Value::Url(url.clone()).to_json_value().unwrap(),
        json!(url.to_string())
    );

    let map = HashMap::from([("key".to_string(), "value".to_string())]);
    assert_eq!(
        Value::StringMap(map).to_json_value().unwrap(),
        json!({"key": "value"})
    );
    let nested = json!({"items": [1, null, true]});
    assert_eq!(Value::Json(nested.clone()).to_json_value().unwrap(), nested);
}

#[test]
fn test_duration_natural_json_projection_obeys_conversion_options() {
    let value = Value::Duration(Duration::from_micros(1_500));
    assert!(matches!(
        value.to_json_value(),
        Err(ValueError::DataConversion(error))
            if matches!(error.reason(), Some(InvalidValueReason::PrecisionLoss))
    ));
    assert_eq!(
        value
            .to_json_value_with(&DataConversionOptions::lossy())
            .unwrap(),
        json!("2ms")
    );

    let values = MultiValues::Duration(vec![Duration::from_micros(1_500)]);
    assert!(
        values
            .to_json_value_with(DataConversionOptions::default_ref())
            .is_err()
    );
    assert_eq!(
        values
            .to_json_value_with(&DataConversionOptions::lossy())
            .unwrap(),
        json!(["2ms"])
    );

    let container = ValueContainer::Collection(values);
    assert_eq!(
        container
            .to_json_value_with(&DataConversionOptions::lossy())
            .unwrap(),
        json!(["2ms"])
    );
}

#[test]
fn test_multi_values_natural_json_projection_preserves_collection_shape() {
    assert_eq!(
        MultiValues::Unset(DataType::Int32).to_json_value().unwrap(),
        serde_json::Value::Null
    );
    assert_eq!(
        MultiValues::Int32(Vec::new()).to_json_value().unwrap(),
        json!([])
    );
    assert_eq!(
        MultiValues::Int32(vec![42]).to_json_value().unwrap(),
        json!([42])
    );
    assert_eq!(
        MultiValues::Int32(vec![1, 2, 3]).to_json_value().unwrap(),
        json!([1, 2, 3])
    );
    assert_eq!(
        MultiValues::StringMap(vec![HashMap::from([(
            "key".to_string(),
            "value".to_string(),
        )])])
        .to_json_value()
        .unwrap(),
        json!([{"key": "value"}])
    );
}

#[test]
fn test_natural_json_projection_reports_non_finite_values() {
    assert!(matches!(
        Value::Float64(f64::NAN).to_json_value(),
        Err(ValueError::DataConversion(error))
            if error == DataConversionError::invalid(
                DataType::Float64,
                DataType::Json,
                InvalidValueReason::NonFinite,
            )
    ));

    assert!(matches!(
        MultiValues::Float32(vec![1.0, f32::INFINITY]).to_json_value(),
        Err(ValueError::DataListConversion(error))
            if error.source_index() == 1
                && error.conversion_error() == &DataConversionError::invalid(
                    DataType::Float32,
                    DataType::Json,
                    InvalidValueReason::NonFinite,
                )
    ));
}

#[derive(Serialize)]
struct SerializablePayload {
    values: Vec<f64>,
    missing: Option<String>,
}

struct NonFiniteMapKey;

impl Serialize for NonFiniteMapKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&f64::INFINITY, &1_u8)?;
        map.end()
    }
}

#[test]
fn test_from_serializable_rejects_nested_non_finite_floats() {
    assert!(matches!(
        Value::from_serializable(&Value::Float64(f64::NAN)),
        Err(ValueError::DataConversion(error))
            if matches!(error.reason(), Some(InvalidValueReason::NonFinite))
    ));

    assert!(matches!(
        Value::from_serializable(&f64::NAN),
        Err(ValueError::DataConversion(error))
            if error == DataConversionError::invalid(
                DataType::Json,
                DataType::Json,
                InvalidValueReason::NonFinite,
            )
    ));

    let payload = SerializablePayload {
        values: vec![1.0, f64::NEG_INFINITY],
        missing: None,
    };
    assert!(matches!(
        Value::from_serializable(&payload),
        Err(ValueError::DataConversion(error))
            if matches!(error.reason(), Some(InvalidValueReason::NonFinite))
    ));

    assert!(matches!(
        Value::from_serializable(&NonFiniteMapKey),
        Err(ValueError::DataConversion(error))
            if matches!(error.reason(), Some(InvalidValueReason::NonFinite))
    ));

    for payload in [
        InvalidEnumPayload::Tuple(f64::NAN, 1),
        InvalidEnumPayload::Struct {
            value: f64::INFINITY,
        },
    ] {
        assert!(matches!(
            Value::from_serializable(&payload),
            Err(ValueError::DataConversion(error))
                if matches!(error.reason(), Some(InvalidValueReason::NonFinite))
        ));
    }
}

#[test]
fn test_from_serializable_preserves_legitimate_null() {
    let payload = SerializablePayload {
        values: vec![1.0, 2.0],
        missing: None,
    };
    assert_eq!(
        Value::from_serializable(&payload).unwrap(),
        Value::Json(json!({"values": [1.0, 2.0], "missing": null}))
    );
}

#[derive(Serialize)]
struct PrimitivePayload {
    boolean: bool,
    int8: i8,
    int16: i16,
    int32: i32,
    int64: i64,
    int128: i128,
    uint8: u8,
    uint16: u16,
    uint32: u32,
    uint64: u64,
    uint128: u128,
    float32: f32,
    float64: f64,
    character: char,
    string: String,
    present: Option<u8>,
}

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
struct NewtypeStruct(u8);

#[derive(Serialize)]
struct TupleStruct(u8, u16);

#[derive(Serialize)]
enum EnumPayload {
    Unit,
    Newtype(u8),
    Tuple(u8, u16),
    Struct { value: u8 },
}

#[derive(Serialize)]
enum InvalidEnumPayload {
    Tuple(f64, u8),
    Struct { value: f64 },
}

struct BytePayload;

impl Serialize for BytePayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&[1, 2, 3])
    }
}

struct DisplayPayload;

impl fmt::Display for DisplayPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("displayed")
    }
}

impl Serialize for DisplayPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

struct ValueBeforeMapKey;

impl Serialize for ValueBeforeMapKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_value(&1_u8)?;
        map.end()
    }
}

struct PendingMapKey;

impl Serialize for PendingMapKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_key("pending")?;
        map.end()
    }
}

#[derive(Clone, Copy)]
enum MapKeyKind {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Float32,
    Float64,
    NonFiniteFloat32,
    NonFiniteFloat64,
    Char,
    String,
    Bytes,
    None,
    Some,
    Unit,
    UnitStruct,
    UnitVariant,
    NewtypeStruct,
    NewtypeVariant,
    Sequence,
    Tuple,
    TupleStruct,
    TupleVariant,
    Map,
    Struct,
    StructVariant,
    CollectString,
}

impl Serialize for MapKeyKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Bool => serializer.serialize_bool(true),
            Self::Int8 => serializer.serialize_i8(-8),
            Self::Int16 => serializer.serialize_i16(-16),
            Self::Int32 => serializer.serialize_i32(-32),
            Self::Int64 => serializer.serialize_i64(-64),
            Self::Int128 => serializer.serialize_i128(-128),
            Self::UInt8 => serializer.serialize_u8(8),
            Self::UInt16 => serializer.serialize_u16(16),
            Self::UInt32 => serializer.serialize_u32(32),
            Self::UInt64 => serializer.serialize_u64(64),
            Self::UInt128 => serializer.serialize_u128(128),
            Self::Float32 => serializer.serialize_f32(3.5),
            Self::Float64 => serializer.serialize_f64(7.25),
            Self::NonFiniteFloat32 => serializer.serialize_f32(f32::INFINITY),
            Self::NonFiniteFloat64 => serializer.serialize_f64(f64::INFINITY),
            Self::Char => serializer.serialize_char('k'),
            Self::String => serializer.serialize_str("key"),
            Self::Bytes => serializer.serialize_bytes(&[1, 2]),
            Self::None => serializer.serialize_none(),
            Self::Some => serializer.serialize_some(&1_u8),
            Self::Unit => serializer.serialize_unit(),
            Self::UnitStruct => serializer.serialize_unit_struct("Key"),
            Self::UnitVariant => serializer.serialize_unit_variant("Key", 0, "Unit"),
            Self::NewtypeStruct => serializer.serialize_newtype_struct("Key", &1_u8),
            Self::NewtypeVariant => {
                serializer.serialize_newtype_variant("Key", 0, "Newtype", &1_u8)
            }
            Self::Sequence => serializer.serialize_seq(Some(0))?.end(),
            Self::Tuple => serializer.serialize_tuple(0)?.end(),
            Self::TupleStruct => serializer.serialize_tuple_struct("Key", 0)?.end(),
            Self::TupleVariant => serializer
                .serialize_tuple_variant("Key", 0, "Tuple", 0)?
                .end(),
            Self::Map => serializer.serialize_map(Some(0))?.end(),
            Self::Struct => serializer.serialize_struct("Key", 0)?.end(),
            Self::StructVariant => serializer
                .serialize_struct_variant("Key", 0, "Struct", 0)?
                .end(),
            Self::CollectString => serializer.collect_str(&DisplayPayload),
        }
    }
}

struct SingleKeyMap(MapKeyKind);

impl Serialize for SingleKeyMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.0, &true)?;
        map.end()
    }
}

struct DeserializerSequence<T>(Vec<T>);

impl<'de, T> IntoDeserializer<'de, DeError> for DeserializerSequence<T>
where
    T: IntoDeserializer<'de, DeError>,
{
    type Deserializer = SeqDeserializer<std::vec::IntoIter<T>, DeError>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.0.into_iter())
    }
}

struct TaggedPayload<V> {
    variant: &'static str,
    value: V,
}

impl<'de, V> EnumAccess<'de> for TaggedPayload<V>
where
    V: IntoDeserializer<'de, DeError>,
{
    type Error = DeError;
    type Variant = TaggedVariant<V>;

    fn variant_seed<S>(self, seed: S) -> Result<(S::Value, Self::Variant), Self::Error>
    where
        S: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(StrDeserializer::<DeError>::new(self.variant))?;
        Ok((variant, TaggedVariant(self.value)))
    }
}

struct TaggedVariant<V>(V);

impl<'de, V> VariantAccess<'de> for TaggedVariant<V>
where
    V: IntoDeserializer<'de, DeError>,
{
    type Error = DeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(de::Error::custom("expected a newtype payload"))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.0.into_deserializer())
    }

    fn tuple_variant<T>(self, _len: usize, _visitor: T) -> Result<T::Value, Self::Error>
    where
        T: Visitor<'de>,
    {
        Err(de::Error::custom("expected a newtype payload"))
    }

    fn struct_variant<T>(
        self,
        _fields: &'static [&'static str],
        _visitor: T,
    ) -> Result<T::Value, Self::Error>
    where
        T: Visitor<'de>,
    {
        Err(de::Error::custom("expected a newtype payload"))
    }
}

/// Builds a Serde enum deserializer for one externally tagged payload.
fn tagged_payload<'de, V>(
    variant: &'static str,
    value: V,
) -> EnumAccessDeserializer<TaggedPayload<V>>
where
    V: IntoDeserializer<'de, DeError>,
{
    EnumAccessDeserializer::new(TaggedPayload { variant, value })
}

/// Holds one of two heterogeneous deserializer inputs.
enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Delegates deserialization to one of two concrete deserializer types.
enum EitherDeserializer<L, R> {
    Left(L),
    Right(R),
}

impl<'de, L, R> IntoDeserializer<'de, DeError> for Either<L, R>
where
    L: IntoDeserializer<'de, DeError>,
    R: IntoDeserializer<'de, DeError>,
{
    type Deserializer = EitherDeserializer<L::Deserializer, R::Deserializer>;

    fn into_deserializer(self) -> Self::Deserializer {
        match self {
            Self::Left(value) => EitherDeserializer::Left(value.into_deserializer()),
            Self::Right(value) => EitherDeserializer::Right(value.into_deserializer()),
        }
    }
}

impl<'de, L, R> serde::Deserializer<'de> for EitherDeserializer<L, R>
where
    L: serde::Deserializer<'de, Error = DeError>,
    R: serde::Deserializer<'de, Error = DeError>,
{
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Left(value) => value.deserialize_any(visitor),
            Self::Right(value) => value.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

/// Builds a complete V1 envelope around a custom payload deserializer.
fn wire_payload<'de, V>(
    shape: &'static str,
    variant: &'static str,
    value: V,
) -> impl serde::Deserializer<'de, Error = DeError>
where
    V: IntoDeserializer<'de, DeError>,
{
    let payload = tagged_payload(variant, value);
    let shape = tagged_payload(shape, payload);
    MapDeserializer::new(
        vec![
            ("version", Either::Left(1_u8)),
            ("value", Either::Right(shape)),
        ]
        .into_iter(),
    )
}

/// Returns whether strict serialization classifies the input as a generic
/// JSON serialization failure.
fn is_serialization_error<T>(value: &T) -> bool
where
    T: Serialize,
{
    matches!(
        Value::from_serializable(value),
        Err(ValueError::DataConversion(error))
            if matches!(
                error.reason(),
                Some(InvalidValueReason::Serialization { .. }),
            )
    )
}

#[test]
fn test_strict_json_serializer_covers_primitive_and_compound_shapes() {
    let primitives = PrimitivePayload {
        boolean: true,
        int8: -8,
        int16: -16,
        int32: -32,
        int64: -64,
        int128: 128,
        uint8: 8,
        uint16: 16,
        uint32: 32,
        uint64: 64,
        uint128: 128,
        float32: 3.5,
        float64: 7.25,
        character: '界',
        string: "text".to_string(),
        present: Some(1),
    };
    assert!(matches!(
        Value::from_serializable(&primitives),
        Ok(Value::Json(serde_json::Value::Object(_)))
    ));
    assert_eq!(
        Value::from_serializable(&BytePayload).unwrap(),
        Value::Json(json!([1, 2, 3]))
    );
    assert_eq!(
        Value::from_serializable(&()).unwrap(),
        Value::Json(serde_json::Value::Null)
    );
    assert_eq!(
        Value::from_serializable(&UnitStruct).unwrap(),
        Value::Json(serde_json::Value::Null)
    );
    assert_eq!(
        Value::from_serializable(&NewtypeStruct(1)).unwrap(),
        Value::Json(json!(1))
    );
    assert_eq!(
        Value::from_serializable(&EnumPayload::Unit).unwrap(),
        Value::Json(json!("Unit"))
    );
    assert_eq!(
        Value::from_serializable(&EnumPayload::Newtype(1)).unwrap(),
        Value::Json(json!({"Newtype": 1}))
    );
    assert_eq!(
        Value::from_serializable(&vec![1_u8, 2]).unwrap(),
        Value::Json(json!([1, 2]))
    );
    assert_eq!(
        Value::from_serializable(&(1_u8, 2_u16)).unwrap(),
        Value::Json(json!([1, 2]))
    );
    assert_eq!(
        Value::from_serializable(&TupleStruct(1, 2)).unwrap(),
        Value::Json(json!([1, 2]))
    );
    assert_eq!(
        Value::from_serializable(&EnumPayload::Tuple(1, 2)).unwrap(),
        Value::Json(json!({"Tuple": [1, 2]}))
    );
    assert_eq!(
        Value::from_serializable(&BTreeMap::from([("key", 1_u8)])).unwrap(),
        Value::Json(json!({"key": 1}))
    );
    assert_eq!(
        Value::from_serializable(&EnumPayload::Struct { value: 1 }).unwrap(),
        Value::Json(json!({"Struct": {"value": 1}}))
    );
    assert_eq!(
        Value::from_serializable(&DisplayPayload).unwrap(),
        Value::Json(json!("displayed"))
    );
}

#[test]
fn test_strict_json_serializer_rejects_wide_numbers_and_invalid_map_state() {
    assert_eq!(
        Value::from_serializable(&-1_i128).unwrap(),
        Value::Json(json!(-1))
    );
    assert!(is_serialization_error(&(u64::MAX as i128 + 1)));
    assert!(is_serialization_error(&(u64::MAX as u128 + 1)));
    assert!(is_serialization_error(&ValueBeforeMapKey));
    assert!(is_serialization_error(&PendingMapKey));
}

#[test]
fn test_strict_json_map_key_serializer_covers_supported_key_shapes() {
    let supported = [
        MapKeyKind::Bool,
        MapKeyKind::Int8,
        MapKeyKind::Int16,
        MapKeyKind::Int32,
        MapKeyKind::Int64,
        MapKeyKind::Int128,
        MapKeyKind::UInt8,
        MapKeyKind::UInt16,
        MapKeyKind::UInt32,
        MapKeyKind::UInt64,
        MapKeyKind::UInt128,
        MapKeyKind::Float32,
        MapKeyKind::Float64,
        MapKeyKind::Char,
        MapKeyKind::String,
        MapKeyKind::UnitVariant,
        MapKeyKind::NewtypeStruct,
        MapKeyKind::CollectString,
    ];

    for key in supported {
        assert!(matches!(
            Value::from_serializable(&SingleKeyMap(key)),
            Ok(Value::Json(serde_json::Value::Object(_)))
        ));
    }
}

#[test]
fn test_strict_json_map_key_serializer_rejects_unsupported_key_shapes() {
    let unsupported = [
        MapKeyKind::Bytes,
        MapKeyKind::None,
        MapKeyKind::Some,
        MapKeyKind::Unit,
        MapKeyKind::UnitStruct,
        MapKeyKind::NewtypeVariant,
        MapKeyKind::Sequence,
        MapKeyKind::Tuple,
        MapKeyKind::TupleStruct,
        MapKeyKind::TupleVariant,
        MapKeyKind::Map,
        MapKeyKind::Struct,
        MapKeyKind::StructVariant,
    ];

    for key in unsupported {
        assert!(is_serialization_error(&SingleKeyMap(key)));
    }

    for key in [MapKeyKind::NonFiniteFloat32, MapKeyKind::NonFiniteFloat64] {
        assert!(matches!(
            Value::from_serializable(&SingleKeyMap(key)),
            Err(ValueError::DataConversion(error))
                if matches!(error.reason(), Some(InvalidValueReason::NonFinite))
        ));
    }
}

#[test]
fn test_value_wire_v1_deserialization_rejects_non_finite_payloads() {
    for error in [
        Value::deserialize(wire_payload("scalar", "float32", f32::NAN)).unwrap_err(),
        Value::deserialize(wire_payload("scalar", "float64", f64::INFINITY)).unwrap_err(),
    ] {
        assert!(
            error
                .to_string()
                .contains("non-finite floating-point value"),
            "{error}",
        );
    }

    let error = MultiValues::deserialize(wire_payload(
        "collection",
        "float32",
        DeserializerSequence(vec![1.0_f32, f32::NAN]),
    ))
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("non-finite floating-point value")
    );

    let error = MultiValues::deserialize(wire_payload(
        "collection",
        "float64",
        DeserializerSequence(vec![1.0_f64, f64::NEG_INFINITY]),
    ))
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("non-finite floating-point value")
    );
}

#[test]
fn test_value_wire_v1_float_deserialization_propagates_malformed_payloads() {
    assert!(Value::deserialize(wire_payload("scalar", "float32", "not-a-float")).is_err(),);
    assert!(Value::deserialize(wire_payload("scalar", "float64", "not-a-float")).is_err(),);
    assert!(
        MultiValues::deserialize(wire_payload(
            "collection",
            "float32",
            DeserializerSequence(vec!["not-a-float"]),
        ))
        .is_err(),
    );
    assert!(
        MultiValues::deserialize(wire_payload(
            "collection",
            "float64",
            DeserializerSequence(vec!["not-a-float"]),
        ))
        .is_err(),
    );
}
