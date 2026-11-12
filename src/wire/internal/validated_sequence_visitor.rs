// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deserializes and validates sequence elements as they are read.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error as _;
use serde::de::SeqAccess;
use serde::de::Visitor;

/// Decodes and converts sequence elements one at a time.
///
/// `P` is the deserialized payload type, `T` is the converted value type, and
/// `F` converts one payload into a validated value. The `convert` field stores
/// that conversion, while `marker` records the relationship between `P` and
/// `T` without storing either type directly. If decoding or conversion of an
/// element fails, the visitor returns immediately and does not request later
/// typed elements. This does not bypass any whole-document preflight performed
/// by the caller, and resources already consumed are not rolled back.
///
/// # Type Parameters
///
/// - `P`: Payload decoded from each sequence element.
/// - `T`: Validated output value stored in input order.
/// - `F`: Conversion owned by the visitor; deserialization requires `FnMut(P)
///   -> Result<T, String>`.
pub(in crate::wire) struct ValidatedSequenceVisitor<P, T, F> {
    /// Converts each decoded payload, returning a message when it is invalid.
    convert: F,
    /// Associates the payload and output types with this visitor.
    marker: PhantomData<fn(P) -> T>,
}

impl<P, T, F> ValidatedSequenceVisitor<P, T, F> {
    /// Creates a visitor with the conversion applied to each decoded payload.
    ///
    /// # Parameters
    ///
    /// - `convert`: Converts a payload into its validated value, or returns a
    ///   diagnostic message when validation fails.
    ///
    /// # Returns
    ///
    /// A visitor owning the conversion, without decoding any input.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn new(convert: F) -> Self {
        Self {
            convert,
            marker: PhantomData,
        }
    }

    /// Deserializes a sequence, converting each payload before reading the
    /// next typed element.
    ///
    /// # Type Parameters
    ///
    /// - `'de`: Lifetime of data borrowed by decoded payloads.
    /// - `D`: Deserializer supplying the sequence and its error type. `P` must
    ///   implement `Deserialize<'de>`, and `F` must convert each `P` into
    ///   `Result<T, String>`.
    ///
    /// # Parameters
    ///
    /// - `deserializer`: The source deserializer, which must provide a sequence
    ///   representation.
    ///
    /// # Returns
    ///
    /// The converted values in input order.
    ///
    /// # Errors
    ///
    /// Returns the deserializer's error if the sequence or a payload cannot
    /// be decoded, or if conversion returns a validation message. After either
    /// failure, no later typed element is requested. Any outer whole-document
    /// preflight remains in effect; consumption performed before an error is
    /// not rolled back.
    #[inline]
    pub(in crate::wire) fn deserialize<'de, D>(self, deserializer: D) -> Result<Vec<T>, D::Error>
    where
        D: Deserializer<'de>,
        P: Deserialize<'de>,
        F: FnMut(P) -> Result<T, String>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de, P, T, F> Visitor<'de> for ValidatedSequenceVisitor<P, T, F>
where
    P: Deserialize<'de>,
    F: FnMut(P) -> Result<T, String>,
{
    /// Validated values collected in the same order as the input elements.
    type Value = Vec<T>;

    /// Describes the sequence of payloads accepted by this visitor.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination for the expected-input description.
    ///
    /// # Returns
    ///
    /// `Ok(())` after writing the description.
    ///
    /// # Errors
    ///
    /// Propagates a formatting failure from the destination.
    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str("a sequence of valid payloads")
    }

    /// Consumes the sequence, validating each payload before requesting
    /// another.
    ///
    /// # Type Parameters
    ///
    /// - `A`: Sequence access supplied by the deserializer.
    ///
    /// # Parameters
    ///
    /// - `sequence`: Source of payloads, consumed incrementally.
    ///
    /// # Returns
    ///
    /// Validated values in input order, or an empty vector for an empty
    /// sequence.
    ///
    /// # Errors
    ///
    /// Returns the first decoding or conversion error without requesting later
    /// elements; previously consumed input is not restored.
    fn visit_seq<A>(mut self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(payload) = sequence.next_element::<P>()? {
            values.push((self.convert)(payload).map_err(A::Error::custom)?);
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use std::iter::empty;

    use serde::de::value::Error as ValueError;
    use serde::de::value::SeqDeserializer;

    use super::ValidatedSequenceVisitor;

    /// Counts yielded payloads to detect reads beyond a validation failure.
    struct Counted<I> {
        inner: I,
        reads: usize,
    }

    impl<I> Iterator for Counted<I>
    where
        I: Iterator,
    {
        type Item = I::Item;

        fn next(&mut self) -> Option<Self::Item> {
            let value = self.inner.next();
            if value.is_some() {
                self.reads += 1;
            }
            value
        }
    }

    #[test]
    fn test_conversion_error_stops_before_reading_tail() {
        let mut values = Counted {
            inner: [1, 2, 3].into_iter(),
            reads: 0,
        };
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(|value| {
            if value == 1 {
                Err("invalid first item".to_owned())
            } else {
                Ok(value)
            }
        });

        let result = visitor.deserialize(SeqDeserializer::<_, ValueError>::new(&mut values));

        assert_eq!(
            result
                .expect_err("the first payload is invalid")
                .to_string(),
            "invalid first item",
        );
        assert_eq!(values.reads, 1, "the tail must remain unread");
    }

    #[test]
    fn test_empty_sequence_returns_empty_vector() {
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(Ok);
        let result = visitor.deserialize(SeqDeserializer::<_, ValueError>::new(empty::<i32>()));
        assert_eq!(
            result.expect("empty input is a valid sequence"),
            Vec::<i32>::new()
        );
    }

    #[test]
    fn test_successful_sequence_preserves_order() {
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(Ok);
        let result =
            visitor.deserialize(SeqDeserializer::<_, ValueError>::new([3, 1, 2].into_iter()));
        assert_eq!(result.expect("all payloads are valid"), vec![3, 1, 2]);
    }

    #[test]
    fn test_second_item_error_stops_before_remaining_tail() {
        let mut values = Counted {
            inner: [1, 2, 3].into_iter(),
            reads: 0,
        };
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(|value| {
            if value == 2 {
                Err("invalid second item".to_owned())
            } else {
                Ok(value)
            }
        });

        let result = visitor.deserialize(SeqDeserializer::<_, ValueError>::new(&mut values));

        assert_eq!(
            result
                .expect_err("the second payload is invalid")
                .to_string(),
            "invalid second item",
        );
        assert_eq!(values.reads, 2, "the remaining tail must stay unread");
    }
}
