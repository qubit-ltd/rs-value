// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializable fixtures that report pathological compound length hints.

use serde::ser::{
    SerializeMap,
    SerializeSeq,
    SerializeStructVariant,
    SerializeTupleVariant,
};
use serde::{
    Serialize,
    Serializer,
};

/// Selects a compound Serde shape that declares an oversized length hint.
pub(super) enum OversizedLengthHint {
    /// An empty sequence with an oversized optional length hint.
    Sequence,
    /// An empty tuple variant with an oversized required length hint.
    TupleVariant,
    /// An empty map with an oversized optional length hint.
    Map,
    /// An empty struct variant with an oversized required length hint.
    StructVariant,
}

impl Serialize for OversizedLengthHint {
    /// Serializes the selected empty compound shape with a pathological hint.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Sequence => serializer.serialize_seq(Some(usize::MAX))?.end(),
            Self::TupleVariant => serializer
                .serialize_tuple_variant(
                    "OversizedLengthHint",
                    0,
                    "Tuple",
                    usize::MAX,
                )?
                .end(),
            Self::Map => serializer.serialize_map(Some(usize::MAX))?.end(),
            Self::StructVariant => serializer
                .serialize_struct_variant(
                    "OversizedLengthHint",
                    1,
                    "Struct",
                    usize::MAX,
                )?
                .end(),
        }
    }
}
