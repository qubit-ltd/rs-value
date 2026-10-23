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
    /// Tracks admitted formatted bytes against the tightest active resource
    /// limit.
    budget: ResourceBudget<ConversionResource, u64>,
    /// Records usage before formatting begins so `finish` can return only the
    /// bytes written by this formatter.
    initial: u64,
    /// Retains the first budget failure returned by the formatter.
    error: Option<MeasuredBudgetError<ConversionResource, u64>>,
}

impl MeasurementWriter {
    /// Borrows no text; retains the initial usage to report only newly written
    /// bytes.
    ///
    /// # Parameters
    ///
    /// - `budget`: the resource budget charged for each formatted fragment.
    ///
    /// # Returns
    ///
    /// A writer that measures subsequent formatted bytes against `budget`.
    #[must_use]
    #[inline]
    pub(super) fn new(budget: ResourceBudget<ConversionResource, u64>) -> Self {
        let initial = budget.used();
        Self {
            budget,
            initial,
            error: None,
        }
    }

    /// Returns the bytes admitted during formatting, or the first budget error.
    ///
    /// # Returns
    ///
    /// The number of newly admitted bytes when formatting stayed within budget.
    ///
    /// # Errors
    ///
    /// Returns the original resource or quantity error recorded while writing.
    #[inline]
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
    ///
    /// # Parameters
    ///
    /// - `text`: the formatted fragment to admit against the remaining budget.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the fragment fits within the budget.
    ///
    /// # Errors
    ///
    /// Returns `fmt::Error` when the budget cannot admit the fragment. The
    /// underlying budget error remains available through `finish`.
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
