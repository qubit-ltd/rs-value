// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Adapter for `serde_json` arbitrary-precision number tokens.

use std::str::FromStr;

use serde::Serialize;
use serde_json::{Number, Value};

use crate::strict_json::{Result, StrictJsonError};

/// Private struct name emitted by `serde_json` with `arbitrary_precision`.
pub(in crate::strict_json) const NUMBER_TOKEN: &str = "$serde_json::private::Number";

/// Converts the string field of a `serde_json` number token into a JSON number.
///
/// # Errors
///
/// Returns [`StrictJsonError::Serialization`] when the token field cannot be
/// serialized as a string or when the string is not a valid JSON number.
pub(in crate::strict_json) fn serialize_number<T>(value: &T) -> Result<Value>
where
    T: ?Sized + Serialize,
{
    let value = value
        .serialize(serde_json::value::Serializer)
        .map_err(|_| StrictJsonError::Serialization)?;
    let Value::String(value) = value else {
        return Err(StrictJsonError::Serialization);
    };
    Number::from_str(&value)
        .map(Value::Number)
        .map_err(|_| StrictJsonError::Serialization)
}
