// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Fluent helpers for constructing JSON value limits in integration tests.

#![cfg(feature = "json")]

use qubit_budget::ResourceLimit;
use qubit_json::JsonDecodeLimits;
use qubit_json::JsonEncodeLimits;
use qubit_json::JsonResource;
use qubit_json::JsonValueBudget;
use qubit_json::JsonValueLimits;

/// Test-only fluent builders for direction-independent JSON value limits.
pub(crate) trait JsonValueLimitsExt {
    /// Configures the root-inclusive nesting-depth maximum.
    fn with_max_depth(self, maximum: usize) -> Self;

    /// Configures the cumulative JSON-node maximum.
    fn with_max_nodes(self, maximum: usize) -> Self;

    /// Configures the per-array item maximum.
    fn with_max_sequence_items(self, maximum: usize) -> Self;

    /// Configures the per-object entry maximum.
    fn with_max_map_entries(self, maximum: usize) -> Self;

    /// Configures the per-key UTF-8 byte maximum.
    fn with_max_key_bytes(self, maximum: usize) -> Self;

    /// Configures the per-string UTF-8 byte maximum.
    fn with_max_string_bytes(self, maximum: usize) -> Self;

    /// Configures the per-number text byte maximum.
    fn with_max_number_bytes(self, maximum: usize) -> Self;

    /// Creates fresh mutable accounting from these limits.
    fn budget(self) -> JsonValueBudget;
}

/// Test-only fluent builders for JSON decode limits.
pub(crate) trait JsonDecodeLimitsExt {
    /// Configures the cumulative input-byte maximum.
    fn with_max_input_bytes(self, maximum: usize) -> Self;

    /// Configures the cumulative JSON-node maximum.
    fn with_max_nodes(self, maximum: usize) -> Self;

    /// Configures the per-string UTF-8 byte maximum.
    fn with_max_string_bytes(self, maximum: usize) -> Self;
}

impl JsonDecodeLimitsExt for JsonDecodeLimits {
    fn with_max_input_bytes(self, maximum: usize) -> Self {
        self.with_input_bytes_limit(ResourceLimit::new(
            JsonResource::InputBytes,
            maximum,
        ))
    }

    fn with_max_nodes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_nodes(maximum);
        self.with_value_limits(value)
    }

    fn with_max_string_bytes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_string_bytes(maximum);
        self.with_value_limits(value)
    }
}

/// Test-only fluent builders for JSON encode limits.
pub(crate) trait JsonEncodeLimitsExt {
    /// Configures the cumulative output-byte maximum.
    fn with_max_output_bytes(self, maximum: usize) -> Self;

    /// Configures the root-inclusive nesting-depth maximum.
    fn with_max_depth(self, maximum: usize) -> Self;

    /// Configures the cumulative JSON-node maximum.
    fn with_max_nodes(self, maximum: usize) -> Self;

    /// Configures the per-array item maximum.
    fn with_max_sequence_items(self, maximum: usize) -> Self;

    /// Configures the per-object entry maximum.
    fn with_max_map_entries(self, maximum: usize) -> Self;

    /// Configures the per-key UTF-8 byte maximum.
    fn with_max_key_bytes(self, maximum: usize) -> Self;

    /// Configures the per-string UTF-8 byte maximum.
    fn with_max_string_bytes(self, maximum: usize) -> Self;

    /// Configures the per-number text byte maximum.
    fn with_max_number_bytes(self, maximum: usize) -> Self;
}

impl JsonEncodeLimitsExt for JsonEncodeLimits {
    fn with_max_output_bytes(self, maximum: usize) -> Self {
        self.with_output_bytes_limit(ResourceLimit::new(
            JsonResource::OutputBytes,
            maximum,
        ))
    }

    fn with_max_depth(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_depth(maximum);
        self.with_value_limits(value)
    }

    fn with_max_nodes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_nodes(maximum);
        self.with_value_limits(value)
    }

    fn with_max_sequence_items(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_sequence_items(maximum);
        self.with_value_limits(value)
    }

    fn with_max_map_entries(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_map_entries(maximum);
        self.with_value_limits(value)
    }

    fn with_max_key_bytes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_key_bytes(maximum);
        self.with_value_limits(value)
    }

    fn with_max_string_bytes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_string_bytes(maximum);
        self.with_value_limits(value)
    }

    fn with_max_number_bytes(self, maximum: usize) -> Self {
        let value = self.value_limits().with_max_number_bytes(maximum);
        self.with_value_limits(value)
    }
}

impl JsonValueLimitsExt for JsonValueLimits {
    fn with_max_depth(self, maximum: usize) -> Self {
        let structure = self
            .structure_limits()
            .with_depth_limit(ResourceLimit::new(JsonResource::Depth, maximum));
        self.with_structure_limits(structure)
    }

    fn with_max_nodes(self, maximum: usize) -> Self {
        let structure = self
            .structure_limits()
            .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, maximum));
        self.with_structure_limits(structure)
    }

    fn with_max_sequence_items(self, maximum: usize) -> Self {
        let structure = self.structure_limits().with_sequence_items_limit(
            ResourceLimit::new(JsonResource::SequenceItems, maximum),
        );
        self.with_structure_limits(structure)
    }

    fn with_max_map_entries(self, maximum: usize) -> Self {
        let structure =
            self.structure_limits()
                .with_map_entries_limit(ResourceLimit::new(
                    JsonResource::MapEntries,
                    maximum,
                ));
        self.with_structure_limits(structure)
    }

    fn with_max_key_bytes(self, maximum: usize) -> Self {
        let structure =
            self.structure_limits()
                .with_key_bytes_limit(ResourceLimit::new(
                    JsonResource::KeyBytes,
                    maximum,
                ));
        self.with_structure_limits(structure)
    }

    fn with_max_string_bytes(self, maximum: usize) -> Self {
        self.with_string_bytes_limit(ResourceLimit::new(
            JsonResource::StringBytes,
            maximum,
        ))
    }

    fn with_max_number_bytes(self, maximum: usize) -> Self {
        self.with_number_bytes_limit(ResourceLimit::new(
            JsonResource::NumberBytes,
            maximum,
        ))
    }

    fn budget(self) -> JsonValueBudget {
        JsonValueBudget::new(self)
    }
}
