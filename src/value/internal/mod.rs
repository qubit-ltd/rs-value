// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal implementation details for scalar values.

#[cfg(feature = "redact")]
mod redacted_string_map;
#[cfg(feature = "redact")]
mod redacted_values;
mod value_repr;

#[cfg(feature = "redact")]
pub(in crate::value) use redacted_string_map::RedactedStringMap;
#[cfg(feature = "redact")]
pub(in crate::value) use redacted_values::write_unredacted_values;

pub(crate) use self::value_repr::ValueRepr;
