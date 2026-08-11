// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structural equality and hashing for JSON payloads.

use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_budget::BudgetError;
use qubit_budget::JsonBudget;
use qubit_budget::JsonResource;

type IdentityHasher =
    BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

/// One pending operation in the iterative JSON hashing traversal.
enum HashFrame<'a> {
    /// Visits one JSON node at its root-inclusive depth.
    Visit(&'a serde_json::Value, usize),

    /// Hashes an array length before its elements are visited.
    HashArrayLength(usize),

    /// Starts an object entry with its independent identity hasher.
    StartObjectEntry(&'a str),

    /// Finishes an object entry and adds its hash to the current object.
    FinishObjectEntry,

    /// Finishes an object and writes its order-independent aggregates.
    FinishObject,
}

/// A destination receiving hashes during one iterative traversal.
enum HashDestination<'a, H> {
    /// The caller-provided root destination.
    Root(&'a mut H),

    /// The independent hasher for one JSON object entry.
    ObjectEntry(std::collections::hash_map::DefaultHasher),
}

impl<H> HashDestination<'_, H>
where
    H: Hasher,
{
    /// Hashes one value into this destination without erasing the hasher type.
    #[inline(always)]
    fn hash<T>(&mut self, value: &T)
    where
        T: Hash + ?Sized,
    {
        match self {
            Self::Root(state) => value.hash(*state),
            Self::ObjectEntry(state) => value.hash(state),
        }
    }

    /// Finishes an object-entry destination and returns its hash.
    ///
    /// # Panics
    ///
    /// Panics if called for the root destination, which indicates an invalid
    /// internal frame sequence.
    #[inline(always)]
    fn finish_object_entry(self) -> u64 {
        match self {
            Self::ObjectEntry(state) => state.finish(),
            Self::Root(_) => {
                unreachable!("the root hasher cannot finish an object entry")
            }
        }
    }
}

/// Order-independent hash aggregates for one object currently being visited.
#[derive(Default)]
struct ObjectHash {
    /// Wrapping sum of the object's entry hashes.
    sum: u64,

    /// Rotated xor of the object's entry hashes.
    xor: u64,
}

/// Compares two JSON trees using structural JSON semantics without recursion.
///
/// # Parameters
///
/// * `left` - Left JSON tree.
/// * `right` - Right JSON tree.
///
/// # Returns
///
/// `true` when both trees are structurally equal. Object member order is not
/// significant and array element order is significant.
#[must_use]
#[inline(always)]
pub(crate) fn json_eq(
    left: &serde_json::Value,
    right: &serde_json::Value,
) -> bool {
    let mut pending = Vec::with_capacity(1);
    pending.push((left, right));
    while let Some((left, right)) = pending.pop() {
        match (left, right) {
            (serde_json::Value::Null, serde_json::Value::Null) => {}
            (serde_json::Value::Bool(left), serde_json::Value::Bool(right)) => {
                if left != right {
                    return false;
                }
            }
            (
                serde_json::Value::Number(left),
                serde_json::Value::Number(right),
            ) => {
                if left != right {
                    return false;
                }
            }
            (
                serde_json::Value::String(left),
                serde_json::Value::String(right),
            ) => {
                if left != right {
                    return false;
                }
            }
            (
                serde_json::Value::Array(left),
                serde_json::Value::Array(right),
            ) => {
                if left.len() != right.len() {
                    return false;
                }
                for (left, right) in left.iter().rev().zip(right.iter().rev()) {
                    pending.push((left, right));
                }
            }
            (
                serde_json::Value::Object(left),
                serde_json::Value::Object(right),
            ) => {
                if left.len() != right.len() {
                    return false;
                }
                for (key, left) in left {
                    let Some(right) = right.get(key) else {
                        return false;
                    };
                    pending.push((left, right));
                }
            }
            _ => return false,
        }
    }
    true
}

/// Hashes a JSON tree using structural, object-order-independent semantics.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher.
pub(crate) fn hash_json<H: Hasher>(value: &serde_json::Value, state: &mut H) {
    let result = hash_json_iterative::<H, JsonResource>(value, state, None);
    if result.is_err() {
        unreachable!("unbudgeted JSON hashing cannot fail");
    }
}

/// Hashes a JSON tree while enforcing one mutable JSON budget.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher, which may be partially updated on failure.
/// * `budget` - Mutable session receiving structural and text checks.
///
/// # Returns
///
/// `Ok(())` after the complete tree is hashed.
///
/// # Errors
///
/// Returns [`BudgetError`] when a node, container, key, string, or number text
/// exceeds the corresponding budget constraint.
pub(crate) fn hash_json_with_budget<H, R>(
    value: &serde_json::Value,
    state: &mut H,
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), BudgetError<R, usize>>
where
    H: Hasher,
    R: Clone,
{
    hash_json_iterative(value, state, Some(budget))
}

/// Runs the shared explicit-stack hashing engine with an optional budget.
///
/// A missing budget skips every constraint check, preserving the infallible
/// behavior of [`hash_json`].
fn hash_json_iterative<H, R>(
    value: &serde_json::Value,
    state: &mut H,
    mut budget: Option<&mut JsonBudget<R, usize>>,
) -> Result<(), BudgetError<R, usize>>
where
    H: Hasher,
    R: Clone,
{
    let mut frames = vec![HashFrame::Visit(value, 1)];
    let mut destinations = vec![HashDestination::Root(state)];
    let mut objects = Vec::<ObjectHash>::new();

    while let Some(frame) = frames.pop() {
        match frame {
            HashFrame::Visit(value, depth) => {
                check_value_budget(value, depth, budget.as_deref_mut())?;
                let destination = destinations
                    .last_mut()
                    .expect("a JSON node must have a hash destination");
                match value {
                    serde_json::Value::Null => destination.hash(&0_u8),
                    serde_json::Value::Bool(value) => {
                        destination.hash(&1_u8);
                        destination.hash(value);
                    }
                    serde_json::Value::Number(value) => {
                        destination.hash(&2_u8);
                        destination.hash(value);
                    }
                    serde_json::Value::String(value) => {
                        destination.hash(&3_u8);
                        destination.hash(value);
                    }
                    serde_json::Value::Array(values) => {
                        destination.hash(&4_u8);
                        let child_depth = depth.saturating_add(1);
                        for value in values.iter().rev() {
                            frames.push(HashFrame::Visit(value, child_depth));
                        }
                        frames.push(HashFrame::HashArrayLength(values.len()));
                    }
                    serde_json::Value::Object(values) => {
                        destination.hash(&5_u8);
                        destination.hash(&values.len());
                        objects.push(ObjectHash::default());
                        frames.push(HashFrame::FinishObject);
                        let child_depth = depth.saturating_add(1);
                        for (key, value) in values.iter().rev() {
                            frames.push(HashFrame::FinishObjectEntry);
                            frames.push(HashFrame::Visit(value, child_depth));
                            frames.push(HashFrame::StartObjectEntry(key));
                        }
                    }
                }
            }
            HashFrame::HashArrayLength(length) => {
                destinations
                    .last_mut()
                    .expect("an array must have a hash destination")
                    .hash(&length);
            }
            HashFrame::StartObjectEntry(key) => {
                if let Some(budget) = budget.as_deref_mut() {
                    budget.check_key_bytes(key.len())?;
                }
                let mut entry = IdentityHasher::default().build_hasher();
                key.hash(&mut entry);
                destinations.push(HashDestination::ObjectEntry(entry));
            }
            HashFrame::FinishObjectEntry => {
                let hash = destinations
                    .pop()
                    .expect("an object entry must have a hash destination")
                    .finish_object_entry();
                let object = objects
                    .last_mut()
                    .expect("an object entry must have an object aggregate");
                object.sum = object.sum.wrapping_add(hash);
                object.xor ^= hash.rotate_left(17);
            }
            HashFrame::FinishObject => {
                let ObjectHash { sum, xor } = objects
                    .pop()
                    .expect("a finished object must have an aggregate");
                let destination = destinations
                    .last_mut()
                    .expect("an object must have a hash destination");
                destination.hash(&sum);
                destination.hash(&xor);
            }
        }
    }
    Ok(())
}

/// Charges one JSON node and checks its container or text-specific limits.
///
/// A missing budget accepts the value without performing any work.
fn check_value_budget<R>(
    value: &serde_json::Value,
    depth: usize,
    budget: Option<&mut JsonBudget<R, usize>>,
) -> Result<(), BudgetError<R, usize>>
where
    R: Clone,
{
    let Some(budget) = budget else {
        return Ok(());
    };
    match value {
        serde_json::Value::Array(values) => {
            budget.enter_array(depth, values.len())
        }
        serde_json::Value::Object(values) => {
            budget.enter_object(depth, values.len())
        }
        serde_json::Value::String(value) => {
            budget.enter_node(depth)?;
            budget.check_string_bytes(value.len())
        }
        serde_json::Value::Number(value) => {
            budget.enter_node(depth)?;
            budget.check_number_bytes(value.as_str().len())
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) => {
            budget.enter_node(depth)
        }
    }
}
