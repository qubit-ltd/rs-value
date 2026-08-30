// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Multiple Values Container
//!
//! Provides type-safe storage and access functionality for multiple values.
// qubit-style: allow source-test-pair
// Tests are intentionally distributed across behavior-specific files under
// tests/multi_values/ rather than collected in multi_values_tests.rs.
// qubit-style: allow multiple-public-types
use std::fmt;
#[cfg(feature = "json")]
use std::hash::Hash;
#[cfg(feature = "json")]
use std::hash::Hasher;

#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::ResourceQuantity;
#[cfg(feature = "json")]
use qubit_budget::json::JsonValueBudget;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::ConversionLimits;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataType;

#[cfg(feature = "json")]
use super::multi_values_identity::hash_multi_values_payload_with_json_budget;
use super::multi_values_ref::MultiValuesRef;
#[cfg(all(feature = "converter", feature = "json"))]
use crate::ValueResult;
#[cfg(feature = "json")]
use crate::identity::hash_json;
#[cfg(feature = "json")]
use crate::identity::preflight_json;

/// Defines the private storage representation for the public multi-value
/// container from the shared value-type table.
macro_rules! define_multi_values_enum {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        /// Internal multiple-values representation.
        ///
        /// Uses an enum to represent multiple values of different types,
        /// providing type-safe storage and access for multiple values.
        ///
        /// This representation is private; downstream code uses
        /// [`MultiValues`] constructors and [`MultiValuesRef`] semantic views
        /// instead of matching storage details.
        ///
        /// # Behavior
        ///
        /// - Stores a homogeneous collection from the closed [`DataType`]
        ///   family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from a concrete empty vector.
        ///
        /// # Equality and hashing
        ///
        /// Equality preserves the collection variant and element order. Float
        /// elements use canonical signed-zero and NaN identity, while map-like
        /// elements hash structurally. Standard hash output is suitable for in-memory
        /// keys but is not a stable persistent fingerprint.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
        /// assert_eq!(values.len(), 3);
        /// assert_eq!(values.get_first_int32().unwrap(), 1);
        ///
        /// let all = values.get_int32s().unwrap();
        /// assert_eq!(all, &[1, 2, 3]);
        ///
        /// values.add(4).unwrap();
        /// assert_eq!(values.len(), 4);
        /// ```
        #[derive(Debug, Clone)]
        pub(crate) enum MultiValuesRepr {
            /// Unset collection with a declared element data type.
            Unset(
                /// Declared element type retained while the collection is unset.
                DataType,
            ),
            $(
                #[doc = $multi_doc]
                $(#[$cfg])*
                $variant(
                    #[doc = concat!("Stored ", $multi_doc, " payload.")]
                    Vec<$type>,
                ),
            )+
        }
    };
}

for_each_value_type!(define_multi_values_enum);

/// Multiple typed runtime values with private storage representation.
///
/// # Examples
///
/// ```
/// use qubit_value::MultiValues;
///
/// let values = MultiValues::from(vec![1_i32, 2, 3]);
/// assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3]);
/// ```
#[must_use]
#[derive(Clone)]
pub struct MultiValues {
    /// Private homogeneous storage backing the stable public accessor API.
    pub(crate) repr: MultiValuesRepr,
}

impl fmt::Debug for MultiValues {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.view().fmt(formatter)
    }
}

/// Implements named collection constructors from the shared value table.
macro_rules! impl_multi_values_constructors {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        impl MultiValues {
            /// Creates an unset collection with an explicit element type.
            ///
            /// # Parameters
            ///
            /// * `data_type` - Element type retained while the collection is unset.
            ///
            /// # Returns
            ///
            /// An unset collection retaining `data_type`.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub const fn Unset(data_type: DataType) -> Self {
                Self::new_unset(data_type)
            }

            /// Creates an unset collection with an explicit element type.
            ///
            /// # Parameters
            ///
            /// * `data_type` - Element type retained while the collection is unset.
            ///
            /// # Returns
            ///
            /// An unset collection retaining `data_type`.
            #[inline(always)]
            pub const fn new_unset(data_type: DataType) -> Self {
                Self { repr: MultiValuesRepr::Unset(data_type) }
            }

            $(
                #[doc = concat!("Creates a collection of ", $multi_doc, ".")]
                ///
                /// # Parameters
                ///
                /// * `values` - Homogeneous elements stored by the collection.
                ///
                /// # Returns
                ///
                /// A typed collection containing `values` in their original order.
                $(#[$cfg])*
                #[allow(non_snake_case)]
                #[inline(always)]
                pub fn $variant(values: Vec<$type>) -> Self {
                    Self { repr: MultiValuesRepr::$variant(values) }
                }
            )+
        }
    };
}

