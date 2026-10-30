// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable duration payload used by scalar and collection adapters.

use std::time::Duration;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use super::duration_payload_visitor::DurationPayloadVisitor;

/// Encodes a [`Duration`] as whole seconds and a fractional nanosecond count.
///
/// The nanosecond component is less than one billion, matching the invariant
/// required by [`Duration::new`].
#[derive(Serialize)]
pub(in crate::wire) struct DurationPayload {
    /// Number of complete seconds in the represented duration.
    pub(in crate::wire) secs: u64,
    /// Fractional nanoseconds within the final second, from zero to
    /// 999,999,999.
    pub(in crate::wire) nanos: u32,
}

impl<'de> Deserialize<'de> for DurationPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(DurationPayloadVisitor)
        } else {
            deserializer.deserialize_struct("DurationPayload", &["secs", "nanos"], DurationPayloadVisitor)
        }
    }
}

impl From<&Duration> for DurationPayload {
    /// Splits a runtime duration into its seconds and fractional nanoseconds.
    ///
    /// # Parameters
    ///
    /// - `value`: The duration whose components are encoded.
    ///
    /// # Returns
    ///
    /// A payload with the seconds and subsecond nanoseconds from `value`.
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
    ///
    /// # Parameters
    ///
    /// - `value`: The serialized seconds and fractional nanoseconds.
    ///
    /// # Returns
    ///
    /// The duration represented by `value` when its nanosecond component is
    /// valid.
    ///
    /// # Errors
    ///
    /// Returns a static error message when `value.nanos` is at least one
    /// billion.
    fn try_from(value: DurationPayload) -> Result<Self, Self::Error> {
        if value.nanos >= 1_000_000_000 {
            return Err("duration nanoseconds must be less than 1000000000");
        }
        Ok(Self::new(value.secs, value.nanos))
    }
}
