// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Typed destinations for iterative JSON hashing.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

/// Routes iterative JSON hashing to a borrowed root or an owned entry hasher.
///
/// The root borrow lasts for the traversal; entry state is consumed when its
/// hash is merged into the order-independent object aggregate.
///
/// # Type Parameters
///
/// * `H` - Caller-selected hasher used for the root JSON value.
pub(in crate::identity) enum HashDestination<'a, H> {
    /// The caller-provided root destination.
    Root(
        /// Caller-owned destination used for the complete root identity.
        &'a mut H,
    ),
    /// The independent hasher for one JSON object entry.
    ObjectEntry(
        /// Independent state whose final hash is merged without key order.
        DefaultHasher,
    ),
}

impl<H> HashDestination<'_, H>
where
    H: Hasher,
{
    /// Hashes one value into this destination without erasing the hasher type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Hashable value type accepted by the active destination.
    ///
    /// # Parameters
    ///
    /// * `value` - Value whose identity bytes are appended to this destination.
    ///
    /// # Panics
    ///
    /// Panics if the value's `Hash` implementation or the active hasher panics.
    /// Bytes already written to the destination are not rolled back.
    #[inline(always)]
    pub(in crate::identity) fn hash<T>(&mut self, value: &T)
    where
        T: Hash + ?Sized,
    {
        match self {
            Self::Root(state) => value.hash(*state),
            Self::ObjectEntry(state) => value.hash(state),
        }
    }

    /// Returns the object-entry hash, or `None` for the root destination.
    ///
    /// # Returns
    ///
    /// `Some(hash)` for an object-entry destination and `None` for the caller's
    /// root destination.
    #[must_use]
    #[inline(always)]
    pub(in crate::identity) fn finish_object_entry(self) -> Option<u64> {
        match self {
            Self::ObjectEntry(state) => Some(state.finish()),
            Self::Root(_) => None,
        }
    }
}
