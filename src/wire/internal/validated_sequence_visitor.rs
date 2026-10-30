// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deserializes and validates sequence elements as they are read.

use std::marker::PhantomData;

use serde::Deserialize;
use serde::Deserializer;
use serde::de::Error as _;
use serde::de::SeqAccess;
use serde::de::Visitor;

/// Converts each sequence payload before requesting the next one.
pub(in crate::wire) struct ValidatedSequenceVisitor<P, T, F> {
    convert: F,
    marker: PhantomData<fn(P) -> T>,
}

impl<P, T, F> ValidatedSequenceVisitor<P, T, F> {
    /// Creates a visitor that converts payloads into validated values.
    pub(in crate::wire) fn new(convert: F) -> Self {
        Self {
            convert,
            marker: PhantomData,
        }
    }

    /// Deserializes a sequence, stopping when an element fails conversion.
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
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a sequence of valid payloads")
    }

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
    use serde::de::value::SeqDeserializer;

    use super::ValidatedSequenceVisitor;

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

        let result = visitor.deserialize(SeqDeserializer::<_, serde::de::value::Error>::new(&mut values));

        assert!(result.is_err());
        assert_eq!(values.reads, 1);
    }

    #[test]
    fn test_empty_sequence_returns_empty_vector() {
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(Ok);
        let result = visitor.deserialize(SeqDeserializer::<_, serde::de::value::Error>::new(std::iter::empty::<
            i32,
        >()));
        assert_eq!(result.expect("empty input is a valid sequence"), Vec::<i32>::new());
    }

    #[test]
    fn test_successful_sequence_preserves_order() {
        let visitor = ValidatedSequenceVisitor::<i32, i32, _>::new(Ok);
        let result = visitor.deserialize(SeqDeserializer::<_, serde::de::value::Error>::new(
            [3, 1, 2].into_iter(),
        ));
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

        let result = visitor.deserialize(SeqDeserializer::<_, serde::de::value::Error>::new(&mut values));

        assert!(result.is_err());
        assert_eq!(values.reads, 2);
    }
}
