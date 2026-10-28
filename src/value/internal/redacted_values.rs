// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Budget-aware redaction helpers for ordinary typed collections.

use std::fmt::Debug;

use qubit_redact::RedactionWriter;

/// Writes a typed collection while admitting and formatting each item under
/// the writer's active collection budget.
///
/// # Parameters
///
/// * `writer` - Structured writer carrying the active policy and budget.
/// * `variant` - Type name used to preserve the collection's debug shape.
/// * `values` - Collection payload whose items are written as unredacted values.
///
/// # Behavior
///
/// Items are formatted only after the writer admits them, so a rejected item
/// and all later items are not accessed or formatted.
pub(in crate::value) fn write_unredacted_values<T: Debug>(
    writer: &mut RedactionWriter<'_>,
    variant: &'static str,
    values: &[T],
) {
    writer.literal(variant);
    writer.literal("(");
    writer.sequence(|items| {
        items.for_each(values, |items, value| {
            items.unredacted_item(|| value);
        });
    });
    writer.literal(")");
}
