// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable duration payload used by scalar and collection adapters.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Stable wire representation of a duration.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::wire) struct DurationPayload {
    /// Whole seconds.
    pub(in crate::wire) secs: u64,
    /// Fractional nanoseconds, always less than one second.
    pub(in crate::wire) nanos: u32,
}

impl From<&Duration> for DurationPayload {
    /// Creates a stable payload from a runtime duration.
    #[inline]
    fn from(value: &Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl TryFrom<DurationPayload> for Duration {
    type Error = &'static str;

    /// Restores a duration after validating its nanosecond component.
    fn try_from(value: DurationPayload) -> Result<Self, Self::Error> {
        if value.nanos >= 1_000_000_000 {
            return Err("duration nanoseconds must be less than 1000000000");
        }
        Ok(Self::new(value.secs, value.nanos))
    }
}
