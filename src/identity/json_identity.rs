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

use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceQuantity;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueTransaction;
use qubit_json::value::traverse::JsonTreeReader;

use super::hash_destination::HashDestination;
use super::hash_frame::HashFrame;
use super::object_hash::ObjectHash;

/// Stable standard hasher used for each order-independent object entry.
type IdentityHasher =
    BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

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
    hash_json_iterative(value, state);
}

/// Hashes a JSON tree while enforcing one mutable JSON budget.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher receiving the structural JSON identity.
/// * `budget` - Mutable session receiving structural and text checks.
///
/// # Returns
///
/// `Ok(())` after the complete tree is hashed.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] when a node, container, key, string, or
/// number text exceeds the corresponding budget constraint. On error, neither
/// `state` nor the committed portion of `budget` is modified. If hashing panics
/// after preflight, the staged budget is also discarded.
#[allow(dead_code)]
pub(crate) fn hash_json_with_budget<H, R, Q>(
    value: &serde_json::Value,
    state: &mut H,
    budget: &mut JsonValueBudget<R, Q>,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    H: Hasher,
    R: Clone,
    Q: ResourceQuantity,
{
    let mut transaction = budget.transaction();
    preflight_json(value, &mut transaction)?;
    hash_json(value, state);
    transaction.commit();
    Ok(())
}

/// Checks every JSON event in `value` against a staged value transaction.
///
/// No caller-owned hasher or committed budget state is touched. Dropping the
/// transaction after an error therefore leaves both external states unchanged.
pub(crate) fn preflight_json<R, Q>(
    value: &serde_json::Value,
    transaction: &mut JsonValueTransaction<'_, R, Q>,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    R: Clone,
    Q: ResourceQuantity,
{
    JsonTreeReader::new(transaction).account(value)
}

/// Runs the infallible explicit-stack hashing engine.
///
/// Container continuations visit one child at a time, so pending traversal
/// storage depends on nesting depth rather than the width of any one array or
/// object.
fn hash_json_iterative<H>(value: &serde_json::Value, state: &mut H)
where
    H: Hasher,
{
    let mut frames = vec![HashFrame::Visit(value, 1)];
    let mut destinations = vec![HashDestination::Root(state)];
    let mut objects = Vec::<ObjectHash>::new();

    while let Some(frame) = frames.pop() {
        match frame {
            HashFrame::Visit(value, depth) => {
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
                        frames.push(HashFrame::VisitArray {
                            values,
                            depth: child_depth,
                            next: 0,
                        });
                        frames.push(HashFrame::HashArrayLength(values.len()));
                    }
                    serde_json::Value::Object(values) => {
                        destination.hash(&5_u8);
                        destination.hash(&values.len());
                        objects.push(ObjectHash::default());
                        frames.push(HashFrame::FinishObject);
                        let child_depth = depth.saturating_add(1);
                        frames.push(HashFrame::VisitObject {
                            entries: values.iter(),
                            depth: child_depth,
                        });
                    }
                }
            }
            HashFrame::HashArrayLength(length) => {
                destinations
                    .last_mut()
                    .expect("an array must have a hash destination")
                    .hash(&length);
            }
            HashFrame::VisitArray {
                values,
                depth,
                next,
            } => {
                if let Some(value) = values.get(next) {
                    frames.push(HashFrame::VisitArray {
                        values,
                        depth,
                        next: next.saturating_add(1),
                    });
                    frames.push(HashFrame::Visit(value, depth));
                }
            }
            HashFrame::VisitObject { mut entries, depth } => {
                if let Some((key, value)) = entries.next() {
                    frames.push(HashFrame::VisitObject { entries, depth });
                    frames.push(HashFrame::FinishObjectEntry);
                    frames.push(HashFrame::Visit(value, depth));
                    frames.push(HashFrame::StartObjectEntry(key));
                }
            }
            HashFrame::StartObjectEntry(key) => {
                let mut entry = IdentityHasher::default().build_hasher();
                key.hash(&mut entry);
                destinations.push(HashDestination::ObjectEntry(entry));
            }
            HashFrame::FinishObjectEntry => {
                let Some(hash) = destinations
                    .pop()
                    .and_then(HashDestination::finish_object_entry)
                else {
                    continue;
                };
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
}
