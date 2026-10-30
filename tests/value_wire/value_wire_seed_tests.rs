// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for the explicit V1 Serde seeds.

use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "big-decimal")]
use num_bigint::BigInt;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use serde::Deserialize;
use serde::Deserializer as SerdeDeserializer;
use serde::de::DeserializeSeed;
use serde::de::EnumAccess;
use serde::de::IntoDeserializer;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde_json::Deserializer;

#[test]
fn test_value_wire_payload_v1_seed_decodes_scalar_shape() {
    let input = r#"{"scalar":{"int32":7}}"#;
    let mut deserializer = Deserializer::from_str(input);
    let payload = ValueWirePayloadV1Seed
        .deserialize(&mut deserializer)
        .expect("the valid scalar V1 payload should decode");

    assert_eq!(payload.into_container(), ValueContainer::from(7_i32));
}

#[test]
fn test_value_wire_payload_v1_seed_decodes_collection_shape() {
    let input = r#"{"collection":{"int32":[7,8]}}"#;
    let mut deserializer = Deserializer::from_str(input);
    let payload = ValueWirePayloadV1Seed::new()
        .deserialize(&mut deserializer)
        .expect("the valid collection V1 payload should decode");

    assert_eq!(payload.into_container(), ValueContainer::from(vec![7_i32, 8]));
}

#[test]
fn test_value_wire_payload_v1_seed_rejects_an_unknown_shape_tag() {
    let input = r#"{"unknown":{"int32":7}}"#;
    let mut deserializer = Deserializer::from_str(input);

    let error = ValueWirePayloadV1Seed::new()
        .deserialize(&mut deserializer)
        .expect_err("an unknown payload shape must be rejected");

    assert!(
        error.to_string().contains("unknown variant `unknown`"),
        "unexpected seed error: {error}"
    );
}

#[test]
fn test_value_wire_v1_seed_preserves_the_golden_envelope() {
    let input = r#"{"version":1,"value":{"scalar":{"string":"ready"}}}"#;
    let mut deserializer = Deserializer::from_str(input);

    let wire = ValueWireV1Seed
        .deserialize(&mut deserializer)
        .expect("the valid V1 envelope should decode");

    assert_eq!(wire.container(), &ValueContainer::from("ready"));
}

#[test]
fn test_value_wire_v1_seed_rejects_an_unsupported_version_precisely() {
    let input = r#"{"version":2,"value":{"scalar":{"int32":7}}}"#;
    let mut deserializer = Deserializer::from_str(input);

    let error = ValueWireV1Seed::new()
        .deserialize(&mut deserializer)
        .expect_err("V1 seed must reject another wire version");

    assert!(
        error.to_string().contains("unsupported qubit-value wire version 2"),
        "unexpected seed error: {error}"
    );
}

