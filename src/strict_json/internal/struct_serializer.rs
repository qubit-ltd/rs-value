// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer state for ordinary structs and `serde_json` number tokens.

use serde::Serialize;
use serde::ser::SerializeStruct;
use serde_json::Value;

use crate::strict_json::{Result, StrictJsonError};

use super::{ObjectSerializer, json_number_serializer};

/// Collects either an ordinary JSON object or one arbitrary-precision number.
pub(in crate::strict_json) enum StructSerializer {
    /// Ordinary struct fields collected as a JSON object.
    Object(ObjectSerializer),
    /// The single number value emitted by `serde_json`'s private token.
    Number(Option<Value>),
}

impl SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes one struct field according to the active struct kind.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Self::Object(serializer) => serializer.serialize_field(key, value),
            Self::Number(output) => {
                if key != json_number_serializer::NUMBER_TOKEN || output.is_some() {
                    return Err(StrictJsonError::Serialization);
                }
                *output = Some(json_number_serializer::serialize_number(value)?);
                Ok(())
            }
        }
    }

    /// Completes the ordinary object or arbitrary-precision number.
    fn end(self) -> Result<Value> {
        match self {
            Self::Object(serializer) => serializer.end(),
            Self::Number(Some(value)) => Ok(value),
            Self::Number(None) => Err(StrictJsonError::Serialization),
        }
    }
}
