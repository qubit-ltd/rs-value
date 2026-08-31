// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable arbitrary-precision decimal payload.

use std::str::FromStr;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use serde::Deserialize;
use serde::Serialize;

use crate::wire::is_valid_big_decimal_scale;

/// Exact wire representation of an arbitrary-precision decimal.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::wire) struct BigDecimalPayload {
    /// Canonical base-10 integer coefficient.
    pub(in crate::wire) coefficient: String,
    /// Number of decimal places represented by the coefficient.
    pub(in crate::wire) scale: i64,
}

impl TryFrom<&BigDecimal> for BigDecimalPayload {
    type Error = &'static str;

    /// Creates an exact payload without formatting the decimal value.
    #[inline]
    fn try_from(value: &BigDecimal) -> Result<Self, Self::Error> {
        let (coefficient, scale) = value.as_bigint_and_exponent();
        if !is_valid_big_decimal_scale(scale) {
            return Err("decimal scale exceeds the V1 maximum absolute scale");
        }
        Ok(Self {
            coefficient: coefficient.to_string(),
            scale,
        })
    }
}

impl TryFrom<BigDecimalPayload> for BigDecimal {
    type Error = &'static str;

    /// Restores a decimal after validating the canonical coefficient.
    fn try_from(value: BigDecimalPayload) -> Result<Self, Self::Error> {
        if !is_valid_big_decimal_scale(value.scale) {
            return Err("decimal scale exceeds the V1 maximum absolute scale");
        }
        let coefficient = BigInt::from_str(&value.coefficient).map_err(|_| "invalid decimal coefficient")?;
        if coefficient.to_string() != value.coefficient {
            return Err("non-canonical decimal coefficient");
        }
        Ok(Self::new(coefficient, value.scale))
    }
}
