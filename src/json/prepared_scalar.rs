// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Admitted scalar projection and borrowed payload materialization.

use serde_json::Map;
use serde_json::Number;
use serde_json::Value as JsonValue;

use crate::ValueRef;

/// One admitted scalar; formatting results are moved into the output.
pub(super) enum PreparedScalar<'a> {
    /// Primitive or borrowed storage requiring no formatting cache.
    Borrowed(ValueRef<'a>),
    /// Text already rendered within the projection budget.
    Formatted(String),
    /// Floating-point number prepared with the required natural precision.
    Number(Number),
}

/// Materializes only the families admitted as borrowing their original storage.
macro_rules! borrowed_payload {
    (String, $class:ident, $value:expr) => {
        JsonValue::String($value.to_owned())
    };
    ($variant:ident, json_bool, $value:expr) => {
        JsonValue::Bool($value)
    };
    ($variant:ident, json_number, $value:expr) => {
        JsonValue::from($value)
    };
    ($variant:ident, json_identity, $value:expr) => {
        crate::wire::json::canonicalize_json_value($value)
    };
    ($variant:ident, json_object, $value:expr) => {{
        let mut entries: Vec<_> = $value.iter().collect();
        entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        let mut object = Map::with_capacity(entries.len());
        for (key, value) in entries {
            object.insert(key.clone(), JsonValue::String(value.clone()));
        }
        JsonValue::Object(object)
    }};
    ($variant:ident, $class:ident, $value:expr) => {{
        let _ = $value;
        unreachable!("formatted payloads must use their prepared rendering")
    }};
}

/// Uses the closed type table for borrowed scalar materialization.
macro_rules! borrowed_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $value {
            ValueRef::Unset(_) => JsonValue::Null,
            $($(#[$cfg])* ValueRef::$variant(value) => borrowed_payload!($variant, $json_class, value),)+
        }
    };
}

impl PreparedScalar<'_> {
    /// Produces the final JSON scalar; cached text and numbers are moved, not
    /// re-rendered.
    pub(super) fn materialize(self) -> JsonValue {
        match self {
            Self::Borrowed(value) => for_each_value_type!(borrowed_match, value),
            Self::Formatted(text) => JsonValue::String(text),
            Self::Number(number) => JsonValue::Number(number),
        }
    }
}
