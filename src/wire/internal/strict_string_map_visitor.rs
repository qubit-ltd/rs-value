// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Map visitor that rejects duplicate string keys during deserialization.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::Visitor;

/// Deserialization visitor that accumulates unique string-keyed entries.
///
/// # Type Parameters
///
/// * `V` - The value type deserialized for each string key.
pub(super) struct StrictStringMapVisitor<V>(
    /// Tracks the deserialized value type without storing or allocating values.
    PhantomData<V>,
);

impl<V> StrictStringMapVisitor<V> {
    /// Creates a visitor for values of type `V`.
    ///
    /// # Returns
    ///
    /// A stateless visitor that rejects repeated keys before decoding their
    /// values.
    #[must_use]
    #[inline]
    pub(super) fn new() -> Self {
        Self(PhantomData)
    }
}

impl<'de, V> Visitor<'de> for StrictStringMapVisitor<V>
where
    V: Deserialize<'de>,
{
    /// Owns the decoded values indexed by their unique string keys.
    type Value = HashMap<String, V>;

    /// Describes the expected input shape.
    ///
    /// # Parameters
    ///
    /// * `formatter` - Destination for Serde's expected-input description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Returns a formatting error if the destination rejects the description.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with unique string keys")
    }

    /// Decodes all entries and rejects repeated keys.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The deserializer's map access implementation.
    ///
    /// # Parameters
    ///
    /// * `map` - Owned access to the input entries, consumed in encounter
    ///   order.
    ///
    /// # Returns
    ///
    /// The owned map of decoded entries when all keys are unique.
    ///
    /// # Errors
    ///
    /// Returns the map access error if an entry cannot be decoded, or a custom
    /// deserialization error if a key appears more than once.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = HashMap::new();
        while let Some(key) = map.next_key::<String>()? {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(map.next_value::<V>()?);
                }
                Entry::Occupied(entry) => {
                    return Err(DeError::custom(format!(
                        "duplicate map key '{}'",
                        entry.key()
                    )));
                }
            }
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::HashMap;

    use serde::Deserialize;
    use serde::Deserializer;
    use serde_json::Deserializer as JsonDeserializer;
    use serde_json::Error as JsonError;

    use super::StrictStringMapVisitor;

    thread_local! {
        static DESERIALIZE_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    /// Counts value decode attempts, including attempts that fail.
    #[derive(Debug)]
    struct Counted;

    impl<'de> Deserialize<'de> for Counted {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            DESERIALIZE_COUNT.with(|count| count.set(count.get() + 1));
            String::deserialize(deserializer).map(|_| Self)
        }
    }

    /// Drives the private visitor with JSON and observable value decoding.
    fn decode(input: &str) -> Result<HashMap<String, Counted>, JsonError> {
        let mut deserializer = JsonDeserializer::from_str(input);
        deserializer.deserialize_map(StrictStringMapVisitor::<Counted>::new())
    }

    #[test]
    fn test_strict_string_map_duplicate_key_skips_second_value() {
        DESERIALIZE_COUNT.with(|count| count.set(0));
        let error = decode(r#"{"k":"ok","k":"bad"}"#).unwrap_err();
        assert!(
            error.to_string().contains("duplicate map key 'k'"),
            "expected duplicate-key error: {error}"
        );
        DESERIALIZE_COUNT.with(|count| {
            assert_eq!(count.get(), 1, "exactly one value decode must be attempted");
        });

        DESERIALIZE_COUNT.with(|count| count.set(0));
        let error = decode(r#"{"k":"ok","k":1}"#).unwrap_err();
        assert!(
            error.to_string().contains("duplicate map key 'k'"),
            "expected duplicate-key error: {error}"
        );
        DESERIALIZE_COUNT.with(|count| {
            assert_eq!(count.get(), 1, "exactly one value decode must be attempted");
        });
    }

    #[test]
    fn test_strict_string_map_unique_empty_and_invalid_values() {
        DESERIALIZE_COUNT.with(|count| count.set(0));
        let values = decode(r#"{"a":"one","b":"two"}"#).unwrap();
        assert_eq!(values.len(), 2, "both unique keys must be retained");
        assert!(
            values.contains_key("a"),
            "the first unique key must be retained"
        );
        assert!(
            values.contains_key("b"),
            "the second unique key must be retained"
        );
        DESERIALIZE_COUNT.with(|count| {
            assert_eq!(count.get(), 2, "both unique values must be decoded");
        });
        DESERIALIZE_COUNT.with(|count| count.set(0));
        assert!(
            decode("{}").unwrap().is_empty(),
            "empty input must produce an empty map"
        );
        DESERIALIZE_COUNT.with(|count| {
            assert_eq!(count.get(), 0, "empty input must not decode a value");
        });
        assert!(
            decode(r#"{"a":1}"#).is_err(),
            "non-string values must be rejected"
        );
        DESERIALIZE_COUNT.with(|count| {
            assert_eq!(count.get(), 1, "exactly one value decode must be attempted");
        });
    }
}
