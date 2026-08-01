// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use std::fmt;

#[cfg(feature = "json")]
use qubit_redact::RedactedJson;
use qubit_redact::{
    Redact, RedactMapValue, RedactValue, RedactedKeyedValue, RedactedMap, RedactedValue,
    RedactionPolicy,
};

use super::Value;
use crate::{MultiValues, NamedMultiValues, NamedValue, ValueContainer};

impl RedactValue for Value {
    /// Redacts string contents while replacing every other variant opaquely.
    fn redact_value<'a>(
        &'a self,
        level: qubit_redact::Sensitivity,
        masking: &'a qubit_redact::MaskingPolicy,
    ) -> RedactedValue<'a> {
        match self {
            Self::String(value) => value.redact_value(level, masking),
            _ => RedactedValue::opaque(level, masking),
        }
    }
}

impl RedactValue for MultiValues {
    /// Replaces a sensitive collection without formatting its contents.
    #[inline(always)]
    fn redact_value<'a>(
        &'a self,
        level: qubit_redact::Sensitivity,
        masking: &'a qubit_redact::MaskingPolicy,
    ) -> RedactedValue<'a> {
        RedactedValue::opaque(level, masking)
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
            Self::StringMap(values) => values.fmt_redacted_map(policy, formatter),
            #[cfg(feature = "json")]
            Self::Json(value) => fmt::Debug::fmt(&RedactedJson::new(value, policy), formatter),
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
                    output.entry(&RedactedMap::new(value, policy.clone()));
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
fn fmt_named_value<T: Redact + RedactValue>(
    name: &str,
    value: &T,
    type_name: &str,
    value_name: &str,
    policy: &RedactionPolicy,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut output = formatter.debug_struct(type_name);
    output.field("name", &name);
    output.field(value_name, &RedactedKeyedValue::new(name, value, policy));
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
