// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Error categories produced by strict JSON serialization.

use std::fmt::Display;

use crate::finite_float::NON_FINITE_FLOAT_MESSAGE;

/// Stable error categories needed by the public conversion layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub(crate) enum StrictJsonError {
    /// A non-finite float was encountered at any nesting level.
    #[error("non-finite float")]
    NonFinite,
    /// The input could not be represented as a JSON value.
    #[error("JSON serialization failed")]
    Serialization,
}

impl serde::ser::Error for StrictJsonError {
    /// Classifies a Serde error without exposing unstable diagnostic text.
    fn custom<T>(message: T) -> Self
    where
        T: Display,
    {
        if message.to_string() == NON_FINITE_FLOAT_MESSAGE {
            Self::NonFinite
        } else {
            Self::Serialization
        }
    }
}
