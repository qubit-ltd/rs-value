// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared resource limits and accounting for JSON wire decoding.
// qubit-style: allow multiple-public-types

use std::fmt;

use serde::Deserializer;
use serde::de::DeserializeSeed;
use serde::de::Error as DeError;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::ValueWireDecodeError;
use super::ValueWireLimitKind;
use super::internal::display_length;
use crate::MultiValuesRef;
use crate::ValueContainer;
use crate::ValueRef;
use crate::wire::JSON_NUMBER_TOKEN;

/// Shared limits applied to one complete wire decode.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireLimits {
    max_input_bytes: usize,
    max_depth: usize,
    max_nodes: usize,
    max_collection_items: usize,
    max_map_entries: usize,
    max_string_bytes: usize,
    max_numeric_bytes: usize,
}

impl WireLimits {
    /// Default maximum complete JSON input length.
    pub const DEFAULT_MAX_INPUT_BYTES: usize = 1_048_576;
    /// Default maximum recursive wire depth.
    pub const DEFAULT_MAX_DEPTH: usize = 64;
    /// Default maximum decoded node count.
    pub const DEFAULT_MAX_NODES: usize = 100_000;
    /// Default maximum elements in one collection.
    pub const DEFAULT_MAX_COLLECTION_ITEMS: usize = 4_096;
    /// Default maximum entries in one map.
    pub const DEFAULT_MAX_MAP_ENTRIES: usize = 4_096;
    /// Default maximum bytes in one decoded string.
    pub const DEFAULT_MAX_STRING_BYTES: usize = 256 * 1024;
    /// Default maximum UTF-8 bytes in one decoded numeric representation.
    pub const DEFAULT_MAX_NUMERIC_BYTES: usize = 4_096;

    /// Creates shared wire limits with the specified input-byte bound.
    #[inline(always)]
    pub const fn new(max_input_bytes: usize) -> Self {
        Self {
            max_input_bytes,
            max_depth: Self::DEFAULT_MAX_DEPTH,
            max_nodes: Self::DEFAULT_MAX_NODES,
            max_collection_items: Self::DEFAULT_MAX_COLLECTION_ITEMS,
            max_map_entries: Self::DEFAULT_MAX_MAP_ENTRIES,
            max_string_bytes: Self::DEFAULT_MAX_STRING_BYTES,
            max_numeric_bytes: Self::DEFAULT_MAX_NUMERIC_BYTES,
        }
    }

    /// Sets the maximum recursive wire depth.
    #[inline(always)]
    #[must_use = "the configured wire depth limit should be used"]
    pub const fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the maximum decoded node count.
    #[inline(always)]
    #[must_use = "the configured wire node limit should be used"]
    pub const fn with_max_nodes(mut self, max_nodes: usize) -> Self {
        self.max_nodes = max_nodes;
        self
    }

    /// Sets the maximum elements in one collection.
    #[inline(always)]
    #[must_use = "the configured collection limit should be used"]
    pub const fn with_max_collection_items(
        mut self,
        max_collection_items: usize,
    ) -> Self {
        self.max_collection_items = max_collection_items;
        self
    }

    /// Sets the maximum entries in one map.
    #[inline(always)]
    #[must_use = "the configured map limit should be used"]
    pub const fn with_max_map_entries(
        mut self,
        max_map_entries: usize,
    ) -> Self {
        self.max_map_entries = max_map_entries;
        self
    }

    /// Sets the maximum bytes in one decoded string.
    #[inline(always)]
    #[must_use = "the configured string limit should be used"]
    pub const fn with_max_string_bytes(
        mut self,
        max_string_bytes: usize,
    ) -> Self {
        self.max_string_bytes = max_string_bytes;
        self
    }

    /// Sets the maximum UTF-8 bytes in one decoded numeric representation.
    #[inline(always)]
    #[must_use = "the configured numeric limit should be used"]
    pub const fn with_max_numeric_bytes(
        mut self,
        max_numeric_bytes: usize,
    ) -> Self {
        self.max_numeric_bytes = max_numeric_bytes;
        self
    }

