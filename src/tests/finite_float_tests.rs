// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-internal finite-float sequence adapter contracts.

use std::cell::Cell;
use std::iter::empty;

use serde::de::value::Error;
use serde::de::value::SeqDeserializer;

use crate::finite_float::NON_FINITE_FLOAT_MESSAGE;
use crate::finite_float::float32_vec;
use crate::finite_float::float64_vec;

#[test]
fn test_float32_sequence_stops_at_first_non_finite_item() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let reads = Cell::new(0);
        let items = [invalid, 1.0, 2.0].into_iter().inspect(|_| {
            reads.set(reads.get() + 1);
        });
        let error = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(items))
            .expect_err("non-finite f32 must be rejected");
        assert_eq!(error.to_string(), NON_FINITE_FLOAT_MESSAGE);
        assert_eq!(reads.get(), 1, "typed tail must remain unread");
    }
}

#[test]
fn test_float64_sequence_stops_at_first_non_finite_item() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let reads = Cell::new(0);
        let items = [invalid, 1.0, 2.0].into_iter().inspect(|_| {
            reads.set(reads.get() + 1);
        });
        let error = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(items))
            .expect_err("non-finite f64 must be rejected");
        assert_eq!(error.to_string(), NON_FINITE_FLOAT_MESSAGE);
        assert_eq!(reads.get(), 1, "typed tail must remain unread");
    }
}

#[test]
fn test_float32_sequence_keeps_valid_prefix_and_stops_before_tail() {
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let reads = Cell::new(0);
        let items = [1.0_f32, invalid, 3.0].into_iter().inspect(|_| {
            reads.set(reads.get() + 1);
        });
        let result = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(items));
        assert!(result.is_err(), "non-finite sequence item must be rejected");
        assert_eq!(
            reads.get(),
            2,
            "only prefix and invalid item may be requested"
        );
    }
}

#[test]
fn test_float64_sequence_keeps_valid_prefix_and_stops_before_tail() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let reads = Cell::new(0);
        let items = [1.0_f64, invalid, 3.0].into_iter().inspect(|_| {
            reads.set(reads.get() + 1);
        });
        let result = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(items));
        assert!(result.is_err(), "non-finite sequence item must be rejected");
        assert_eq!(
            reads.get(),
            2,
            "only prefix and invalid item may be requested"
        );
    }
}

#[test]
fn test_finite_sequences_preserve_empty_and_boundary_values() {
    let empty32 = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(empty::<f32>()))
        .expect("empty f32 collection is valid");
    let empty64 = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(empty::<f64>()))
        .expect("empty f64 collection is valid");
    assert!(empty32.is_empty(), "empty f32 sequence must remain empty");
    assert!(empty64.is_empty(), "empty f64 sequence must remain empty");
    let values32 = [f32::MIN, -0.0, 0.0, f32::from_bits(1), f32::MAX];
    let values64 = [f64::MIN, -0.0, 0.0, f64::from_bits(1), f64::MAX];
    let decoded32 =
        float32_vec::deserialize(SeqDeserializer::<_, Error>::new(values32.into_iter()))
            .expect("finite f32 values must keep their representation");
    let decoded64 =
        float64_vec::deserialize(SeqDeserializer::<_, Error>::new(values64.into_iter()))
            .expect("finite f64 values must keep their representation");
    assert_eq!(
        decoded32
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        values32.map(f32::to_bits).to_vec(),
        "finite f32 sequence must preserve order and exact bits"
    );
    assert_eq!(
        decoded64
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        values64.map(f64::to_bits).to_vec(),
        "finite f64 sequence must preserve order and exact bits"
    );
}