for_each_value_type!(impl_multi_values_constructors);

impl MultiValues {
    /// Hashes this collection while applying `budget` to JSON elements.
    ///
    /// # Type Parameters
    ///
    /// * `H` - Hasher receiving the semantic collection identity.
    /// * `R` - Resource identifier used by the JSON budget.
    /// * `Q` - Quantity type used by the JSON budget.
    ///
    /// # Parameters
    ///
    /// * `state` - Hasher that receives the same identity representation as
    ///   [`Hash::hash`].
    /// * `budget` - Mutable JSON traversal budget, used only for JSON elements.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete semantic identity is hashed.
    ///
    /// # Errors
    ///
    /// Returns [`qubit_budget::MeasuredBudgetError`] when a JSON element
    /// exceeds a configured limit.
    /// On error, neither `state` nor the committed portion of `budget` is
    /// modified. A hasher panic also drops the staged budget transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// use qubit_budget::{ResourceLimit, StructureLimits};
    /// use qubit_budget::json::{JsonResource, JsonValueBudget, JsonValueLimits};
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Json(vec![serde_json::json!([null])]);
    /// let structure = StructureLimits::<JsonResource, usize>::builder().nodes_limit(
    ///     ResourceLimit::new(JsonResource::Nodes, 1_usize),
    /// ).build();
    /// let mut budget = JsonValueBudget::new(
    ///     JsonValueLimits::builder().structure_limits(structure).build(),
    /// );
    /// let mut hasher = DefaultHasher::new();
    ///
    /// assert!(values.hash_with_json_budget(&mut hasher, &mut budget).is_err());
    /// drop(hasher);
    /// // The rejected values did not consume committed budget state.
    /// ```
    #[cfg(feature = "json")]
    pub fn hash_with_json_budget<H, R, Q>(
        &self,
        state: &mut H,
        budget: &mut JsonValueBudget<R, Q>,
    ) -> Result<(), MeasuredBudgetError<R, Q>>
    where
        H: Hasher,
        R: Clone,
        Q: ResourceQuantity,
    {
        match &self.repr {
            MultiValuesRepr::Json(values) => {
                let mut transaction = budget.transaction();
                for value in values {
                    preflight_json(value, &mut transaction)?;
                }
                std::mem::discriminant(&self.repr).hash(state);
                values.len().hash(state);
                for value in values {
                    hash_json(value, state);
                }
                transaction.commit();
                Ok(())
            }
            _ => {
                std::mem::discriminant(&self.repr).hash(state);
                hash_multi_values_payload_with_json_budget(&self.repr, state, budget)
            }
        }
    }

