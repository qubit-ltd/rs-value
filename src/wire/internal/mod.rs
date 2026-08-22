// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal payload types for canonical wire adapters.

#[cfg(feature = "big-decimal")]
mod big_decimal_payload;
#[cfg(feature = "json")]
mod canonical_json;
mod canonical_string_map;
mod duration_payload;
mod strict_string_map;

#[cfg(feature = "big-decimal")]
pub(in crate::wire) use big_decimal_payload::BigDecimalPayload;
#[cfg(feature = "json")]
pub(in crate::wire) use canonical_json::CanonicalJson;
pub(in crate::wire) use canonical_string_map::CanonicalStringMap;
pub(in crate::wire) use duration_payload::DurationPayload;
pub(in crate::wire) use strict_string_map::StrictStringMap;
