// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared resource limits and accounting for JSON wire decoding.

use super::{ValueWireDecodeError, ValueWireLimitKind};
use crate::{MultiValuesRef, ValueContainer, ValueRef};

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
    max_numeric_digits: usize,
}

impl WireLimits {
    /// Default maximum complete JSON input length.
    pub const DEFAULT_MAX_INPUT_BYTES: usize = 1_048_576;
    /// Compatibility name for the default complete JSON input length.
    pub const DEFAULT_MAX_JSON_BYTES: usize = Self::DEFAULT_MAX_INPUT_BYTES;
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
    /// Default maximum digits in one decoded number.
    pub const DEFAULT_MAX_NUMERIC_DIGITS: usize = 4_096;

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
            max_numeric_digits: Self::DEFAULT_MAX_NUMERIC_DIGITS,
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
    pub const fn with_max_collection_items(mut self, max_collection_items: usize) -> Self {
        self.max_collection_items = max_collection_items;
        self
    }

    /// Sets the maximum entries in one map.
    #[inline(always)]
    #[must_use = "the configured map limit should be used"]
    pub const fn with_max_map_entries(mut self, max_map_entries: usize) -> Self {
        self.max_map_entries = max_map_entries;
        self
    }

    /// Sets the maximum bytes in one decoded string.
    #[inline(always)]
    #[must_use = "the configured string limit should be used"]
    pub const fn with_max_string_bytes(mut self, max_string_bytes: usize) -> Self {
        self.max_string_bytes = max_string_bytes;
        self
    }

    /// Sets the maximum digits in one decoded number.
    #[inline(always)]
    #[must_use = "the configured numeric limit should be used"]
    pub const fn with_max_numeric_digits(mut self, max_numeric_digits: usize) -> Self {
        self.max_numeric_digits = max_numeric_digits;
        self
    }

    /// Returns the maximum complete input length.
    #[must_use]
    #[inline(always)]
    pub const fn max_input_bytes(self) -> usize {
        self.max_input_bytes
    }

    /// Returns the maximum complete JSON input length.
    #[must_use]
    #[inline(always)]
    pub const fn max_json_bytes(self) -> usize {
        self.max_input_bytes()
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

    /// Returns the maximum digits in one decoded number.
    #[must_use]
    #[inline(always)]
    pub const fn max_numeric_digits(self) -> usize {
        self.max_numeric_digits
    }

    /// Checks a complete input length and starts a shared accounting session.
    #[inline]
    pub fn begin(self, input_bytes: usize) -> Result<WireBudget, ValueWireDecodeError> {
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

    /// Checks a complete input length without starting an accounting session.
    #[inline]
    pub const fn check_json_bytes(self, input_bytes: usize) -> Result<(), ValueWireDecodeError> {
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
#[derive(Debug, Clone, Copy)]
pub struct WireBudget {
    limits: WireLimits,
    nodes: usize,
}

impl WireBudget {
    /// Returns the configured limits for this session.
    #[inline(always)]
    pub const fn limits(self) -> WireLimits {
        self.limits
    }

    /// Charges one decoded node.
    #[inline]
    pub fn check_node(&mut self) -> Result<(), ValueWireDecodeError> {
        self.nodes = self.nodes.saturating_add(1);
        self.check_limit(ValueWireLimitKind::Nodes, self.nodes, self.limits.max_nodes)
    }

    /// Checks a recursive depth.
    #[inline]
    pub fn check_depth(&self, depth: usize) -> Result<(), ValueWireDecodeError> {
        self.check_limit(ValueWireLimitKind::Depth, depth, self.limits.max_depth)
    }

    /// Checks one collection length.
    #[inline]
    pub fn check_collection_items(&self, items: usize) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::CollectionItems,
            items,
            self.limits.max_collection_items,
        )
    }

    /// Checks one map length.
    #[inline]
    pub fn check_map_entries(&self, entries: usize) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::MapEntries,
            entries,
            self.limits.max_map_entries,
        )
    }

    /// Checks one decoded string length.
    #[inline]
    pub fn check_string_bytes(&self, bytes: usize) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::StringBytes,
            bytes,
            self.limits.max_string_bytes,
        )
    }

    /// Checks one decoded numeric length.
    #[inline]
    pub fn check_numeric_digits(&self, digits: usize) -> Result<(), ValueWireDecodeError> {
        self.check_limit(
            ValueWireLimitKind::NumericDigits,
            digits,
            self.limits.max_numeric_digits,
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
            ValueContainer::Scalar(value) => self.check_value_ref(value.view(), depth),
            ValueContainer::Collection(values) => {
                self.check_node()?;
                self.check_collection_items(values.len())?;
                self.check_multi_values(values.view(), depth)
            }
        }
    }

    /// Validates one scalar Value against the shared budget.
    #[inline]
    pub fn check_value(&mut self, value: &crate::Value) -> Result<(), ValueWireDecodeError> {
        self.check_value_ref(value.view(), 1)
    }

    fn check_value_ref(
        &mut self,
        value: ValueRef<'_>,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        self.check_depth(depth)?;
        self.check_node()?;
        match value {
            ValueRef::String(value) => self.check_string_bytes(value.len()),
            ValueRef::StringMap(value) => {
                self.check_map_entries(value.len())?;
                for (key, value) in value {
                    self.check_string_bytes(key.len())?;
                    self.check_string_bytes(value.len())?;
                }
                Ok(())
            }
            #[cfg(feature = "json")]
            ValueRef::Json(value) => self.check_json(value, depth + 1),
            #[cfg(feature = "big-integer")]
            ValueRef::BigInteger(value) => self.check_numeric_digits(value.to_string().len()),
            #[cfg(feature = "big-decimal")]
            ValueRef::BigDecimal(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Int8(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Int16(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Int32(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Int64(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Int128(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::UInt8(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::UInt16(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::UInt32(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::UInt64(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::UInt128(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Float32(value) => self.check_numeric_digits(value.to_string().len()),
            ValueRef::Float64(value) => self.check_numeric_digits(value.to_string().len()),
            _ => Ok(()),
        }
    }

    fn check_multi_values(
        &mut self,
        values: MultiValuesRef<'_>,
        depth: usize,
    ) -> Result<(), ValueWireDecodeError> {
        macro_rules! check_values {
            ($values:expr) => {{
                for value in $values {
                    self.check_value_ref(value, depth + 1)?;
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
                check_values!(values.iter().map(|_| ValueRef::Char('\0')))
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
                    self.check_value_ref(ValueRef::String(value), depth + 1)?;
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
                    self.check_json(value, depth + 1)?;
                }
                Ok(())
            }
            serde_json::Value::Object(values) => {
                self.check_map_entries(values.len())?;
                for (key, value) in values {
                    self.check_string_bytes(key.len())?;
                    self.check_json(value, depth + 1)?;
                }
                Ok(())
            }
            serde_json::Value::String(value) => self.check_string_bytes(value.len()),
            serde_json::Value::Number(value) => self.check_numeric_digits(value.to_string().len()),
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

/// Compatibility alias for the former value-only limit type.
pub type ValueWireLimits = WireLimits;
