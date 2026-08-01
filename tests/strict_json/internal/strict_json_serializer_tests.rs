// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strict JSON scalar serializer behavior.

#[cfg(all(feature = "converter", feature = "json"))]
use std::fmt;

#[cfg(all(feature = "converter", feature = "json"))]
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
#[cfg(all(feature = "converter", feature = "json"))]
use serde::{Serialize, Serializer};

#[cfg(all(feature = "converter", feature = "json"))]
struct ScalarProbe(u8);

#[cfg(all(feature = "converter", feature = "json"))]
impl Serialize for ScalarProbe {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            0 => serializer.serialize_bool(true),
            1 => serializer.serialize_i8(-1),
            2 => serializer.serialize_i16(-1),
            3 => serializer.serialize_i32(-1),
            4 => serializer.serialize_i64(-1),
            5 => serializer.serialize_i128(-1),
            6 => serializer.serialize_u8(1),
            7 => serializer.serialize_u16(1),
            8 => serializer.serialize_u32(1),
            9 => serializer.serialize_u64(1),
            10 => serializer.serialize_u128(1),
            11 => serializer.serialize_f32(1.0),
            12 => serializer.serialize_f64(1.0),
            13 => serializer.serialize_char('x'),
            14 => serializer.serialize_str("x"),
            15 => serializer.serialize_bytes(&[1, 2]),
            16 => serializer.serialize_none(),
            17 => serializer.serialize_some(&1_i32),
            18 => serializer.serialize_unit(),
            19 => serializer.serialize_unit_struct("Unit"),
            20 => serializer.serialize_unit_variant("Enum", 0, "Unit"),
            21 => serializer.serialize_newtype_struct("New", &1_i32),
            22 => serializer.serialize_newtype_variant("Enum", 0, "New", &1_i32),
            23 => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(&1_i32)?;
                seq.end()
            }
            24 => {
                let mut tuple = serializer.serialize_tuple(1)?;
                tuple.serialize_element(&1_i32)?;
                tuple.end()
            }
            25 => {
                let mut tuple = serializer.serialize_tuple_struct("Tuple", 1)?;
                tuple.serialize_field(&1_i32)?;
                tuple.end()
            }
            26 => {
                let mut tuple = serializer.serialize_tuple_variant("Enum", 0, "Tuple", 1)?;
                tuple.serialize_field(&1_i32)?;
                tuple.end()
            }
            27 => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("key", &1_i32)?;
                map.end()
            }
            28 => {
                let mut object = serializer.serialize_struct("Object", 1)?;
                object.serialize_field("key", &1_i32)?;
                object.end()
            }
            29 => {
                let mut object = serializer.serialize_struct_variant("Enum", 0, "Object", 1)?;
                object.serialize_field("key", &1_i32)?;
                object.end()
            }
            _ => serializer.collect_str(&DisplayProbe),
        }
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
struct DisplayProbe;

#[cfg(all(feature = "converter", feature = "json"))]
impl fmt::Display for DisplayProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("display")
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializes_scalar_variants() {
    use qubit_value::Value;

    assert_eq!(
        Value::from_serializable(&true)
            .expect("bool should serialize")
            .to_json_value()
            .expect("project JSON"),
        serde_json::json!(true),
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializer_preserves_float32_text() {
    use qubit_value::strict_json::to_value;
    use serde_json::Number;
    use serde_json::to_string;

    for bits in [
        0xC65B_9806_u32, // -14054.006
        0x4823_0AF3_u32, // 166955.8
        0x9CA9_7CE0_u32, // 0.000000000000000000000000000004358592
        0x4078_7ACD_u32, // 3.8824952
        0x2696_F5F4_u32, // 0.000000000000001047500658
    ] {
        let f32_value = f32::from_bits(bits);
        let value = to_value(&f32_value).expect("serialize float");
        let projected = to_string(&value).expect("serialize json");
        let legacy_text = serde_json::to_string(&serde_json::Value::Number(Number::from_f64(
            f64::from(f32_value),
        )))
        .expect("legacy serialize json");

        assert_eq!(
            projected,
            f32_value.to_string(),
            "strict json should preserve f32 display text",
        );
        assert_ne!(
            projected, legacy_text,
            "this sample should differ from f32->f64 cast path",
        );
    }
}

/// Exercises every scalar and compound entry point of the strict serializer.
#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_strict_json_serializer_covers_serde_entry_points() {
    use qubit_value::Value;

    for index in 0..=30 {
        let _ = Value::from_serializable(&ScalarProbe(index)).expect("probe should serialize");
    }
}