#[test]
fn test_wire_record_json_rejects_sequence_form() {
    for input in [
        r#"[1,{"scalar":{"int32":7}}]"#,
        r#"{"version":1,"value":{"scalar":{"duration":[1,2]}}}"#,
        r#"{"version":1,"value":{"collection":{"duration":[[1,2]]}}}"#,
    ] {
        assert!(ValueWireV1::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }
}

#[test]
fn test_wire_record_json_requires_known_unique_fields() {
    for input in [
        r#"{"version":1}"#,
        r#"{"version":1,"value":{"scalar":{"int32":7}},"extra":0}"#,
        r#"{"version":1,"version":1,"value":{"scalar":{"int32":7}}}"#,
    ] {
        assert!(ValueWireV1::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }
}

#[test]
fn test_wire_record_json_accepts_reordered_fields_and_json_array_payload() {
    let reordered = r#"{"value":{"scalar":{"int32":7}},"version":1}"#;
    assert!(ValueWireV1::decode_json_slice(reordered.as_bytes()).is_ok());

    let json_array = r#"{"version":1,"value":{"scalar":{"json":[1,2]}}}"#;
    assert!(ValueWireV1::decode_json_slice(json_array.as_bytes()).is_ok());
}

#[test]
fn test_named_wire_records_json_reject_sequence_form() {
    use qubit_value::NamedMultiValues;
    use qubit_value::NamedValue;

    for input in [
        r#"["count",{"version":1,"value":{"scalar":{"int32":7}}}]"#,
        r#"{"name":"count","value":[1,{"scalar":{"int32":7}}]}"#,
    ] {
        assert!(NamedValue::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }

    for input in [
        r#"["counts",{"version":1,"value":{"collection":{"int32":[7]}}}]"#,
        r#"{"name":"counts","value":[1,{"collection":{"int32":[7]}}]}"#,
    ] {
        assert!(
            NamedMultiValues::decode_json_slice(input.as_bytes()).is_err(),
            "{input}"
        );
    }
}

#[test]
fn test_named_wire_records_json_accept_reordered_object_fields() {
    use qubit_value::NamedMultiValues;
    use qubit_value::NamedValue;

    let named_value = r#"{"value":{"value":{"scalar":{"int32":7}},"version":1},"name":"count"}"#;
    assert!(NamedValue::decode_json_slice(named_value.as_bytes()).is_ok());

    let named_values = r#"{"value":{"value":{"collection":{"int32":[7]}},"version":1},"name":"counts"}"#;
    assert!(NamedMultiValues::decode_json_slice(named_values.as_bytes()).is_ok());
}

#[test]
fn test_named_wire_records_json_reject_invalid_map_fields() {
    use qubit_value::NamedMultiValues;
    use qubit_value::NamedValue;

    for input in [
        r#"{"value":{"version":1,"value":{"scalar":{"int32":7}}}}"#,
        r#"{"name":"count"}"#,
        r#"{"name":"count","name":"other","value":{"version":1,"value":{"scalar":{"int32":7}}}}"#,
        r#"{"name":"count","value":{"version":1,"value":{"scalar":{"int32":7}}},"value":{"version":1,"value":{"scalar":{"int32":8}}}}"#,
        r#"{"name":"count","value":{"version":1,"value":{"scalar":{"int32":7}}},"extra":0}"#,
    ] {
        assert!(NamedValue::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }

    for input in [
        r#"{"value":{"version":1,"value":{"collection":{"int32":[7]}}}}"#,
        r#"{"name":"counts"}"#,
        r#"{"name":"counts","value":{"version":1,"value":{"collection":{"int32":[7]}}},"value":{"version":1,"value":{"collection":{"int32":[8]}}}}"#,
        r#"{"name":"counts","name":"other","value":{"version":1,"value":{"collection":{"int32":[7]}}}}"#,
        r#"{"name":"counts","value":{"version":1,"value":{"collection":{"int32":[7]}}},"extra":0}"#,
    ] {
        assert!(
            NamedMultiValues::decode_json_slice(input.as_bytes()).is_err(),
            "{input}"
        );
    }
}

#[test]
fn test_duration_wire_record_json_rejects_invalid_map_fields() {
    for input in [
        r#"{"version":1,"value":{"scalar":{"duration":{"secs":1}}}}"#,
        r#"{"version":1,"value":{"scalar":{"duration":{"nanos":2}}}}"#,
        r#"{"version":1,"value":{"scalar":{"duration":{"secs":1,"secs":2,"nanos":0}}}}"#,
        r#"{"version":1,"value":{"scalar":{"duration":{"secs":1,"nanos":0,"extra":true}}}}"#,
        r#"{"version":1,"value":{"scalar":{"duration":{"secs":1,"nanos":1000000000}}}}"#,
        r#"{"version":1,"value":{"collection":{"duration":[{"secs":1,"nanos":0,"extra":true}]}}}"#,
    ] {
        assert!(ValueWireV1::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_wire_record_json_rejects_sequence_form() {
    for input in [
        r#"{"version":1,"value":{"scalar":{"bigdecimal":["1",0]}}}"#,
        r#"{"version":1,"value":{"collection":{"bigdecimal":[["1",0]]}}}"#,
    ] {
        assert!(ValueWireV1::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_wire_record_json_rejects_invalid_map_fields() {
    for input in [
        r#"{"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1"}}}}"#,
        r#"{"version":1,"value":{"scalar":{"bigdecimal":{"scale":0}}}}"#,
        r#"{"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1","coefficient":"2","scale":0}}}}"#,
        r#"{"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1","scale":0,"extra":true}}}}"#,
        r#"{"version":1,"value":{"collection":{"bigdecimal":[{"coefficient":"1","scale":0,"extra":true}]}}}"#,
        r#"{"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1","scale":150001}}}}"#,
    ] {
        assert!(ValueWireV1::decode_json_slice(input.as_bytes()).is_err(), "{input}");
    }
}

#[test]
fn test_wire_envelope_non_human_readable_deserialize_struct_accepts_sequence() {
    let input = serde_json::json!([1, {"scalar": {"int32": 7}}]);
    let deserializer = input.into_deserializer();
    let wire = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(deserializer))
        .expect("non-human-readable struct deserialization should retain sequence support");

    assert_eq!(wire.container(), &ValueContainer::from(7_i32));
}

#[test]
fn test_duration_payload_non_human_readable_deserialize_struct_accepts_sequences() {
    let scalar = serde_json::json!({"version": 1, "value": {"scalar": {"duration": [1, 2]}}});
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(scalar.into_deserializer()))
        .expect("non-human-readable scalar DurationPayload should retain sequence support");
    assert_eq!(decoded.container(), &ValueContainer::from(Duration::new(1, 2)));

    let collection = serde_json::json!({"version": 1, "value": {"collection": {"duration": [[1, 2]]}}});
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(collection.into_deserializer()))
        .expect("non-human-readable collection DurationPayload should retain sequence support");
    assert_eq!(decoded.container(), &ValueContainer::from(vec![Duration::new(1, 2)]));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_big_decimal_payload_non_human_readable_deserialize_struct_accepts_sequences() {
    let scalar = serde_json::json!({"version": 1, "value": {"scalar": {"bigdecimal": ["1", 0]}}});
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(scalar.into_deserializer()))
        .expect("non-human-readable scalar BigDecimalPayload should retain sequence support");
    assert!(decoded.container().is_scalar());
    let (coefficient, scale) = decoded
        .container()
        .get_first_ref::<BigDecimal>()
        .expect("the scalar wire value should be a BigDecimal")
        .as_bigint_and_scale();
    assert_eq!(coefficient.as_ref(), &BigInt::from(1));
    assert_eq!(scale, 0);

    let collection = serde_json::json!({"version": 1, "value": {"collection": {"bigdecimal": [["1", 0]]}}});
    let decoded = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(collection.into_deserializer()))
        .expect("non-human-readable collection BigDecimalPayload should retain sequence support");
    assert!(decoded.container().is_collection());
    let (coefficient, scale) = decoded
        .container()
        .get_first_ref::<BigDecimal>()
        .expect("the collection wire value should contain a BigDecimal")
        .as_bigint_and_scale();
    assert_eq!(coefficient.as_ref(), &BigInt::from(1));
    assert_eq!(scale, 0);
}

#[test]
fn test_wire_envelope_non_human_readable_deserialize_struct_reports_observed_extra_length() {
    let input = serde_json::json!([1, {"scalar": {"int32": 7}}, true]);
    let error = ValueWireV1Seed
        .deserialize(NonHumanReadableDeserializer(input.into_deserializer()))
        .expect_err("an extra sequence element must be rejected");

    assert!(error.to_string().contains("invalid length 3"), "{error}");
}

#[test]
fn test_named_non_human_readable_deserialize_struct_accepts_sequence() {
    use qubit_value::NamedMultiValues;
    use qubit_value::NamedValue;

    let named_value = serde_json::json!(["count", {"version": 1, "value": {"scalar": {"int32": 7}}}]);
    let decoded = NamedValue::deserialize(NonHumanReadableDeserializer(named_value.into_deserializer()))
        .expect("non-human-readable named-value struct should retain sequence support");
    assert_eq!(decoded.name(), "count");
    assert_eq!(decoded.value(), &Value::Int32(7));

    let named_values = serde_json::json!(["counts", {"version": 1, "value": {"collection": {"int32": [7]}}}]);
    let decoded = NamedMultiValues::deserialize(NonHumanReadableDeserializer(named_values.into_deserializer()))
        .expect("non-human-readable named-values struct should retain sequence support");
    assert_eq!(decoded.name(), "counts");
    assert_eq!(decoded.values(), &MultiValues::Int32(vec![7]));
}

struct NonHumanReadableDeserializer<D>(D);

impl<'de, D> SerdeDeserializer<'de> for NonHumanReadableDeserializer<D>
where
    D: SerdeDeserializer<'de>,
{
    type Error = D::Error;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_any(NonHumanReadableVisitor(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .deserialize_struct(name, fields, NonHumanReadableVisitor(visitor))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .deserialize_enum(name, variants, NonHumanReadableVisitor(visitor))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_seq(NonHumanReadableVisitor(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_tuple(len, NonHumanReadableVisitor(visitor))
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .deserialize_tuple_struct(name, len, NonHumanReadableVisitor(visitor))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_map(NonHumanReadableVisitor(visitor))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_option(NonHumanReadableVisitor(visitor))
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .deserialize_newtype_struct(name, NonHumanReadableVisitor(visitor))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.deserialize_ignored_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        unit unit_struct identifier
    }
}

struct NonHumanReadableSeed<S>(S);

impl<'de, S> DeserializeSeed<'de> for NonHumanReadableSeed<S>
where
    S: DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: SerdeDeserializer<'de>,
    {
        self.0.deserialize(NonHumanReadableDeserializer(deserializer))
    }
}

struct NonHumanReadableSeqAccess<A>(A);

impl<'de, A> SeqAccess<'de> for NonHumanReadableSeqAccess<A>
where
    A: SeqAccess<'de>,
{
    type Error = A::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.0.next_element_seed(NonHumanReadableSeed(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct NonHumanReadableMapAccess<A>(A);

impl<'de, A> MapAccess<'de> for NonHumanReadableMapAccess<A>
where
    A: MapAccess<'de>,
{
    type Error = A::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.0.next_key_seed(NonHumanReadableSeed(seed))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0.next_value_seed(NonHumanReadableSeed(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct NonHumanReadableEnumAccess<A>(A);

impl<'de, A> EnumAccess<'de> for NonHumanReadableEnumAccess<A>
where
    A: EnumAccess<'de>,
{
    type Error = A::Error;
    type Variant = NonHumanReadableVariantAccess<A::Variant>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0
            .variant_seed(NonHumanReadableSeed(seed))
            .map(|(value, variant)| (value, NonHumanReadableVariantAccess(variant)))
    }
}

struct NonHumanReadableVariantAccess<A>(A);

impl<'de, A> VariantAccess<'de> for NonHumanReadableVariantAccess<A>
where
    A: VariantAccess<'de>,
{
    type Error = A::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        self.0.unit_variant()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        self.0.newtype_variant_seed(NonHumanReadableSeed(seed))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.tuple_variant(len, NonHumanReadableVisitor(visitor))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.struct_variant(fields, NonHumanReadableVisitor(visitor))
    }
}

struct NonHumanReadableVisitor<V>(V);

impl<'de, V> Visitor<'de> for NonHumanReadableVisitor<V>
where
    V: Visitor<'de>,
{
    type Value = V::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_bool(value)
    }
    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_i8(value)
    }
    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_i16(value)
    }
    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_i32(value)
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_i64(value)
    }
    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_i128(value)
    }
    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_u8(value)
    }
    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_u16(value)
    }
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_u32(value)
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_u64(value)
    }
    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_u128(value)
    }
    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_f32(value)
    }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_f64(value)
    }
    fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_char(value)
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_str(value)
    }
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_borrowed_str(value)
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_string(value)
    }
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_bytes(value)
    }
    fn visit_borrowed_bytes<E>(self, value: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_borrowed_bytes(value)
    }
    fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_byte_buf(value)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_none()
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: SerdeDeserializer<'de>,
    {
        self.0.visit_some(NonHumanReadableDeserializer(deserializer))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.0.visit_unit()
    }
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: SerdeDeserializer<'de>,
    {
        self.0.visit_newtype_struct(NonHumanReadableDeserializer(deserializer))
    }
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.0.visit_seq(NonHumanReadableSeqAccess(seq))
    }
    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.0.visit_map(NonHumanReadableMapAccess(map))
    }
    fn visit_enum<A>(self, access: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        self.0.visit_enum(NonHumanReadableEnumAccess(access))
    }
}
