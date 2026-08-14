// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use std::fmt;
use std::fmt::Write as _;

use qubit_redact::MaskingPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactValue;
use qubit_redact::RedactedKeyedResult;
use qubit_redact::RedactedMapResult;
use qubit_redact::RedactedValue;
use qubit_redact::RedactionSession;
use qubit_redact::Sensitivity;

use super::Value;
use super::ValueRepr;
use crate::MultiValues;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::ValueContainer;
use crate::ValueRef;
use crate::multi_values::MultiValuesRef;
use crate::multi_values::MultiValuesRepr;

struct ByteCounter(usize);

impl fmt::Write for ByteCounter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0 = self.0.saturating_add(value.len());
        Ok(())
    }
}

fn display_len<T: fmt::Display + ?Sized>(value: &T) -> usize {
    let mut counter = ByteCounter(0);
    let _ = write!(&mut counter, "{value}");
    counter.0
}

fn debug_len<T: fmt::Debug + ?Sized>(value: &T) -> usize {
    let mut counter = ByteCounter(0);
    let _ = write!(&mut counter, "{value:?}");
    counter.0
}

#[cfg(feature = "json")]
struct JsonByteCounter(usize);

#[cfg(feature = "json")]
impl std::io::Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "json")]
fn json_len(value: &serde_json::Value) -> usize {
    let mut counter = JsonByteCounter(0);
    serde_json::to_writer(&mut counter, value).map_or(usize::MAX, |_| counter.0)
}

fn value_input_bytes(value: &Value) -> usize {
    match value.view() {
        ValueRef::Unset(_) => 1,
        ValueRef::Bool(value) => display_len(&value),
        ValueRef::Char(value) => display_len(&value),
        ValueRef::Int8(value) => display_len(&value),
        ValueRef::Int16(value) => display_len(&value),
        ValueRef::Int32(value) => display_len(&value),
        ValueRef::Int64(value) => display_len(&value),
        ValueRef::Int128(value) => display_len(&value),
        ValueRef::UInt8(value) => display_len(&value),
        ValueRef::UInt16(value) => display_len(&value),
        ValueRef::UInt32(value) => display_len(&value),
        ValueRef::UInt64(value) => display_len(&value),
        ValueRef::UInt128(value) => display_len(&value),
        ValueRef::Float32(value) => display_len(&value),
        ValueRef::Float64(value) => display_len(&value),
        #[cfg(feature = "big-integer")]
        ValueRef::BigInteger(value) => display_len(value),
        #[cfg(feature = "big-decimal")]
        ValueRef::BigDecimal(value) => display_len(value),
        ValueRef::String(value) => value.len(),
        #[cfg(feature = "chrono")]
        ValueRef::Date(value) => display_len(value),
        #[cfg(feature = "chrono")]
        ValueRef::Time(value) => display_len(value),
        #[cfg(feature = "chrono")]
        ValueRef::DateTime(value) => display_len(value),
        #[cfg(feature = "chrono")]
        ValueRef::Instant(value) => display_len(value),
        ValueRef::Duration(value) => debug_len(value),
        #[cfg(feature = "url")]
        ValueRef::Url(value) => value.as_str().len(),
        ValueRef::StringMap(values) => values.iter().fold(2, |total, (key, value)| {
            total
                .saturating_add(debug_len(key))
                .saturating_add(debug_len(value))
                .saturating_add(4)
        }),
        #[cfg(feature = "json")]
        ValueRef::Json(value) => json_len(value),
    }
}

