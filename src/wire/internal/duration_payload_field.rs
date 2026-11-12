// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field identifiers for the duration payload record.

use serde::Deserialize;
use serde::Deserializer;

/// Identifies the fixed fields of the duration payload record.
#[derive(Debug)]
pub(super) enum DurationPayloadField {
    /// The `secs` field.
    Secs,
    /// The `nanos` field.
    Nanos,
}

impl<'de> Deserialize<'de> for DurationPayloadField {
    /// Deserializes one field name without accepting non-string identifiers.
    ///
    /// # Type Parameters
    ///
    /// - `D`: The deserializer supplying the field name.
    ///
    /// # Parameters
    ///
    /// - `deserializer`: The input deserializer.
    ///
    /// # Returns
    ///
    /// The matching fixed field identifier.
    ///
    /// # Errors
    ///
    /// Returns an error for unknown names and key representations that are
    /// not strings or UTF-8 byte strings.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(
            super::duration_payload_field_visitor::DurationPayloadFieldVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::DurationPayloadField;
    use crate::tests::allocation_support::assert_escaped_field_is_recognized;
    use crate::tests::allocation_support::assert_key_forms_match_string;
    use crate::tests::allocation_support::assert_string_forms_allocate_nothing;

    #[test]
    fn test_duration_fields_classify_without_allocating() {
        assert_string_forms_allocate_nothing::<DurationPayloadField>("secs");
        assert_string_forms_allocate_nothing::<DurationPayloadField>("nanos");
    }

    #[test]
    fn test_duration_field_key_forms_match_string() {
        assert_key_forms_match_string::<DurationPayloadField>("secs");
    }

    #[test]
    fn test_duration_escaped_fields_are_recognized() {
        assert_escaped_field_is_recognized::<DurationPayloadField>(r#""s\u0065cs""#);
        assert_escaped_field_is_recognized::<DurationPayloadField>(r#""n\u0061nos""#);
    }
}
