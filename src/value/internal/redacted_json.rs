// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Recursive JSON formatting with policy-aware key redaction.

use std::fmt;

use qubit_redact::RedactionPolicy;
use serde_json::Value as JsonValue;

/// A JSON value rendered with key-aware recursive redaction.
#[must_use]
pub(in crate::value) struct RedactedJson<'a> {
    /// Original JSON value borrowed without cloning.
    value: &'a JsonValue,
    /// Policy used to classify every object key.
    policy: &'a RedactionPolicy,
}

impl<'a> RedactedJson<'a> {
    /// Creates a recursive redaction formatter for one JSON value.
    ///
    /// # Parameters
    ///
    /// * `value` - JSON value to render without cloning.
    /// * `policy` - Policy used to classify object keys.
    ///
    /// # Returns
    ///
    /// A formatter borrowing `value` and `policy`.
    #[inline(always)]
    pub(in crate::value) const fn new(
        value: &'a JsonValue,
        policy: &'a RedactionPolicy,
    ) -> Self {
        Self { value, policy }
    }
}

impl fmt::Debug for RedactedJson<'_> {
    /// Formats JSON objects and arrays while masking policy-selected children.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            JsonValue::Array(values) => {
                let mut output = formatter.debug_list();
                for value in values {
                    output.entry(&Self::new(value, self.policy));
                }
                output.finish()
            }
            JsonValue::Object(values) => {
                let mut output = formatter.debug_map();
                for (key, value) in values {
                    if let Some(sensitivity) = self.policy.sensitivity_for(key)
                    {
                        match value {
                            JsonValue::String(text) => {
                                output.entry(
                                    &key,
                                    &self
                                        .policy
                                        .masking()
                                        .mask(sensitivity, text),
                                );
                            }
                            _ => {
                                output.entry(&key, &"<redacted>");
                            }
                        };
                    } else {
                        output.entry(&key, &Self::new(value, self.policy));
                    }
                }
                output.finish()
            }
            value => fmt::Debug::fmt(value, formatter),
        }
    }
}
