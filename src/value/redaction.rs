// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;
use qubit_redact::Sensitivity;

use super::Value;
use super::ValueRepr;
use crate::MultiValues;
use crate::MultiValuesRef;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::ValueContainer;

impl Redact for Value {
    /// Writes one value through the shared structured writer.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        match &self.repr {
            ValueRepr::StringMap(values) => {
                writer.record("Value", |fields| {
                    fields.map("StringMap", values.iter());
                });
            }
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => {
                writer.record("Value", |fields| {
                    fields.json("Json", &value.to_string());
                });
            }
            _ => {
                writer.unredacted(self);
            }
        }
    }
}

impl Redact for MultiValues {
    /// Writes a typed collection through the shared structured writer.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        match self.view() {
            MultiValuesRef::String(values) => {
                writer.sequence(|items| {
                    for value in values {
                        items.unredacted_item(|| value);
                    }
                });
            }
            _ => {
                writer.unredacted(self);
            }
        }
    }
}

impl Redact for ValueContainer {
    /// Writes the selected container payload through the shared session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        match self {
            Self::Scalar(value) => {
                writer.tuple("ValueContainer", |fields| {
                    fields.nested("Scalar", value);
                });
            }
            Self::Collection(values) => {
                writer.tuple("ValueContainer", |fields| {
                    fields.nested("Collection", values);
                });
            }
        }
    }
}

impl Redact for NamedValue {
    /// Writes the name and policy-selected value through one session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("NamedValue", |fields| {
            fields.unredacted("name", || self.name());
            fields.sensitive(Sensitivity::Low, "value", || self.value());
        });
    }
}

impl Redact for NamedMultiValues {
    /// Writes the name and policy-selected collection through one session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("NamedMultiValues", |fields| {
            fields.unredacted("name", || self.name());
            fields.sensitive(Sensitivity::Low, "value", || self.values());
        });
    }
}
