// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public bound for strict reads from either runtime value shape.

use crate::{
    MultiValues,
    Value,
    ValueError,
};

/// Marks target types that can be read strictly from a scalar or collection.
///
/// This blanket trait preserves the exact-type semantics of [`Value`] and
/// [`MultiValues`] while hiding their shape-specific conversion bounds from
/// APIs that delegate to both containers.
pub trait StrictValueRead:
    for<'a> TryFrom<&'a Value, Error = ValueError>
    + for<'a> TryFrom<&'a MultiValues, Error = ValueError>
{
}

impl<T> StrictValueRead for T where
    for<'a> T: TryFrom<&'a Value, Error = ValueError>
        + TryFrom<&'a MultiValues, Error = ValueError>
{
}
