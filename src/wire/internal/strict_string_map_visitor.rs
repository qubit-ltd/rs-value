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
use serde::de;
use serde::de::MapAccess;
use serde::de::Visitor;

/// Deserialization visitor that accumulates unique string-keyed entries.
///
/// # Type Parameters
///
/// * `V` - The value type deserialized for each string key.
pub(super) struct StrictStringMapVisitor<V>(PhantomData<V>);

impl<V> StrictStringMapVisitor<V> {
    /// Creates a visitor for values of type `V`.
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
    type Value = HashMap<String, V>;

    /// Describes the expected input shape.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with unique string keys")
    }

    /// Decodes all entries and rejects repeated keys.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The deserializer's map access implementation.
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
                    return Err(de::Error::custom(format!("duplicate map key '{}'", entry.key())));
                }
            }
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde::Deserialize;
    use serde::Deserializer as _;

    use super::StrictStringMapVisitor;

    thread_local! {
        static DESERIALIZE_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    #[derive(Debug)]
    struct Counted;

    impl<'de> Deserialize<'de> for Counted {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            DESERIALIZE_COUNT.with(|count| count.set(count.get() + 1));
            String::deserialize(deserializer).map(|_| Self)
        }
    }

    fn decode(input: &str) -> Result<std::collections::HashMap<String, Counted>, serde_json::Error> {
        let mut deserializer = serde_json::Deserializer::from_str(input);
        deserializer.deserialize_map(StrictStringMapVisitor::<Counted>::new())
    }

    #[test]
    fn test_strict_string_map_duplicate_key_skips_second_value() {
        DESERIALIZE_COUNT.with(|count| count.set(0));
        let error = decode(r#"{"k":"ok","k":"bad"}"#).unwrap_err();
        assert!(error.to_string().contains("duplicate map key 'k'"));
        DESERIALIZE_COUNT.with(|count| assert_eq!(count.get(), 1));

        DESERIALIZE_COUNT.with(|count| count.set(0));
        let error = decode(r#"{"k":"ok","k":1}"#).unwrap_err();
        assert!(error.to_string().contains("duplicate map key 'k'"));
        DESERIALIZE_COUNT.with(|count| assert_eq!(count.get(), 1));
    }

    #[test]
    fn test_strict_string_map_unique_empty_and_invalid_values() {
        DESERIALIZE_COUNT.with(|count| count.set(0));
        assert_eq!(decode(r#"{"a":"one","b":"two"}"#).unwrap().len(), 2);
        DESERIALIZE_COUNT.with(|count| assert_eq!(count.get(), 2));
        DESERIALIZE_COUNT.with(|count| count.set(0));
        assert!(decode("{}").unwrap().is_empty());
        DESERIALIZE_COUNT.with(|count| assert_eq!(count.get(), 0));
        assert!(decode(r#"{"a":1}"#).is_err());
        DESERIALIZE_COUNT.with(|count| assert_eq!(count.get(), 1));
    }
}
