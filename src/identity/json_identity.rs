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
use qubit_budget::json::JsonMeasurement;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueTransaction;
use qubit_json::value::json_number_lexeme_length;

type IdentityHasher =
    BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

/// One pending operation in the iterative JSON hashing traversal.
enum HashFrame<'a> {
    /// Visits one JSON node at its root-inclusive depth.
    Visit(&'a serde_json::Value, usize),

    /// Hashes an array length before its elements are visited.
    HashArrayLength(usize),

    /// Continues visiting an array from its next element.
    VisitArray {
        /// Array elements being visited.
        values: &'a [serde_json::Value],

        /// Root-inclusive depth of each array element.
        depth: usize,

        /// Index of the next element to visit.
        next: usize,
    },

    /// Continues visiting an object from its next entry.
    VisitObject {
        /// Object entries being visited.
        entries: serde_json::map::Iter<'a>,

        /// Root-inclusive depth of each object value.
        depth: usize,
    },

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
/// Returns [`BudgetError`] when a node, container, key, string, or number text
/// exceeds the corresponding budget constraint. On error, neither `state` nor
/// the committed portion of `budget` is modified. If hashing panics after
/// preflight, the staged budget is also discarded.
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
    enum Frame<'a> {
        Visit(&'a serde_json::Value, usize),
        VisitArray {
            values: &'a [serde_json::Value],
            depth: usize,
            next: usize,
        },
        VisitObject {
            entries: serde_json::map::Iter<'a>,
            depth: usize,
        },
    }

    let mut frames = vec![Frame::Visit(value, 1)];
    while let Some(frame) = frames.pop() {
        match frame {
            Frame::Visit(value, depth) => {
                let measurement = match value {
                    serde_json::Value::Null => JsonMeasurement::Null { depth },
                    serde_json::Value::Bool(_) => {
                        JsonMeasurement::Boolean { depth }
                    }
                    serde_json::Value::String(value) => {
                        JsonMeasurement::String {
                            depth,
                            bytes: value.len(),
                        }
                    }
                    serde_json::Value::Number(value) => {
                        JsonMeasurement::Number {
                            depth,
                            bytes: json_number_lexeme_length(value),
                        }
                    }
                    serde_json::Value::Array(values) => {
                        JsonMeasurement::Array {
                            depth,
                            items: values.len(),
                        }
                    }
                    serde_json::Value::Object(values) => {
                        JsonMeasurement::Object {
                            depth,
                            entries: values.len(),
                        }
                    }
                };
                transaction.try_admit(measurement)?;

                match value {
                    serde_json::Value::Array(values) => {
                        frames.push(Frame::VisitArray {
                            values,
                            depth: depth.saturating_add(1),
                            next: 0,
                        });
                    }
                    serde_json::Value::Object(values) => {
                        frames.push(Frame::VisitObject {
                            entries: values.iter(),
                            depth: depth.saturating_add(1),
                        });
                    }
                    _ => {}
                }
            }
            Frame::VisitArray {
                values,
                depth,
                next,
            } => {
                if let Some(value) = values.get(next) {
                    frames.push(Frame::VisitArray {
                        values,
                        depth,
                        next: next.saturating_add(1),
                    });
                    frames.push(Frame::Visit(value, depth));
                }
            }
            Frame::VisitObject { mut entries, depth } => {
                if let Some((key, value)) = entries.next() {
                    frames.push(Frame::VisitObject { entries, depth });
                    transaction
                        .try_admit(JsonMeasurement::Key { bytes: key.len() })?;
                    frames.push(Frame::Visit(value, depth));
                }
            }
        }
    }
    Ok(())
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
}
