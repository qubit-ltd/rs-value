// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared admission for one complete natural JSON projection.

// Keeps the private accounting implementation local to this projection helper.
mod internal {
    pub(super) mod projection_budget;
}

pub(super) use internal::projection_budget::ProjectionBudget;

#[cfg(test)]
mod tests {
    use std::fmt;

    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::DataType;

    use super::ProjectionBudget;
    use crate::ValueError;

    struct FailingDisplay;

    impl fmt::Display for FailingDisplay {
        fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    #[test]
    fn test_projection_budget_reports_display_failure() {
        let policy = ConversionPolicy::default();
        let limits = ConversionLimits::default();
        let mut budget = ProjectionBudget::new(DataType::String, &policy, &limits);

        let error = budget
            .display(&FailingDisplay, 0, false)
            .expect_err("formatting failure must become a conversion error");
        assert!(
            matches!(error, ValueError::Conversion(_)),
            "a failing Display implementation must produce a conversion error"
        );
    }
}
