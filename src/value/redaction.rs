// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use std::fmt;

use qubit_redact::{
    Redact,
    RedactionPolicy,
};
#[cfg(feature = "json")]
use serde_json::Value as JsonValue;

use super::Value;

impl Redact for Value {
    /// Writes this value through `policy` without altering its ordinary debug
    /// representation when no key context is available.
    ///
    /// String maps classify every entry by its key. Other variants retain their
    /// ordinary debug representation until a structured redaction rule applies.
    fn fmt_redacted(
        &self,
        policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::StringMap(values) => {
                let mut output = formatter.debug_map();
                for (key, value) in values {
                    if let Some(sensitivity) = policy.sensitivity_for(key) {
                        output.entry(
                            &key,
                            &policy.masking().mask(sensitivity, value),
                        );
                    } else {
                        output.entry(&key, value);
                    }
                }
                output.finish()
            }
            #[cfg(feature = "json")]
            Self::Json(value) => {
                fmt::Debug::fmt(&RedactedJson { value, policy }, formatter)
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

/// A JSON value rendered with key-aware recursive redaction.
#[cfg(feature = "json")]
struct RedactedJson<'a> {
    /// Original JSON value borrowed without cloning.
    value: &'a JsonValue,
    /// Policy used to classify every object key.
    policy: &'a RedactionPolicy,
}

#[cfg(feature = "json")]
impl fmt::Debug for RedactedJson<'_> {
    /// Formats JSON objects and arrays while masking children selected by the
    /// shared redaction policy.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            JsonValue::Array(values) => {
                let mut output = formatter.debug_list();
                for value in values {
                    output.entry(&Self {
                        value,
                        policy: self.policy,
                    });
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
                        output.entry(
                            &key,
                            &Self {
                                value,
                                policy: self.policy,
                            },
                        );
                    }
                }
                output.finish()
            }
            value => fmt::Debug::fmt(value, formatter),
        }
    }
}
