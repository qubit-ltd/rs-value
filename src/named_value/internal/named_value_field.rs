// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field identifiers for the named value record.

use serde::Deserialize;
use serde::Deserializer;

/// Identifies the fixed fields of the named value record.
#[derive(Debug)]
pub(super) enum NamedValueField {
    /// The `name` field.
    Name,
    /// The `value` field.
    Value,
}

impl<'de> Deserialize<'de> for NamedValueField {
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
        deserializer
            .deserialize_identifier(super::named_value_field_visitor::NamedValueFieldVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::NamedValueField;
    use crate::tests::allocation_support::assert_escaped_field_is_recognized;
    use crate::tests::allocation_support::assert_key_forms_match_string;
    use crate::tests::allocation_support::assert_string_forms_allocate_nothing;

    #[test]
    fn test_named_value_fields_classify_without_allocating() {
        assert_string_forms_allocate_nothing::<NamedValueField>("name");
        assert_string_forms_allocate_nothing::<NamedValueField>("value");
    }

    #[test]
    fn test_named_value_field_key_forms_match_string() {
        assert_key_forms_match_string::<NamedValueField>("name");
    }

    #[test]
    fn test_named_value_escaped_fields_are_recognized() {
        assert_escaped_field_is_recognized::<NamedValueField>(r#""n\u0061me""#);
        assert_escaped_field_is_recognized::<NamedValueField>(r#""\u0076alue""#);
    }
}