    /// Borrows the stable semantic view of this collection.
    ///
    /// # Returns
    ///
    /// A non-owning homogeneous view that hides private storage details.
    #[must_use = "the borrowed collection view should be used"]
    #[inline(always)]
    pub fn view(&self) -> MultiValuesRef<'_> {
        match &self.repr {
            MultiValuesRepr::Unset(data_type) => MultiValuesRef::Unset(*data_type),
            MultiValuesRepr::Bool(values) => MultiValuesRef::Bool(values),
            MultiValuesRepr::Char(values) => MultiValuesRef::Char(values),
            MultiValuesRepr::Int8(values) => MultiValuesRef::Int8(values),
            MultiValuesRepr::Int16(values) => MultiValuesRef::Int16(values),
            MultiValuesRepr::Int32(values) => MultiValuesRef::Int32(values),
            MultiValuesRepr::Int64(values) => MultiValuesRef::Int64(values),
            MultiValuesRepr::Int128(values) => MultiValuesRef::Int128(values),
            MultiValuesRepr::UInt8(values) => MultiValuesRef::UInt8(values),
            MultiValuesRepr::UInt16(values) => MultiValuesRef::UInt16(values),
            MultiValuesRepr::UInt32(values) => MultiValuesRef::UInt32(values),
            MultiValuesRepr::UInt64(values) => MultiValuesRef::UInt64(values),
            MultiValuesRepr::UInt128(values) => MultiValuesRef::UInt128(values),
            MultiValuesRepr::Float32(values) => MultiValuesRef::Float32(values),
            MultiValuesRepr::Float64(values) => MultiValuesRef::Float64(values),
            #[cfg(feature = "big-integer")]
            MultiValuesRepr::BigInteger(values) => MultiValuesRef::BigInteger(values),
            #[cfg(feature = "big-decimal")]
            MultiValuesRepr::BigDecimal(values) => MultiValuesRef::BigDecimal(values),
            MultiValuesRepr::String(values) => MultiValuesRef::String(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Date(values) => MultiValuesRef::Date(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Time(values) => MultiValuesRef::Time(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::DateTime(values) => MultiValuesRef::DateTime(values),
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Instant(values) => MultiValuesRef::Instant(values),
            MultiValuesRepr::Duration(values) => MultiValuesRef::Duration(values),
            #[cfg(feature = "url")]
            MultiValuesRepr::Url(values) => MultiValuesRef::Url(values),
            MultiValuesRepr::StringMap(values) => MultiValuesRef::StringMap(values),
            #[cfg(feature = "json")]
            MultiValuesRepr::Json(values) => MultiValuesRef::Json(values),
        }
    }
}

// ============================================================================
// Getter method generation macros
// ============================================================================

/// Unified multiple values getter generation macro
///
/// Generates `get_[xxx]s` methods for `MultiValues`, returning a reference to
/// value slices.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
macro_rules! impl_get_multi_values {
    // Simple type: return slice reference
    ($(#[$attr:meta])* slice: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the container is unset"]
        #[doc = "with the requested type, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs. A concrete empty vector returns"]
        #[doc = "an empty slice."]
        #[must_use = "the strict collection read result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match &self.repr {
                MultiValuesRepr::$variant(v) => Ok(v),
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetCollection {
                        data_type: *dt,
                    }))
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Complex type: return Vec reference (e.g., Vec<String>, Vec<Vec<u8>>)
    ($(#[$attr:meta])* vec: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the container is unset"]
        #[doc = "with the requested type, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs. A concrete empty vector returns"]
        #[doc = "an empty slice."]
        #[must_use = "the strict collection read result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<&[$type]> {
            match &self.repr {
                MultiValuesRepr::$variant(v) => Ok(v.as_slice()),
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetCollection {
                        data_type: *dt,
                    }))
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

/// Unified multiple values get_first method generation macro
///
/// Generates `get_first_[xxx]` methods for `MultiValues`, used to get the first
/// value.
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so you
/// can add `///` comments before macro invocations.
macro_rules! impl_get_first_value {
    // Copy type: directly return value
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the requested type matches"]
        #[doc = "but no value is stored, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs."]
        #[must_use = "the strict first-value result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$type> {
            match &self.repr {
                MultiValuesRepr::$variant(v) if !v.is_empty() => Ok(v[0]),
                MultiValuesRepr::$variant(_) => {
                    Err(ValueError::Missing($crate::ValueMissing::EmptyCollection {
                        data_type: $data_type,
                    }))
                }
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetCollection {
                        data_type: *dt,
                    }))
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Reference type: return reference
    ($(#[$attr:meta])* ref: $method:ident, $variant:ident, $ret_type:ty, $data_type:expr, $conversion:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the requested type matches"]
        #[doc = "but no value is stored, or [`ValueError::TypeMismatch`] when"]
        #[doc = "the stored data type differs."]
        #[must_use = "the strict first-value result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match &self.repr {
                MultiValuesRepr::$variant(v) if !v.is_empty() => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(&v[0]))
                },
                MultiValuesRepr::$variant(_) => {
                    Err(ValueError::Missing($crate::ValueMissing::EmptyCollection {
                        data_type: $data_type,
                    }))
                }
                MultiValuesRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetCollection {
                        data_type: *dt,
                    }))
                }
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

#[cfg(all(feature = "converter", feature = "json"))]
impl MultiValues {
    /// Projects this collection to its natural JSON representation.
    ///
    /// Unset is `null`; every concrete collection is an array, including empty
    /// and one-item collections.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this collection.
    ///
    /// # Errors
    ///
    /// Returns a list conversion error containing the zero-based source index
    /// when an item cannot be represented as JSON.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<serde_json::Value> {
        self.to_json_value_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Projects this collection using explicit conversion policy and limits.
    ///
    /// # Parameters
    ///
    /// * `policy` - Controls duration units and precision-loss behavior.
    /// * `limits` - Bounds conversion resource consumption.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this collection.
    ///
    /// # Errors
    ///
    /// Returns an indexed list conversion error when an item cannot be
    /// represented under the requested policy and limits.
    #[inline(always)]
    pub fn to_json_value_with(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<serde_json::Value> {
        crate::json::multi_values_to_json_value_with(self, policy, limits)
    }
}

// Implements generic construction, strict reads, mutation, and state queries.
#[path = "multi_values_core.rs"]
mod multi_values_core;
// Implements policy-driven conversions through `qubit-datatype`.
#[cfg(feature = "converter")]
#[path = "multi_values_converters.rs"]
mod multi_values_converters;
// Implements type-specific strict getters.
#[path = "multi_values_getters.rs"]
mod multi_values_getters;
