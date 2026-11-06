// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structural equality and hashing for JSON payloads.

use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_budget::MeasuredBudgetError;
use qubit_budget::ResourceQuantity;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueTransaction;
use qubit_json::value::traverse::JsonTreeReader;
use serde_json::Value;

use super::hash_destination::HashDestination;
use super::internal::hash_frame::HashFrame;
use super::internal::json_equality_frame::JsonEqualityFrame;
use super::internal::object_hash::ObjectHash;

/// Deterministic within this build, non-randomized hasher for each object
/// entry.
///
/// The standard hasher algorithm is not a persistent identity format across
/// Rust releases.
type IdentityHasher = BuildHasherDefault<DefaultHasher>;

/// Compares two JSON trees using structural JSON semantics without recursion.
///
/// Borrows both inputs without mutation. Continuation storage grows with tree
/// depth, and object children are matched by key lookup rather than order.
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
pub(crate) fn json_eq(left: &Value, right: &Value) -> bool {
    let mut frames = Vec::new();
    let mut next = Some((left, right));
    while let Some((left, right)) = next.take() {
        match (left, right) {
            (Value::Null, Value::Null) => {}
            (Value::Bool(left), Value::Bool(right)) if left == right => {}
            (Value::Number(left), Value::Number(right)) if left == right => {}
            (Value::String(left), Value::String(right)) if left == right => {}
            (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
                frames.push(JsonEqualityFrame::Array {
                    left: left.iter(),
                    right: right.iter(),
                });
            }
            (Value::Object(left), Value::Object(right)) if left.len() == right.len() => {
                frames.push(JsonEqualityFrame::Object {
                    left: left.iter(),
                    right,
                });
            }
            _ => return false,
        }
        while let Some(frame) = frames.last_mut() {
            next = match frame {
                JsonEqualityFrame::Array { left, right } => left.next().zip(right.next()),
                JsonEqualityFrame::Object { left, right } => match left.next() {
                    Some((key, left)) => {
                        let Some(right) = right.get(key) else {
                            return false;
                        };
                        Some((left, right))
                    }
                    None => None,
                },
            };
            if next.is_some() {
                break;
            }
            frames.pop();
        }
    }
    true
}

/// Hashes a JSON tree using structural, object-order-independent semantics.
///
/// Appends to `state` without mutating `value`. Traversal allocates
/// continuation and object-entry state proportional to nesting depth.
///
/// # Type Parameters
///
/// * `H` - Hasher receiving the structural JSON identity.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher.
///
/// # Panics
///
/// Propagates panics from the caller-provided hasher; writes already made to
/// `state` are not rolled back.
#[inline]
pub(crate) fn hash_json<H: Hasher>(value: &Value, state: &mut H) {
    hash_json_iterative(value, state);
}

/// Hashes a JSON tree while enforcing one mutable JSON budget.
///
/// Preflights the tree before writing to the hasher, then publishes staged
/// accounting only after hashing succeeds. Both traversals borrow `value` and
/// use continuation storage proportional to nesting depth when traversing.
///
/// # Type Parameters
///
/// * `H` - Hasher receiving the structural JSON identity.
/// * `R` - Resource identifier used by the JSON budget.
/// * `Q` - Quantity type used by the JSON budget.
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
///
/// # Panics
///
/// Propagates panics from the caller-provided hasher. The budget remains
/// uncommitted, but writes already made to `state` are not rolled back.
#[allow(dead_code)]
pub(crate) fn hash_json_with_budget<H, R, Q>(
    value: &Value,
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
    transaction.commit()
}

/// Checks every JSON event in `value` against a staged value transaction.
///
/// No caller-owned hasher or committed budget state is touched. Dropping the
/// transaction after an error therefore leaves both external states unchanged.
/// Successful charges remain staged for the caller to commit. A budget
/// rejection poisons the transaction, preventing publication of earlier staged
/// charges. With no configured limits, accounting returns without traversing
/// the tree; otherwise its continuation storage grows with nesting depth.
///
/// # Type Parameters
///
/// * `R` - Resource identifier used by the JSON budget.
/// * `Q` - Quantity type used by the JSON budget.
///
/// # Parameters
///
/// * `value` - JSON tree whose complete resource usage is measured.
/// * `transaction` - Staged budget transaction charged during traversal.
///
/// # Returns
///
/// `Ok(())` after every JSON event fits within the staged budget.
///
/// # Errors
///
/// Returns [`MeasuredBudgetError`] for the first resource measurement or
/// configured limit that cannot be satisfied.
#[inline]
pub(crate) fn preflight_json<R, Q>(
    value: &Value,
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
///
/// # Type Parameters
///
/// * `H` - Hasher receiving the structural JSON identity.
///
/// # Parameters
///
/// * `value` - JSON tree to traverse without recursion.
/// * `state` - Destination hasher.
///
/// # Panics
///
/// Panics if an internal continuation lacks its matching destination or object
/// aggregate, or if the caller-provided hasher panics. Hasher writes are not
/// rolled back.
fn hash_json_iterative<H>(value: &Value, state: &mut H)
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
                    Value::Null => destination.hash(&0_u8),
                    Value::Bool(value) => {
                        destination.hash(&1_u8);
                        destination.hash(value);
                    }
                    Value::Number(value) => {
                        destination.hash(&2_u8);
                        destination.hash(value);
                    }
                    Value::String(value) => {
                        destination.hash(&3_u8);
                        destination.hash(value);
                    }
                    Value::Array(values) => {
                        destination.hash(&4_u8);
                        let child_depth = depth.saturating_add(1);
                        frames.push(HashFrame::VisitArray {
                            values,
                            depth: child_depth,
                            next: 0,
                        });
                        frames.push(HashFrame::HashArrayLength(values.len()));
                    }
                    Value::Object(values) => {
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
            HashFrame::VisitArray { values, depth, next } => {
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
                let Some(hash) = destinations.pop().and_then(HashDestination::finish_object_entry) else {
                    continue;
                };
                let object = objects
                    .last_mut()
                    .expect("an object entry must have an object aggregate");
                object.sum = object.sum.wrapping_add(hash);
                object.xor ^= hash.rotate_left(17);
            }
            HashFrame::FinishObject => {
                let ObjectHash { sum, xor } = objects.pop().expect("a finished object must have an aggregate");
                let destination = destinations.last_mut().expect("an object must have a hash destination");
                destination.hash(&sum);
                destination.hash(&xor);
            }
        }
    }
}
