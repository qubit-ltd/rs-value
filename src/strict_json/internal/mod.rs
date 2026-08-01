// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal serializers composing the strict JSON value serializer.

mod map_key_serializer;
mod object_serializer;
mod sequence_serializer;
mod json_number_serializer;
mod strict_json_error;
mod strict_json_serializer;
mod struct_serializer;
mod struct_variant_serializer;
mod tuple_variant_serializer;

pub(in crate::strict_json) use map_key_serializer::MapKeySerializer;
pub(in crate::strict_json) use object_serializer::ObjectSerializer;
pub(in crate::strict_json) use sequence_serializer::SequenceSerializer;
pub(in crate::strict_json) use struct_serializer::StructSerializer;
pub(crate) use strict_json_error::StrictJsonError;
pub(in crate::strict_json) use strict_json_serializer::StrictJsonSerializer;
pub(in crate::strict_json) use struct_variant_serializer::StructVariantSerializer;
pub(in crate::strict_json) use tuple_variant_serializer::TupleVariantSerializer;