    /// Returns the maximum complete input length.
    #[must_use]
    #[inline(always)]
    pub const fn max_input_bytes(self) -> usize {
        self.max_input_bytes
    }

    /// Returns the maximum recursive depth.
    #[must_use]
    #[inline(always)]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Returns the maximum decoded node count.
    #[must_use]
    #[inline(always)]
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum elements in one collection.
    #[must_use]
    #[inline(always)]
    pub const fn max_collection_items(self) -> usize {
        self.max_collection_items
    }

    /// Returns the maximum entries in one map.
    #[must_use]
    #[inline(always)]
    pub const fn max_map_entries(self) -> usize {
        self.max_map_entries
    }

    /// Returns the maximum bytes in one decoded string.
    #[must_use]
    #[inline(always)]
    pub const fn max_string_bytes(self) -> usize {
        self.max_string_bytes
    }

    /// Returns the maximum UTF-8 bytes in one decoded numeric representation.
    #[must_use]
    #[inline(always)]
    pub const fn max_numeric_bytes(self) -> usize {
        self.max_numeric_bytes
    }

    /// Checks a complete input length and starts a shared accounting session.
    #[inline]
    pub fn begin(
        self,
        input_bytes: usize,
    ) -> Result<WireBudget, ValueWireDecodeError> {
        if input_bytes > self.max_input_bytes {
            return Err(ValueWireDecodeError::InputTooLarge {
                input_bytes,
                max_input_bytes: self.max_input_bytes,
            });
        }
        Ok(WireBudget {
            limits: self,
            nodes: 0,
        })
    }

    /// Preflights one complete JSON document before decoding its runtime
    /// representation, then starts a semantic accounting session.
    ///
    /// The preflight validates complete-input size and JSON syntax while
    /// traversing the document without materializing a JSON tree. Runtime
    /// resource limits are charged by the returned budget after decoding so
    /// embedded protocol wrappers do not consume value-node or depth headroom.
    ///
    /// # Errors
    ///
    /// Returns an input-size or JSON syntax error before the caller
    /// deserializes its concrete wire DTO. Semantic resource errors are
    /// returned by the budget after the DTO has been materialized.
    /// Normal decode paths should use [`Self::begin`] and let their Serde
    /// decoder validate syntax once.
    #[inline]
    pub fn begin_json(
        self,
        input: &[u8],
    ) -> Result<WireBudget, ValueWireDecodeError> {
        self.check_json_bytes(input.len())?;
        let mut deserializer = serde_json::Deserializer::from_slice(input);
        let mut preflight = JsonPreflightSeed::new(input.len());
        if let Err(error) = (&mut preflight).deserialize(&mut deserializer) {
            if let Some(error) = preflight.violation.take() {
                return Err(error);
            }
            return Err(ValueWireDecodeError::InvalidJson(error));
        }
        deserializer
            .end()
            .map_err(ValueWireDecodeError::InvalidJson)?;
        self.begin(input.len())
    }

    /// Checks a complete input length without starting an accounting session.
    #[inline]
    pub const fn check_json_bytes(
        self,
        input_bytes: usize,
    ) -> Result<(), ValueWireDecodeError> {
        if input_bytes > self.max_input_bytes {
            Err(ValueWireDecodeError::InputTooLarge {
                input_bytes,
                max_input_bytes: self.max_input_bytes,
            })
        } else {
            Ok(())
        }
    }
}

impl Default for WireLimits {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Self::DEFAULT_MAX_INPUT_BYTES)
    }
}

/// Mutable accounting state shared by one complete wire decode.
#[must_use]
#[derive(Debug)]
pub struct WireBudget {
    limits: WireLimits,
    nodes: usize,
}

impl WireBudget {
    /// Returns the configured limits for this session.
    #[inline(always)]
    pub const fn limits(&self) -> WireLimits {
        self.limits
    }

