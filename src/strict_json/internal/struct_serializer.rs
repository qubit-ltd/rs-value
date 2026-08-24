// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer state for ordinary structs.

use serde::Serialize;
use serde::ser::SerializeStruct;
use serde_json::Value;

use super::ObjectSerializer;
use crate::strict_json::Result;
use crate::strict_json::StrictJsonError;

/// Collects ordinary struct fields as a JSON object.
pub(in crate::strict_json) struct StructSerializer(pub(super) ObjectSerializer);

impl SerializeStruct for StructSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes one struct field according to the active struct kind.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.0.serialize_field(key, value)
    }

    /// Completes the ordinary object.
    fn end(self) -> Result<Value> {
        self.0.end()
    }
}
