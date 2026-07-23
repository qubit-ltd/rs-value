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

use super::Value;
#[cfg(feature = "json")]
use super::internal::RedactedJson;

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
                fmt::Debug::fmt(&RedactedJson::new(value, policy), formatter)
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}
