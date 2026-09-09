// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Bounded formatting measurement without a temporary text allocation.

use std::fmt;

use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceBudget;
use qubit_datatype::ConversionResource;

/// Counts formatted bytes against the tightest remaining projection budget.
pub(super) struct MeasurementWriter {
    budget: ResourceBudget<ConversionResource, u64>,
    initial: u64,
    error: Option<MeasuredBudgetError<ConversionResource, u64>>,
}

impl MeasurementWriter {
    /// Borrows no text; retains the initial usage to report only newly written
    /// bytes.
    pub(super) fn new(budget: ResourceBudget<ConversionResource, u64>) -> Self {
        let initial = budget.used();
        Self {
            budget,
            initial,
            error: None,
        }
    }

    /// Returns measured bytes or the original resource/quantity error.
    pub(super) fn finish(self) -> Result<u64, MeasuredBudgetError<ConversionResource, u64>> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(self.budget.used() - self.initial),
        }
    }
}

impl fmt::Write for MeasurementWriter {
    /// Admits each fragment before counting it; stops immediately on overflow
    /// or exhaustion.
    fn write_str(&mut self, text: &str) -> fmt::Result {
        match self.budget.try_consume_usize(text.len()) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.error = Some(error);
                Err(fmt::Error)
            }
        }
    }
}
