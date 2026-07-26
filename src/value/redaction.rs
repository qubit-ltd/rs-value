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
use crate::{
    MultiValues,
    NamedMultiValues,
    NamedValue,
    ValueContainer,
};

/// Formats a string map with sensitivity determined by each map key.
struct RedactedStringMap<'a> {
    /// Map whose entries are rendered through the policy.
    values: &'a std::collections::HashMap<String, String>,
    /// Policy that classifies map keys.
    policy: &'a RedactionPolicy,
}

impl fmt::Debug for RedactedStringMap<'_> {
    /// Writes each map entry while masking values for sensitive keys.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = formatter.debug_map();
        for (key, value) in self.values {
            if let Some(sensitivity) = self.policy.sensitivity_for(key) {
                output.entry(
                    &key,
                    &self.policy.masking().mask(sensitivity, value),
                );
            } else {
                output.entry(&key, value);
            }
        }
        output.finish()
    }
}

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
            Self::StringMap(values) => fmt::Debug::fmt(
                &RedactedStringMap { values, policy },
                formatter,
            ),
            #[cfg(feature = "json")]
            Self::Json(value) => {
                fmt::Debug::fmt(&RedactedJson::new(value, policy), formatter)
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl Redact for MultiValues {
    /// Writes collection entries through the policy where their structure has
    /// key-bearing values; other typed collections retain normal debug output.
    fn fmt_redacted(
        &self,
        policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::StringMap(values) => {
                let mut output = formatter.debug_list();
                for value in values {
                    output.entry(&RedactedStringMap {
                        values: value,
                        policy,
                    });
                }
                output.finish()
            }
            #[cfg(feature = "json")]
            Self::Json(values) => {
                let mut output = formatter.debug_list();
                for value in values {
                    output.entry(&RedactedJson::new(value, policy));
                }
                output.finish()
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl Redact for ValueContainer {
    /// Delegates policy-aware rendering to the explicit scalar or collection.
    fn fmt_redacted(
        &self,
        policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Scalar(value) => value.fmt_redacted(policy, formatter),
            Self::Collection(values) => values.fmt_redacted(policy, formatter),
        }
    }
}

/// Formats a named value while applying its name as the policy lookup key.
fn fmt_named_value<T: Redact + fmt::Debug>(
    name: &str,
    value: &T,
    type_name: &str,
    value_name: &str,
    policy: &RedactionPolicy,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut output = formatter.debug_struct(type_name);
    output.field("name", &name);
    if let Some(sensitivity) = policy.sensitivity_for(name) {
        let value = format!("{value:?}");
        output.field(value_name, &policy.masking().mask(sensitivity, &value));
    } else {
        output.field(value_name, &value.redacted_with(policy));
    }
    output.finish()
}

impl Redact for NamedValue {
    /// Uses the wrapper name to determine whether its complete value is masked.
    fn fmt_redacted(
        &self,
        policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fmt_named_value(
            self.name(),
            self.value(),
            "NamedValue",
            "value",
            policy,
            formatter,
        )
    }
}

impl Redact for NamedMultiValues {
    /// Uses the wrapper name to determine whether its complete collection is
    /// masked.
    fn fmt_redacted(
        &self,
        policy: &RedactionPolicy,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fmt_named_value(
            self.name(),
            self.values(),
            "NamedMultiValues",
            "value",
            policy,
            formatter,
        )
    }
}
