// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use qubit_redact::MaskingPolicy;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_redact::domain::Redact;
use qubit_redact::domain::RedactValue;
use qubit_redact::domain::RedactedValue;
use qubit_redact::domain::RedactionWriter;

use super::Value;
use super::ValueRepr;
use crate::MultiValues;
use crate::MultiValuesRef;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::ValueContainer;

impl RedactValue for Value {
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
    /// Writes one value through the shared structured writer.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_, '_>) {
        match &self.repr {
            ValueRepr::StringMap(values) => {
                writer
                    .render(|writer| writer.redacted_map(values));
            }
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => {
                writer.render(|writer| {
                    Redactor::new(writer.policy().clone())
                        .json()
                        .redact_value(value)
                });
            }
            _ => writer.render(|_| self),
        }
    }
}

impl Redact for MultiValues {
    /// Writes a typed collection through the shared structured writer.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_, '_>) {
        macro_rules! write_items {
            ($variant:literal, $values:expr) => {{
                writer.tuple($variant, |fields| {
                    for value in $values {
                        let _ = fields.item(|_| value);
                    }
                });
            }};
        }

        match self.view() {
            MultiValuesRef::Unset(data_type) => {
                writer.tuple("Unset", |fields| {
                    let _ = fields.item(|_| data_type);
                });
            }
            MultiValuesRef::Bool(values) => write_items!("Bool", values),
            MultiValuesRef::Char(values) => write_items!("Char", values),
            MultiValuesRef::Int8(values) => write_items!("Int8", values),
            MultiValuesRef::Int16(values) => write_items!("Int16", values),
            MultiValuesRef::Int32(values) => write_items!("Int32", values),
            MultiValuesRef::Int64(values) => write_items!("Int64", values),
            MultiValuesRef::Int128(values) => write_items!("Int128", values),
            MultiValuesRef::UInt8(values) => write_items!("UInt8", values),
            MultiValuesRef::UInt16(values) => write_items!("UInt16", values),
            MultiValuesRef::UInt32(values) => write_items!("UInt32", values),
            MultiValuesRef::UInt64(values) => write_items!("UInt64", values),
            MultiValuesRef::UInt128(values) => write_items!("UInt128", values),
            MultiValuesRef::Float32(values) => write_items!("Float32", values),
            MultiValuesRef::Float64(values) => write_items!("Float64", values),
            #[cfg(feature = "big-integer")]
            MultiValuesRef::BigInteger(values) => {
                write_items!("BigInteger", values)
            }
            #[cfg(feature = "big-decimal")]
            MultiValuesRef::BigDecimal(values) => {
                write_items!("BigDecimal", values)
            }
            MultiValuesRef::String(values) => write_items!("String", values),
            #[cfg(feature = "chrono")]
            MultiValuesRef::Date(values) => write_items!("Date", values),
            #[cfg(feature = "chrono")]
            MultiValuesRef::Time(values) => write_items!("Time", values),
            #[cfg(feature = "chrono")]
            MultiValuesRef::DateTime(values) => {
                write_items!("DateTime", values)
            }
            #[cfg(feature = "chrono")]
            MultiValuesRef::Instant(values) => write_items!("Instant", values),
            MultiValuesRef::Duration(values) => {
                write_items!("Duration", values)
            }
            #[cfg(feature = "url")]
            MultiValuesRef::Url(values) => write_items!("Url", values),
            MultiValuesRef::StringMap(values) => {
                write_items!("StringMap", values)
            }
            #[cfg(feature = "json")]
            MultiValuesRef::Json(values) => {
                writer.tuple("Json", |fields| {
                    fields.list(|items| {
                        for value in values {
                            let _ = items.item_text(|session| {
                                Redactor::new(session.policy().clone())
                                    .json()
                                    .redact_value(value)
                                    .as_str()
                                    .to_owned()
                            });
                        }
                    });
                });
            }
        }
    }
}

impl Redact for ValueContainer {
    /// Writes the selected container payload through the shared session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_, '_>) {
        match self {
            Self::Scalar(value) => {
                writer.render(|writer| writer.redacted(value));
            }
            Self::Collection(values) => {
                writer.render(|writer| writer.redacted(values));
            }
        }
    }
}

impl Redact for NamedValue {
    /// Writes the name and policy-selected value through one session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_, '_>) {
        writer.record("NamedValue", |fields| {
            let _ = fields.field("name", || self.name());
            let _ = fields.value("value", |writer| {
                writer.redacted_keyed(self.name(), self.value())
            });
        });
    }
}

impl Redact for NamedMultiValues {
    /// Writes the name and policy-selected collection through one session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_, '_>) {
        writer.record("NamedMultiValues", |fields| {
            let _ = fields.field("name", || self.name());
            let _ = fields.value("value", |writer| {
                writer.redacted_keyed(self.name(), self.values())
            });
        });
    }
}