    /// Charges one decoded node.
    #[inline]
    pub fn check_node(&mut self) -> Result<(), ValueWireDecodeError> {
        self.nodes = self.nodes.saturating_add(1);
        self.check_limit(
            ValueWireLimitKind::Nodes,
            self.nodes,
            self.limits.max_nodes,
        )
    }

    /// Checks a recursive depth.
    #[inline]
    pub fn check_depth(
        &self,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::Depth,
            depth,
            self.limits.max_depth,
        )
    }

    /// Checks one collection length.
    #[inline]
    pub fn check_collection_items(
        &self,
        items: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::CollectionItems,
            items,
            self.limits.max_collection_items,
        )
    }

    /// Checks one map length.
    #[inline]
    pub fn check_map_entries(
        &self,
        entries: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::MapEntries,
            entries,
            self.limits.max_map_entries,
        )
    }

    /// Checks one decoded string length.
    #[inline]
    pub fn check_string_bytes(
        &self,
        bytes: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::StringBytes,
            bytes,
            self.limits.max_string_bytes,
        )
    }

    /// Checks one decoded numeric representation length in UTF-8 bytes.
    #[inline]
    pub fn check_numeric_bytes(
        &self,
        bytes: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::NumericBytes,
            bytes,
            self.limits.max_numeric_bytes,
        )
    }

    /// Validates a decoded value container against the shared budget.
    #[inline]
    pub fn check_container(
        &mut self,
        container: &ValueContainer,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_container_at(container, 1)
    }

    /// Validates one decoded value container at `depth`.
    pub fn check_container_at(
        &mut self,
        container: &ValueContainer,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        match container {
            ValueContainer::Scalar(value) => {
                self.check_value_ref(value.view(), depth)
            }
            ValueContainer::Collection(values) => {
                self.check_node()?;
                self.check_collection_items(values.len())?;
                self.check_multi_values_ref(values.view(), depth)
            }
        }
    }

    /// Validates one scalar Value against the shared budget.
    #[inline]
    pub fn check_value(
        &mut self,
        value: &crate::Value,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_value_ref(value.view(), 1)
    }

    /// Validates one homogeneous collection against the shared budget.
    ///
    /// # Parameters
    ///
    /// * `values` - Collection whose elements are charged to this budget.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when the collection exceeds the
    /// configured depth, node, item, string, map, or numeric limits.
    #[inline]
    pub fn check_multi_values(
        &mut self,
        values: &crate::MultiValues,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_multi_values_at(values, 1)
    }

    /// Validates one homogeneous collection at an embedding depth.
    ///
    /// # Parameters
    ///
    /// * `values` - Collection whose elements are charged to this budget.
    /// * `depth` - Root-inclusive depth of the collection.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when the collection exceeds the
    /// configured depth, node, item, string, map, or numeric limits.
    pub fn check_multi_values_at(
        &mut self,
        values: &crate::MultiValues,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        self.check_collection_items(values.len())?;
        self.check_multi_values_ref(values.view(), depth)
    }

    /// Validates a named scalar and reuses scalar budget accounting.
    ///
    /// # Parameters
    ///
    /// * `value` - Named scalar to validate.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error for the wrapper name or nested value.
    #[inline]
    pub fn check_named_value(
        &mut self,
        value: &crate::NamedValue,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_named_value_at(value, 1)
    }

    /// Validates a named scalar at an embedding depth.
    ///
    /// # Parameters
    ///
    /// * `value` - Named scalar to validate.
    /// * `depth` - Root-inclusive depth of the named wrapper.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error for the wrapper name or nested value.
    pub fn check_named_value_at(
        &mut self,
        value: &crate::NamedValue,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        self.check_string_bytes(value.name().len())?;
        self.check_value_ref(value.value().view(), depth.saturating_add(1))
    }

    /// Validates a named collection and reuses collection budget accounting.
    ///
    /// # Parameters
    ///
    /// * `value` - Named collection to validate.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error for the wrapper name or nested values.
    #[inline]
    pub fn check_named_multi_values(
        &mut self,
        value: &crate::NamedMultiValues,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_named_multi_values_at(value, 1)
    }

    /// Validates a named collection at an embedding depth.
    ///
    /// # Parameters
    ///
    /// * `value` - Named collection to validate.
    /// * `depth` - Root-inclusive depth of the named wrapper.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error for the wrapper name or nested values.
    pub fn check_named_multi_values_at(
        &mut self,
        value: &crate::NamedMultiValues,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        self.check_string_bytes(value.name().len())?;
        self.check_multi_values_at(value.values(), depth.saturating_add(1))
    }

    fn check_value_ref(
        &mut self,
        value: ValueRef<'_>,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        match value {
            ValueRef::Char(value) => self.check_string_bytes(value.len_utf8()),
            ValueRef::String(value) => self.check_string_bytes(value.len()),
            ValueRef::StringMap(value) => {
                self.check_map_entries(value.len())?;
                for (key, value) in value {
                    self.check_string_bytes(key.len())?;
                    self.check_depth(depth.saturating_add(1))?;
                    self.check_node()?;
                    self.check_string_bytes(value.len())?;
                }
                Ok(())
            }
            #[cfg(feature = "json")]
            ValueRef::Json(value) => {
                self.check_json(value, depth.saturating_add(1))
            }
            #[cfg(feature = "big-integer")]
            ValueRef::BigInteger(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            #[cfg(feature = "big-decimal")]
            ValueRef::BigDecimal(value) => {
                self.check_numeric_bytes(big_decimal_numeric_len(value))
            }
            ValueRef::Int8(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Int16(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Int32(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Int64(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Int128(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::UInt8(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::UInt16(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::UInt32(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::UInt64(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::UInt128(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Float32(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            ValueRef::Float64(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            #[cfg(feature = "chrono")]
            ValueRef::Date(value) => {
                self.check_string_bytes(display_length(value.format("%F")))
            }
            #[cfg(feature = "chrono")]
            ValueRef::Time(value) => self.check_string_bytes(display_length(
                value.format("%H:%M:%S%.f"),
            )),
            #[cfg(feature = "chrono")]
            ValueRef::DateTime(value) => self.check_string_bytes(
                display_length(value.format("%Y-%m-%dT%H:%M:%S%.f")),
            ),
            #[cfg(feature = "chrono")]
            ValueRef::Instant(value) => self.check_string_bytes(
                display_length(value.format("%Y-%m-%dT%H:%M:%S%.fZ")),
            ),
            ValueRef::Duration(value) => {
                self.check_numeric_bytes(display_length(value.as_secs()))?;
                self.check_numeric_bytes(display_length(value.subsec_nanos()))
            }
            #[cfg(feature = "url")]
            ValueRef::Url(value) => {
                self.check_string_bytes(value.as_str().len())
            }
            ValueRef::Unset(_) | ValueRef::Bool(_) => Ok(()),
        }
    }

    /// Validates one scalar value at an embedding depth.
    ///
    /// Use this method when the scalar is nested inside an outer wire
    /// document. The depth is inclusive and should be supplied by the outer
    /// protocol's accounting traversal.
    ///
    /// # Parameters
    ///
    /// * `value` - Scalar value to validate.
    /// * `depth` - Root-inclusive depth of the scalar in the complete document.
    ///
    /// # Errors
    ///
    /// Returns a resource-limit error when the value exceeds the configured
    /// depth, node, string, or numeric budget.
    #[inline(always)]
    pub fn check_value_at(
        &mut self,
        value: &crate::Value,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_value_ref(value.view(), depth)
    }

    fn check_multi_values_ref(
        &mut self,
        values: MultiValuesRef<'_>,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        macro_rules! check_values {
            ($values:expr) => {{
                for value in $values {
                    self.check_value_ref(value, depth.saturating_add(1))?;
                }
                Ok(())
            }};
        }
        match values {
            MultiValuesRef::Unset(_) => Ok(()),
            MultiValuesRef::Bool(values) => {
                check_values!(values.iter().map(|_| ValueRef::Bool(false)))
            }
            MultiValuesRef::Char(values) => {
                check_values!(values.iter().copied().map(ValueRef::Char))
            }
            MultiValuesRef::Int8(values) => {
                check_values!(values.iter().copied().map(ValueRef::Int8))
            }
            MultiValuesRef::Int16(values) => {
                check_values!(values.iter().copied().map(ValueRef::Int16))
            }
            MultiValuesRef::Int32(values) => {
                check_values!(values.iter().copied().map(ValueRef::Int32))
            }
            MultiValuesRef::Int64(values) => {
                check_values!(values.iter().copied().map(ValueRef::Int64))
            }
            MultiValuesRef::Int128(values) => {
                check_values!(values.iter().copied().map(ValueRef::Int128))
            }
            MultiValuesRef::UInt8(values) => {
                check_values!(values.iter().copied().map(ValueRef::UInt8))
            }
            MultiValuesRef::UInt16(values) => {
                check_values!(values.iter().copied().map(ValueRef::UInt16))
            }
            MultiValuesRef::UInt32(values) => {
                check_values!(values.iter().copied().map(ValueRef::UInt32))
            }
            MultiValuesRef::UInt64(values) => {
                check_values!(values.iter().copied().map(ValueRef::UInt64))
            }
            MultiValuesRef::UInt128(values) => {
                check_values!(values.iter().copied().map(ValueRef::UInt128))
            }
            MultiValuesRef::Float32(values) => {
                check_values!(values.iter().copied().map(ValueRef::Float32))
            }
            MultiValuesRef::Float64(values) => {
                check_values!(values.iter().copied().map(ValueRef::Float64))
            }
            #[cfg(feature = "big-integer")]
            MultiValuesRef::BigInteger(values) => {
                check_values!(values.iter().map(ValueRef::BigInteger))
            }
            #[cfg(feature = "big-decimal")]
            MultiValuesRef::BigDecimal(values) => {
                check_values!(values.iter().map(ValueRef::BigDecimal))
            }
            MultiValuesRef::String(values) => {
                for value in values {
                    self.check_value_ref(
                        ValueRef::String(value),
                        depth.saturating_add(1),
                    )?;
                }
                Ok(())
            }
            #[cfg(feature = "chrono")]
            MultiValuesRef::Date(values) => {
                check_values!(values.iter().map(ValueRef::Date))
            }
            #[cfg(feature = "chrono")]
            MultiValuesRef::Time(values) => {
                check_values!(values.iter().map(ValueRef::Time))
            }
            #[cfg(feature = "chrono")]
            MultiValuesRef::DateTime(values) => {
                check_values!(values.iter().map(ValueRef::DateTime))
            }
            #[cfg(feature = "chrono")]
            MultiValuesRef::Instant(values) => {
                check_values!(values.iter().map(ValueRef::Instant))
            }
            MultiValuesRef::Duration(values) => {
                check_values!(values.iter().map(ValueRef::Duration))
            }
            #[cfg(feature = "url")]
            MultiValuesRef::Url(values) => {
                check_values!(values.iter().map(ValueRef::Url))
            }
            MultiValuesRef::StringMap(values) => {
                check_values!(values.iter().map(ValueRef::StringMap))
            }
            #[cfg(feature = "json")]
            MultiValuesRef::Json(values) => {
                check_values!(values.iter().map(ValueRef::Json))
            }
        }
    }

    #[cfg(feature = "json")]
    fn check_json(
        &mut self,
        value: &serde_json::Value,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        match value {
            serde_json::Value::Array(values) => {
                self.check_collection_items(values.len())?;
                for value in values {
                    self.check_json(value, depth.saturating_add(1))?;
                }
                Ok(())
            }
            serde_json::Value::Object(values) => {
                self.check_map_entries(values.len())?;
                for (key, value) in values {
                    self.check_string_bytes(key.len())?;
                    self.check_json(value, depth.saturating_add(1))?;
                }
                Ok(())
            }
            serde_json::Value::String(value) => {
                self.check_string_bytes(value.len())
            }
            serde_json::Value::Number(value) => {
                self.check_numeric_bytes(display_length(value))
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) => Ok(()),
        }
    }

    fn check_limit(
        &self,
        kind: ValueWireLimitKind,
        value: usize,
        maximum: usize,
    ) -> Result<(), ValueWireDecodeError> {
        if value > maximum {
            Err(ValueWireDecodeError::LimitExceeded {
                kind,
                value,
                maximum,
            })
        } else {
            Ok(())
        }
    }
}

/// Returns the bounded V1 decimal payload length without expanding `scale`.
///
/// V1 represents a decimal as its canonical integer coefficient plus a
/// separately bounded scale. Formatting the decimal itself can expand a large
/// negative scale into an arbitrarily long string, which is unrelated to the
/// encoded coefficient size.
#[cfg(feature = "big-decimal")]
#[inline]
fn big_decimal_numeric_len(value: &bigdecimal::BigDecimal) -> usize {
    let (coefficient, _) = value.as_bigint_and_scale();
    display_length(coefficient.as_ref())
}

/// Parses JSON with input-bounded traversal without materializing a JSON tree
/// or a wire DTO.
struct JsonPreflightSeed {
    limits: WireLimits,
    nodes: usize,
    violation: Option<ValueWireDecodeError>,
}

impl JsonPreflightSeed {
    #[inline(always)]
    fn new(input_bytes: usize) -> Self {
        // Every JSON node and decoded scalar must occupy at least one input
        // byte. Using the complete input length as the syntax-traversal ceiling
        // avoids guessing how many wrapper nodes an outer protocol contributes;
        // exact semantic limits are enforced after runtime decoding.
        let limits = WireLimits {
            max_input_bytes: input_bytes,
            max_depth: input_bytes,
            max_nodes: input_bytes,
            max_collection_items: input_bytes,
            max_map_entries: input_bytes,
            max_string_bytes: input_bytes,
            max_numeric_bytes: input_bytes,
        };
        Self {
            limits,
            nodes: 0,
            violation: None,
        }
    }

    #[inline]
    fn check_limit<E>(
        &mut self,
        kind: ValueWireLimitKind,
        value: usize,
        maximum: usize,
    ) -> Result<(), E>
    where
        E: DeError,
    {
        if value > maximum {
            self.violation = Some(ValueWireDecodeError::LimitExceeded {
                kind,
                value,
                maximum,
            });
            Err(E::custom(format_args!(
                "wire input {kind:?} value {value} exceeds the limit of {maximum}"
            )))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn check_node<E>(&mut self) -> Result<(), E>
    where
        E: DeError,
    {
        self.nodes = self.nodes.saturating_add(1);
        self.check_limit(
            ValueWireLimitKind::Nodes,
            self.nodes,
            self.limits.max_nodes,
        )
    }

    #[inline]
    fn check_depth<E>(&mut self, depth: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.check_limit(
            ValueWireLimitKind::Depth,
            depth,
            self.limits.max_depth,
        )
    }

    #[inline]
    fn check_collection_items<E>(&mut self, items: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.check_limit(
            ValueWireLimitKind::CollectionItems,
            items,
            self.limits.max_collection_items,
        )
    }

    #[inline]
    fn check_map_entries<E>(&mut self, entries: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.check_limit(
            ValueWireLimitKind::MapEntries,
            entries,
            self.limits.max_map_entries,
        )
    }

    #[inline]
    fn check_string_bytes<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.check_limit(
            ValueWireLimitKind::StringBytes,
            bytes,
            self.limits.max_string_bytes,
        )
    }

    #[inline]
    fn check_numeric_bytes<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.check_limit(
            ValueWireLimitKind::NumericBytes,
            bytes,
            self.limits.max_numeric_bytes,
        )
    }
}

impl<'de> DeserializeSeed<'de> for &mut JsonPreflightSeed {
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self,
            depth: 1,
        })
    }
}

struct JsonPreflightVisitor<'a> {
    preflight: &'a mut JsonPreflightSeed,
    depth: usize,
}

impl JsonPreflightVisitor<'_> {
    #[inline]
    fn scalar<E>(&mut self) -> Result<(), E>
    where
        E: DeError,
    {
        self.preflight.check_depth(self.depth)?;
        self.preflight.check_node()
    }

    #[inline]
    fn string<E>(&mut self, value: &str) -> Result<(), E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.check_string_bytes(value.len())
    }

    #[inline]
    fn number<E>(&mut self, bytes: usize) -> Result<(), E>
    where
        E: DeError,
    {
        self.scalar()?;
        self.preflight.check_numeric_bytes(bytes)
    }
}

impl<'de> Visitor<'de> for JsonPreflightVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(mut self, _value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_i64<E>(mut self, value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(display_length(value))
    }

    fn visit_u64<E>(mut self, value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(display_length(value))
    }

    fn visit_f64<E>(mut self, value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.number(display_length(value))
    }

    fn visit_unit<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_none<E>(mut self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.scalar()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        (&mut *self.preflight).deserialize(deserializer)
    }

    fn visit_borrowed_str<E>(
        mut self,
        value: &'de str,
    ) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    fn visit_str<E>(mut self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(value)
    }

    fn visit_string<E>(mut self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        self.string(&value)
    }

    fn visit_seq<A>(mut self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.scalar()?;
        let mut items: usize = 0;
        while access
            .next_element_seed(JsonPreflightChildSeed {
                preflight: self.preflight,
                depth: self.depth.saturating_add(1),
            })?
            .is_some()
        {
            items = items.saturating_add(1);
            self.preflight.check_collection_items(items)?;
        }
        Ok(())
    }

    fn visit_map<A>(mut self, mut access: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.scalar()?;
        let Some(first_key) = access.next_key::<String>()? else {
            return Ok(());
        };
        self.preflight.check_string_bytes(first_key.len())?;

        let mut entries = 1_usize;
        self.preflight.check_map_entries(entries)?;
        if first_key == JSON_NUMBER_TOKEN {
            let number_text = access.next_value::<String>()?;
            let mut next_key = access.next_key::<String>()?;
            if next_key.is_none() {
                return self.preflight.check_numeric_bytes(number_text.len());
            }
            // The marker key was part of a real object. Its first value is a
            // normal JSON string; the already-read extra key is accounted for
            // by the loop below.
            self.preflight.check_depth(self.depth.saturating_add(1))?;
            self.preflight.check_node()?;
            self.preflight.check_string_bytes(number_text.len())?;
            while let Some(key) = next_key.take() {
                entries = entries.saturating_add(1);
                self.preflight.check_map_entries(entries)?;
                self.preflight.check_string_bytes(key.len())?;
                access.next_value_seed(JsonPreflightChildSeed {
                    preflight: self.preflight,
                    depth: self.depth.saturating_add(1),
                })?;
                next_key = access.next_key::<String>()?;
            }
            return Ok(());
        }
        access.next_value_seed(JsonPreflightChildSeed {
            preflight: self.preflight,
            depth: self.depth.saturating_add(1),
        })?;
        while let Some(key) = access.next_key::<String>()? {
            entries = entries.saturating_add(1);
            self.preflight.check_map_entries(entries)?;
            self.preflight.check_string_bytes(key.len())?;
            access.next_value_seed(JsonPreflightChildSeed {
                preflight: self.preflight,
                depth: self.depth.saturating_add(1),
            })?;
        }
        Ok(())
    }
}

struct JsonPreflightChildSeed<'a> {
    preflight: &'a mut JsonPreflightSeed,
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for JsonPreflightChildSeed<'_> {
    type Value = ();

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(JsonPreflightVisitor {
            preflight: self.preflight,
            depth: self.depth,
        })
    }
}