fn multi_values_input_bytes(values: &MultiValues) -> usize {
    match values.view() {
        MultiValuesRef::Unset(_) => 1,
        MultiValuesRef::Bool(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Char(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Int8(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Int16(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Int32(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Int64(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Int128(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::UInt8(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::UInt16(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::UInt32(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::UInt64(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::UInt128(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Float32(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Float64(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "big-integer")]
        MultiValuesRef::BigInteger(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "big-decimal")]
        MultiValuesRef::BigDecimal(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::String(values) => values
            .iter()
            .map(debug_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "chrono")]
        MultiValuesRef::Date(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "chrono")]
        MultiValuesRef::Time(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "chrono")]
        MultiValuesRef::DateTime(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "chrono")]
        MultiValuesRef::Instant(values) => values
            .iter()
            .map(display_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::Duration(values) => values
            .iter()
            .map(debug_len)
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "url")]
        MultiValuesRef::Url(values) => values
            .iter()
            .map(|value| value.as_str().len())
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        MultiValuesRef::StringMap(values) => values
            .iter()
            .map(|map| {
                map.iter().fold(2usize, |total, (key, value)| {
                    total
                        .saturating_add(debug_len(key))
                        .saturating_add(debug_len(value))
                        .saturating_add(4)
                })
            })
            .sum::<usize>()
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
        #[cfg(feature = "json")]
        MultiValuesRef::Json(values) => values
            .iter()
            .fold(0usize, |total, value| total.saturating_add(json_len(value)))
            .saturating_add(2 + values.len().saturating_sub(1) * 2),
    }
}

impl RedactValue for Value {
    fn redaction_input_bytes(&self) -> usize {
        value_input_bytes(self)
    }

    /// Redacts string contents while replacing every other variant opaquely.
    fn redact_value<'a>(
        &'a self,
        level: Sensitivity,
        masking: &MaskingPolicy,
    ) -> RedactedValue<'a> {
        match &self.repr {
            ValueRepr::String(value) => value.redact_value(level, masking),
            _ => RedactedValue::opaque(level, masking),
        }
    }
}

impl RedactValue for MultiValues {
    fn redaction_input_bytes(&self) -> usize {
        multi_values_input_bytes(self)
    }

    /// Replaces a sensitive collection without formatting its contents.
    #[inline(always)]
    fn redact_value<'a>(
        &'a self,
        level: Sensitivity,
        masking: &MaskingPolicy,
    ) -> RedactedValue<'a> {
        RedactedValue::opaque(level, masking)
    }
}

impl Redact for Value {
    fn redaction_input_bytes(&self) -> usize {
        value_input_bytes(self)
    }

    /// Writes this value through `policy` without altering its ordinary debug
    /// representation when no key context is available.
    ///
    /// String maps classify every entry by its key. Other variants retain their
    /// ordinary debug representation until a structured redaction rule applies.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match &self.repr {
            ValueRepr::StringMap(values) => {
                fmt::Debug::fmt(&RedactedMapResult::new(values, session), formatter)
            }
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => {
                let redacted = session.json().redact_value(value);
                formatter.write_str(redacted.as_str())
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl Redact for MultiValues {
    fn redaction_input_bytes(&self) -> usize {
        multi_values_input_bytes(self)
    }

    /// Writes collection entries through the policy where their structure has
    /// key-bearing values; other typed collections retain normal debug output.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match &self.repr {
            MultiValuesRepr::StringMap(values) => {
                let mut output = formatter.debug_list();
                for value in values {
                    output.entry(&RedactedMapResult::new(value, session));
                }
                output.finish()
            }
            #[cfg(feature = "json")]
            MultiValuesRepr::Json(values) => {
                formatter.write_str("[")?;
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    let redacted = session.json().redact_value(value);
                    formatter.write_str(redacted.as_str())?;
                }
                formatter.write_str("]")
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl Redact for ValueContainer {
    fn redaction_input_bytes(&self) -> usize {
        match self {
            Self::Scalar(value) => value_input_bytes(value),
            Self::Collection(values) => multi_values_input_bytes(values),
        }
    }

    /// Delegates policy-aware rendering to the explicit scalar or collection.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Scalar(value) => value.fmt_redacted(session, formatter),
            Self::Collection(values) => values.fmt_redacted(session, formatter),
        }
    }
}

/// Formats a named value while applying its name as the policy lookup key.
fn fmt_named_value<T: Redact + RedactValue>(
    name: &str,
    value: &T,
    type_name: &str,
    value_name: &str,
    session: &mut RedactionSession<'_>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut output = formatter.debug_struct(type_name);
    output.field("name", &name);
    output.field(value_name, &RedactedKeyedResult::new(name, value, session));
    output.finish()
}

impl Redact for NamedValue {
    fn redaction_input_bytes(&self) -> usize {
        self.name()
            .len()
            .saturating_add(value_input_bytes(self.value()))
    }

    /// Uses the wrapper name to determine whether its complete value is masked.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fmt_named_value(
            self.name(),
            self.value(),
            "NamedValue",
            "value",
            session,
            formatter,
        )
    }
}

impl Redact for NamedMultiValues {
    fn redaction_input_bytes(&self) -> usize {
        self.name()
            .len()
            .saturating_add(multi_values_input_bytes(self.values()))
    }

    /// Uses the wrapper name to determine whether its complete collection is
    /// masked.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        fmt_named_value(
            self.name(),
            self.values(),
            "NamedMultiValues",
            "value",
            session,
            formatter,
        )
    }
}
