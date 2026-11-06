// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed traversal state for JSON projection admission.

pub(super) mod json_children;
mod measurement_writer;
pub(super) mod prepared_projection;
pub(super) mod prepared_scalar;

pub(super) use measurement_writer::MeasurementWriter;
