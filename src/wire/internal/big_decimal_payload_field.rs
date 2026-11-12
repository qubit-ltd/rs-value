// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field identifiers for the big-decimal payload record.

use serde::Deserialize;
use serde::Deserializer;

/// Identifies the fixed fields of the big-decimal payload record.
#[derive(Debug)]
pub(super) enum BigDecimalPayloadField {
    /// The `coefficient` field.
    Coefficient,
    /// The `scale` field.
    Scale,
}

impl<'de> Deserialize<'de> for BigDecimalPayloadField {
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
            super::big_decimal_payload_field_visitor::BigDecimalPayloadFieldVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::BigDecimalPayloadField;
    use crate::tests::allocation_support::assert_escaped_field_is_recognized;
    use crate::tests::allocation_support::assert_key_forms_match_string;
    use crate::tests::allocation_support::assert_string_forms_allocate_nothing;

    #[test]
    fn test_big_decimal_fields_classify_without_allocating() {
        assert_string_forms_allocate_nothing::<BigDecimalPayloadField>("coefficient");
        assert_string_forms_allocate_nothing::<BigDecimalPayloadField>("scale");
    }

    #[test]
    fn test_big_decimal_field_key_forms_match_string() {
        assert_key_forms_match_string::<BigDecimalPayloadField>("coefficient");
    }

    #[test]
    fn test_big_decimal_escaped_fields_are_recognized() {
        assert_escaped_field_is_recognized::<BigDecimalPayloadField>(r#""co\u0065fficient""#);
        assert_escaped_field_is_recognized::<BigDecimalPayloadField>(r#""sc\u0061le""#);
    }
}
