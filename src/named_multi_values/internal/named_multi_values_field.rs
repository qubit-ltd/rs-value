// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field identifiers for the named multi-values record.

use serde::Deserialize;
use serde::Deserializer;

/// Identifies the fixed fields of the named multi-values record.
#[derive(Debug)]
pub(super) enum NamedMultiValuesField {
    /// The `name` field.
    Name,
    /// The `value` field.
    Value,
}

impl<'de> Deserialize<'de> for NamedMultiValuesField {
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
        deserializer.deserialize_identifier(super::named_multi_values_field_visitor::NamedMultiValuesFieldVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::NamedMultiValuesField;
    use crate::tests::allocation_support::assert_escaped_field_is_recognized;
    use crate::tests::allocation_support::assert_key_forms_match_string;
    use crate::tests::allocation_support::assert_string_forms_allocate_nothing;

    #[test]
    fn test_named_multi_values_fields_classify_without_allocating() {
        assert_string_forms_allocate_nothing::<NamedMultiValuesField>("name");
        assert_string_forms_allocate_nothing::<NamedMultiValuesField>("value");
    }

    #[test]
    fn test_named_multi_values_field_key_forms_match_string() {
        assert_key_forms_match_string::<NamedMultiValuesField>("name");
    }

    #[test]
    fn test_named_multi_values_escaped_fields_are_recognized() {
        assert_escaped_field_is_recognized::<NamedMultiValuesField>(r#""n\u0061me""#);
        assert_escaped_field_is_recognized::<NamedMultiValuesField>(r#""\u0076alue""#);
    }
}
