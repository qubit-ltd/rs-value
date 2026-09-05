// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Borrowed map adapter for nested collection redaction.

use std::collections::HashMap;

use qubit_redact::Redact;
use qubit_redact::RedactionWriter;

/// Borrows one map so nested redaction shares its parent's policy and budget.
pub(in crate::value) struct RedactedStringMap<'a> {
    /// Entries classified by their business keys without cloning storage.
    pub(in crate::value) values: &'a HashMap<String, String>,
}

impl Redact for RedactedStringMap<'_> {
    /// Writes borrowed entries through the shared key-aware map scope.
    fn write_redacted(&self, writer: &mut RedactionWriter<'_>) {
        writer.record("Value", |fields| {
            fields.map("StringMap", self.values);
        });
    }
}
