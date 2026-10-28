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

use super::Value;
use super::ValueRepr;
use super::internal::RedactedStringMap;
use super::internal::write_unredacted_values;
use crate::MultiValues;
use crate::MultiValuesRef;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::ValueContainer;

macro_rules! redact_collection_payload {
    (String, $values:expr, $writer:expr) => {
        $writer.sequence(|items| {
            items.for_each($values, |items, value| {
                items.unredacted_item(|| value);
            });
        })
    };
    (StringMap, $values:expr, $writer:expr) => {
        $writer.sequence(|items| {
            items.for_each($values, |items, values| {
                items.nested_item(&RedactedStringMap { values });
            });
        })
    };
    (Json, $values:expr, $writer:expr) => {
        $writer.sequence(|items| {
            items.for_each($values, |items, value| {
                items.json_value_item(value);
            });
        })
    };
    ($variant:ident, $values:expr, $writer:expr) => {
        write_unredacted_values($writer, stringify!($variant), $values)
    };
}

macro_rules! redact_multi_values_match {
    ($values:expr, $writer:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $values {
            MultiValuesRef::Unset(_) => unreachable!("unset values are handled before dispatch"),
            $($(#[$cfg])* MultiValuesRef::$variant(values) => {
                redact_collection_payload!($variant, values, $writer)
            },)+
        }
    };
}

impl Redact for Value {
    /// Writes one value through the shared structured writer.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        match &self.repr {
            ValueRepr::StringMap(values) => {
                writer.record("Value", |fields| {
                    fields.map("StringMap", values);
                });
            }
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => {
                writer.record("Value", |fields| {
                    fields.json_value("Json", value);
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
        let values = self.view();
        if matches!(values, MultiValuesRef::Unset(_)) {
            writer.unredacted(self);
            return;
        }
        for_each_value_type!(redact_multi_values_match, values, writer);
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
            fields.keyed_nested("value", self.name(), self.value());
        });
    }
}

impl Redact for NamedMultiValues {
    /// Writes the name and policy-selected collection through one session.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("NamedMultiValues", |fields| {
            fields.unredacted("name", || self.name());
            fields.keyed_nested("value", self.name(), self.values());
        });
    }
}
