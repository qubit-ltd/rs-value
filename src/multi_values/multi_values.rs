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
use std::collections::HashMap;
use std::fmt;
#[cfg(feature = "json")]
use std::hash::Hash;
#[cfg(feature = "json")]
use std::hash::Hasher;
use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::DateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "chrono")]
use chrono::NaiveDateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveTime;
#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::ResourceQuantity;
#[cfg(feature = "json")]
use qubit_budget::json::JsonValueBudget;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionLimits;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionPolicy;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionSession;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionError;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionTarget;
#[cfg(feature = "converter")]
use qubit_datatype::DataConverter;
#[cfg(feature = "converter")]
use qubit_datatype::DataConverters;
use qubit_datatype::DataType;
#[cfg(feature = "url")]
use url::Url;

use super::internal::MultiValuesRepr;
#[cfg(feature = "json")]
use super::multi_values_identity::hash_multi_values_payload_with_json_budget;
use super::multi_values_ref::MultiValuesRef;
use crate::IntoValueDefault;
use crate::Value;
use crate::ValueError;
use crate::ValueResult;
#[cfg(feature = "json")]
use crate::identity::hash_json;
#[cfg(feature = "json")]
use crate::identity::preflight_json;
use crate::value::ValueRepr;

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
                $(, $_wire:tt)*
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
                transaction.commit()
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
        self.to_json_value_with(
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
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

/// Maps private collection storage variants to their runtime data types.
macro_rules! multi_values_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(dt) => *dt,
            $($(#[$cfg])* MultiValuesRepr::$variant(_) => $data_type,)+
        }
    };
}

/// Returns the concrete element count for each collection storage variant.
macro_rules! multi_values_count_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(_) => 0,
            $($(#[$cfg])* MultiValuesRepr::$variant(values) => values.len(),)+
        }
    };
}

/// Clears the concrete elements of each collection storage variant.
macro_rules! multi_values_clear_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &mut $value.repr {
            MultiValuesRepr::Unset(_) => {}
            $($(#[$cfg])* MultiValuesRepr::$variant(values) => values.clear(),)+
        }
    };
}

/// Appends same-typed elements to an existing collection storage variant.
macro_rules! multi_values_append_match {
    ($left:expr, $right:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match (&mut $left.repr, &mut $right.repr) {
            $(
                $(#[$cfg])*
                (MultiValuesRepr::$variant(values), MultiValuesRepr::$variant(other_values)) => {
                    values.append(other_values);
                }
            )+
            (slot @ MultiValuesRepr::Unset(_), other_values) => {
                *slot = std::mem::replace(other_values, MultiValuesRepr::Unset(DataType::String));
            }
            _ => unreachable!(),
        }
    };
}

/// Clones the first collection element into a scalar value.
macro_rules! multi_values_first_value_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(data_type) => Value::new_unset(*data_type),
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => values
                    .first()
                    .map(|value| materialize_stored!($materialization, value))
                    .map(Value::$variant)
                    .unwrap_or(Value::new_unset($data_type)),
            )+
        }
    };
}

/// Moves the first collection element into a scalar value.
macro_rules! multi_values_into_first_value_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $value.repr {
            MultiValuesRepr::Unset(data_type) => Value::new_unset(data_type),
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => values
                    .into_iter()
                    .next()
                    .map(Value::$variant)
                    .unwrap_or(Value::new_unset($data_type)),
            )+
        }
    };
}

/// Merges same-typed collection storage while preserving element order.
macro_rules! multi_values_merge_match {
    ($left:expr, $right:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match (&mut $left.repr, &$right.repr) {
            $(
                $(#[$cfg])*
                (MultiValuesRepr::$variant(values), MultiValuesRepr::$variant(other_values)) => {
                    values.extend_from_slice(other_values)
                }
            )+
            (slot @ MultiValuesRepr::Unset(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }
    };
}

/// Converts one private scalar storage variant into collection storage.
macro_rules! value_into_multi_values_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $value.repr {
            ValueRepr::Unset(data_type) => MultiValues::new_unset(data_type),
            $($(#[$cfg])* ValueRepr::$variant(value) => {
                MultiValues::$variant(vec![value_storage_into_multi!($variant, value)])
            },)+
        }
    };
}

impl MultiValues {
    /// Generic constructor method
    ///
    /// Creates `MultiValues` from any supported input form, avoiding direct
    /// use of enum variants at call sites.
    ///
    /// Supported input forms include single values, vectors, slices, arrays,
    /// borrowed vectors, and borrowed string collections for supported element
    /// types.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Parameters
    ///
    /// * `values` - Values to convert into a collection.
    ///
    /// # Returns
    ///
    /// Returns `MultiValues` wrapping the converted input values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// // Basic types
    /// let mv = MultiValues::new(vec![1, 2, 3]);
    /// assert_eq!(mv.len(), 3);
    ///
    /// // Strings
    /// let mv = MultiValues::new(vec!["a".to_string(), "b".to_string()]);
    /// assert_eq!(mv.len(), 2);
    /// ```
    #[inline(always)]
    pub fn new<S>(values: S) -> Self
    where
        S: Into<Self>,
    {
        values.into()
    }

    /// Generic getter method for multiple values.
    ///
    /// Performs a strict typed read of all stored values as `Vec<T>`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target element type to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the list of values when the stored type matches `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the container is unset with the
    /// requested type, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// let multi = MultiValues::Int32(vec![1, 2, 3]);
    ///
    /// // Through type inference
    /// let nums: Vec<i32> = multi.get().unwrap();
    /// assert_eq!(nums, vec![1, 2, 3]);
    ///
    /// // Explicitly specify type parameter
    /// let nums = multi.get::<i32>().unwrap();
    /// assert_eq!(nums, vec![1, 2, 3]);
    /// ```
    #[must_use = "the strict collection read result should be handled"]
    #[inline(always)]
    pub fn get<T>(&self) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
    {
        Vec::<T>::try_from(self)
    }

    /// Generic getter method with a default value list.
    ///
    /// Returns the supplied default only when this container is unset. A
    /// concrete empty vector remains an empty result.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type for the strict read.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used only for unset storage.
    ///
    /// # Returns
    ///
    /// The concrete stored list, or `default` for unset storage.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`.
    #[must_use = "the strict collection read result should be handled"]
    #[inline(always)]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Strictly reads all values or calls `default` only when storage is unset.
    ///
    /// A concrete empty collection is returned unchanged and type mismatches
    /// are preserved without invoking the callback.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only for unset storage.
    ///
    /// # Returns
    ///
    /// The stored list or the callback result.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] for an incompatible concrete
    /// collection without invoking the callback.
    #[must_use = "the strict collection read result should be handled"]
    #[inline(always)]
    pub fn get_or_else<T, F>(&self, default: F) -> ValueResult<Vec<T>>
    where
        for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> Vec<T>,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => Ok(default()),
            result => result,
        }
    }

    /// Generic getter method for the first value
    ///
    /// Reads the first stored value as `T`, performing strict type checking.
    ///
    /// `get_first<T>()` does not do cross-type conversion. When the `converter`
    /// feature is enabled, use `to<T>()` for compatible cross-type conversion.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target element type to retrieve.
    ///
    /// # Returns
    ///
    /// Returns the first value when the stored type matches `T` and at least
    /// one value exists.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the requested type matches but no
    /// value is stored, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// let multi = MultiValues::Int32(vec![42, 100, 200]);
    ///
    /// // Through type inference
    /// let first: i32 = multi.get_first().unwrap();
    /// assert_eq!(first, 42);
    ///
    /// // Explicitly specify type parameter
    /// let first = multi.get_first::<i32>().unwrap();
    /// assert_eq!(first, 42);
    ///
    /// // String type
    /// let multi = MultiValues::String(vec!["hello".to_string(), "world".to_string()]);
    /// let first: String = multi.get_first().unwrap();
    /// assert_eq!(first, "hello");
    /// ```
    #[must_use = "the strict first-value result should be handled"]
    #[inline(always)]
    pub fn get_first<T>(&self) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        T::try_from(self)
    }

    /// Generic first-value getter with a default value.
    ///
    /// Returns the supplied default only when the container is unset. A
    /// concrete empty vector returns [`ValueError::Missing`]; type mismatches
    /// are also preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict first-item read.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used only for unset storage.
    ///
    /// # Returns
    ///
    /// The first concrete item, or `default` for unset storage.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] for a concrete empty collection or
    /// [`ValueError::TypeMismatch`] when the stored type differs from `T`.
    #[must_use = "the strict first-value result should be handled"]
    #[inline(always)]
    pub fn get_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get_first() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Strictly reads the first value or calls `default` only when unset.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    /// * `F` - Deferred fallback producing one element.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only for unset storage.
    ///
    /// # Returns
    ///
    /// The first stored item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves empty-collection and type-mismatch errors without invoking
    /// the callback.
    #[must_use = "the strict first-value result should be handled"]
    #[inline(always)]
    pub fn get_first_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> T,
    {
        match self.get_first() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => Ok(default()),
            result => result,
        }
    }

    /// Generic setter method
    ///
    /// Replaces the entire list with the converted input values.
    ///
    /// This operation updates the stored type to the input element type and
    /// does not validate runtime compatibility with the previous variant.
    ///
    /// Supports any input that can be converted into [`MultiValues`], including
    /// single values, vectors, slices, arrays, and borrowed vectors for
    /// supported element types.
    ///
    /// Existing values are replaced, and the stored type becomes the converted
    /// input type.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Parameters
    ///
    /// * `values` - The values to set.
    ///
    /// # Compile-time restriction
    ///
    /// Unsupported input types fail to compile because they do not implement
    /// `Into<MultiValues>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// // 1) Vec<T>
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// mv.set(vec![42, 100, 200]);
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200]);
    ///
    /// // 2) &[T]
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// let slice = &[7, 8, 9][..];
    /// mv.set(slice);
    /// assert_eq!(mv.get_int32s().unwrap(), &[7, 8, 9]);
    ///
    /// // 3) Single T
    /// let mut mv = MultiValues::Unset(DataType::Int32);
    /// mv.set(42);
    /// assert_eq!(mv.get_int32s().unwrap(), &[42]);
    ///
    /// // String example
    /// let mut mv = MultiValues::Unset(DataType::String);
    /// mv.set(vec!["hello".to_string(), "world".to_string()]);
    /// assert_eq!(mv.get_strings().unwrap(), &["hello", "world"]);
    /// ```
    #[inline(always)]
    pub fn set<S>(&mut self, values: S)
    where
        S: Into<Self>,
    {
        *self = values.into();
    }

    /// Generic add method
    ///
    /// Appends converted input values to the existing list with strict type
    /// checking.
    ///
    /// Supports any input that can be converted into [`MultiValues`], including
    /// single values, vectors, slices, arrays, and borrowed vectors for
    /// supported element types.
    ///
    /// The converted input must have the same data type as the current
    /// container. An empty container keeps its declared type until
    /// non-empty values of the same type are appended.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Input type convertible into [`MultiValues`].
    ///
    /// # Parameters
    ///
    /// * `values` - Values to append.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending, including when the input is empty.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the converted input data type
    /// differs from the current container data type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// // 1) Single T
    /// let mut mv = MultiValues::Int32(vec![42]);
    /// mv.add(100).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100]);
    ///
    /// // 2) Vec<T>
    /// mv.add(vec![200, 300]).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200, 300]);
    ///
    /// // 3) &[T]
    /// let slice = &[400, 500][..];
    /// mv.add(slice).unwrap();
    /// assert_eq!(mv.get_int32s().unwrap(), &[42, 100, 200, 300, 400, 500]);
    /// ```
    pub fn add<S>(&mut self, values: S) -> ValueResult<()>
    where
        S: Into<Self>,
    {
        let mut other = values.into();
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }
        if other.is_empty() {
            return Ok(());
        }

        for_each_value_type!(multi_values_append_match, self, other);

        Ok(())
    }

    /// Get the data type of the values
    ///
    /// # Returns
    ///
    /// Returns the data type corresponding to these multiple values
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[must_use = "the collection element type should be used"]
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        for_each_value_type!(multi_values_data_type_match, self)
    }

    /// Returns the number of values.
    ///
    /// # Returns
    ///
    /// The number of values contained in these multiple values. An unset
    /// collection has length zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![1, 2, 3]);
    /// assert_eq!(values.len(), 3);
    ///
    /// let empty = MultiValues::Unset(DataType::String);
    /// assert_eq!(empty.len(), 0);
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn len(&self) -> usize {
        for_each_value_type!(multi_values_count_match, self)
    }

    /// Tests whether this collection contains no values.
    ///
    /// An unset collection and a concrete empty vector both have length zero.
    /// Use [`MultiValues::is_unset`] when the distinction between no collection
    /// and a concrete empty collection matters.
    ///
    /// # Returns
    ///
    /// `true` when [`Self::len`] is zero; otherwise, `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Tests whether this container has no concrete vector.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`MultiValues::Unset`]. A concrete empty vector
    /// returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let values = MultiValues::Int32(vec![]);
    /// assert!(!values.is_unset());
    ///
    /// let empty = MultiValues::Unset(DataType::String);
    /// assert!(empty.is_unset());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_unset(&self) -> bool {
        matches!(self.repr, MultiValuesRepr::Unset(_))
    }

    /// Tests whether a concrete collection belongs to the numeric type family.
    ///
    /// A concrete empty numeric vector returns `true`; an unset collection
    /// returns `false`, even when its declared type is numeric.
    ///
    /// # Returns
    ///
    /// `true` for concrete collections with a numeric element type.
    #[inline(always)]
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete vector while preserving its declared data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        *self = MultiValues::new_unset(self.data_type());
    }

    /// Clears all values while preserving a concrete collection and its type.
    /// An unset collection remains unset because it has no concrete vector to
    /// clear.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.clear();
    /// assert_eq!(values.len(), 0);
    /// assert_eq!(values.data_type(), DataType::Int32);
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        for_each_value_type!(multi_values_clear_match, self)
    }

    /// Set the data type
    ///
    /// If the new type differs from the current type, clears all values and
    /// sets the new type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - The data type to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::MultiValues;
    ///
    /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
    /// values.set_type(DataType::String);
    /// assert!(values.is_unset());
    /// assert_eq!(values.data_type(), DataType::String);
    /// ```
    #[inline(always)]
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = MultiValues::new_unset(data_type);
        }
    }

    /// Converts the first element to a single [`Value`].
    ///
    /// Returns `Value::Unset` with the same declared type when no element is
    /// stored.
    ///
    /// # Returns
    ///
    /// A cloned first item, or a typed unset value when no item exists.
    #[must_use = "the projected first value should be used"]
    #[inline(always)]
    pub fn first_value(&self) -> Value {
        for_each_value_type!(multi_values_first_value_match, self)
    }

    /// Consumes this collection and returns its first item as a [`Value`].
    ///
    /// Empty and unset collections become [`Value::Unset`] with the same data
    /// type. Owned element storage is moved instead of cloned.
    ///
    /// # Returns
    ///
    /// The owned first item, or a typed unset value when no item exists.
    pub fn into_first_value(self) -> Value {
        for_each_value_type!(multi_values_into_first_value_match, self)
    }

    /// Appends all values from another container with the same data type.
    ///
    /// # Parameters
    ///
    /// * `other` - Collection whose values are cloned and appended.
    ///
    /// # Returns
    ///
    /// `Ok(())` after appending, including when `other` is empty.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when `other` has a different data
    /// type.
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }
        if other.is_empty() {
            return Ok(());
        }
        for_each_value_type!(multi_values_merge_match, self, other);
        Ok(())
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        for_each_value_type!(value_into_multi_values_match, value)
    }
}

/// Converts the first collection element with a standalone policy and limits.
#[cfg(feature = "converter")]
macro_rules! multi_values_convert_first_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    convert_first_with(DataConverters::from(values), $policy, $limits)
                }
            )+
        }
    };
}

/// Converts every collection element with a standalone policy and limits.
#[cfg(feature = "converter")]
macro_rules! multi_values_convert_list_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    convert_values_with(DataConverters::from(values), $policy, $limits)
                }
            )+
        }
    };
}

/// Converts the first collection element through a caller-owned session.
#[cfg(feature = "converter")]
macro_rules! multi_values_convert_first_in_match {
    ($value:expr, $session:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    DataConverters::from(values)
                        .to_first_in($session)
                        .map_err(ValueError::from)
                }
            )+
        }
    };
}

/// Converts every collection element through a caller-owned session.
#[cfg(feature = "converter")]
macro_rules! multi_values_convert_list_in_match {
    ($value:expr, $session:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    DataConverters::from(values)
                        .to_vec_in($session)
                        .map_err(ValueError::from)
                }
            )+
        }
    };
}

// ============================================================================
// Inherent conversion APIs
// ============================================================================

/// Converts the first item from a batch converter using conversion policy and
/// limits.
///
/// # Type Parameters
///
/// * `T` - Target type.
/// * `I` - Iterator type wrapped by `DataConverters`.
///
/// # Parameters
///
/// * `values` - Batch converter containing source values.
/// * `policy` - Conversion policy forwarded to `qubit_datatype`.
/// * `limits` - Conversion limits forwarded to `qubit_datatype`.
///
/// # Returns
///
/// Returns the converted first value.
///
/// # Errors
///
/// Returns the mapped single-value conversion error for an empty source or an
/// invalid first source value.
#[inline(always)]
#[cfg(feature = "converter")]
fn convert_first_with<'a, T, I>(
    values: DataConverters<I>,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<T>
where
    T: DataConversionTarget,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values
        .to_first_with(policy, limits)
        .map_err(ValueError::from)
}

/// Converts every item from a batch converter using conversion policy and
/// limits.
///
/// # Type Parameters
///
/// * `T` - Target element type.
/// * `I` - Iterator type wrapped by `DataConverters`.
///
/// # Parameters
///
/// * `values` - Batch converter containing source values.
/// * `policy` - Conversion policy forwarded to `qubit_datatype`.
/// * `limits` - Conversion limits forwarded to `qubit_datatype`.
///
/// # Returns
///
/// Returns converted values in the original order.
///
/// # Errors
///
/// Returns a mapped batch conversion error containing the failing source index.
#[inline(always)]
#[cfg(feature = "converter")]
fn convert_values_with<'a, T, I>(
    values: DataConverters<I>,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<Vec<T>>
where
    T: DataConversionTarget,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values.to_vec_with(policy, limits).map_err(ValueError::from)
}

#[cfg(feature = "converter")]
impl MultiValues {
    /// Converts the first stored value to `T`.
    ///
    /// Unlike [`Self::get_first`], this method uses shared `DataConverter`
    /// conversion rules instead of strict type matching. For example, a stored
    /// `String("1")` can be converted to `bool`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Returns
    ///
    /// The converted first value.
    ///
    /// # Errors
    ///
    /// Returns a structured missing-value conversion error when the container
    /// is unset, an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    #[inline(always)]
    pub fn to_first<T>(&self) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        self.to_first_with(
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
    }

    /// Converts the first stored value to `T`, or returns `default` when the
    /// container is unset or conversion reports a missing value.
    ///
    /// A concrete empty collection remains an error and does not use the
    /// default.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Parameters
    ///
    /// * `default` - Value returned for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted first value, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_first() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts the first value or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted first item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves empty-collection and concrete-value conversion errors without
    /// invoking the callback.
    #[inline]
    pub fn to_first_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_first() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }

    /// Converts the first stored value to `T` using conversion policy and
    /// limits.
    ///
    /// Stored strings are collection items and are never split again by scalar
    /// string collection policy.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Parameters
    ///
    /// * `policy` - Conversion policy forwarded to `qubit_datatype`.
    /// * `limits` - Conversion limits forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// The converted first value.
    ///
    /// # Errors
    ///
    /// Returns a structured missing-value conversion error when the container
    /// is unset, an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    pub fn to_first_with<T>(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_first_match, self, policy, limits)
    }

    /// Converts the first stored value using an existing conversion session.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type supported by the shared conversion layer.
    ///
    /// # Parameters
    ///
    /// * `session` - Caller-owned session providing policy, limits, and budget.
    ///
    /// # Returns
    ///
    /// The converted first element.
    ///
    /// # Errors
    ///
    /// Returns a structured missing, conversion, or budget error when the
    /// first element cannot be produced as `T`.
    pub fn to_first_in<T>(&self, session: &mut ConversionSession<'_>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_first_in_match, self, session)
    }

    /// Converts the first stored value to `T` using conversion policy and
    /// limits, or returns `default` when storage is unset or conversion
    /// reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset storage or a
    ///   conversion-missing result.
    /// * `policy` - Conversion policy forwarded to `qubit_datatype`.
    /// * `limits` - Conversion limits forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// The converted first item, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns an empty-collection error or a conversion error for concrete
    /// values that cannot be converted under the provided policy and limits.
    #[inline]
    pub fn to_first_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_first_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts the first value with the provided policy and limits, or calls
    /// `default` when storage is unset or conversion reports a missing
    /// value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    /// * `policy` - Conversion policy forwarded to the shared converter.
    /// * `limits` - Conversion limits forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted first item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_first_or_else_with<T, F>(
        &self,
        default: F,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_first_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }

    /// Converts all stored values to `T`.
    ///
    /// Unlike [`Self::get`], this method uses shared `DataConverter` conversion
    /// rules for every element instead of strict type matching. A concrete
    /// empty vector returns an empty vector; an unset container reports a
    /// missing-value conversion error.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Returns
    ///
    /// A vector containing all converted values in the original order.
    ///
    /// # Errors
    ///
    /// Returns the first conversion error encountered while converting an
    /// element.
    pub fn to_list<T>(&self) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        self.to_list_with(
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
    }

    /// Converts all stored values to `T`, or returns `default` when storage is
    /// unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used for unset storage or a
    ///   conversion-missing result.
    ///
    /// # Returns
    ///
    /// All converted items, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns the first item conversion error for concrete storage.
    #[inline]
    pub fn to_list_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self.to_list() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts all values or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element conversion type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted list or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_list_or_else<T, F>(&self, default: F) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
        F: FnOnce() -> Vec<T>,
    {
        match self.to_list() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }

    /// Converts all stored values to `T` using conversion policy and limits.
    ///
    /// Stored strings are collection items and are never split again by scalar
    /// string collection policy.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `policy` - Conversion policy forwarded to `qubit_datatype`.
    /// * `limits` - Conversion limits forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// A vector containing all converted values in the original order.
    ///
    /// # Errors
    ///
    /// Returns the first conversion error encountered while converting an
    /// element.
    pub fn to_list_with<T>(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_list_match, self, policy, limits)
    }

    /// Converts every stored value using an existing conversion session.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type supported by the shared conversion layer.
    ///
    /// # Parameters
    ///
    /// * `session` - Caller-owned session providing policy, limits, and budget.
    ///
    /// # Returns
    ///
    /// Converted elements in their original order.
    ///
    /// # Errors
    ///
    /// Returns the first structured missing, conversion, or budget error.
    pub fn to_list_in<T>(&self, session: &mut ConversionSession<'_>) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_list_in_match, self, session)
    }

    /// Converts all stored values to `T` using conversion policy and limits, or
    /// returns `default` when storage is unset or conversion reports a
    /// missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used for unset storage or a
    ///   conversion-missing result.
    /// * `policy` - Conversion policy forwarded to `qubit_datatype`.
    /// * `limits` - Conversion limits forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// All converted items, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns the first item conversion error for concrete storage.
    #[inline]
    pub fn to_list_or_with<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self.to_list_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts all values with the provided policy and limits, or calls
    /// `default` when storage is unset or conversion reports a missing
    /// value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element conversion type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    /// * `policy` - Conversion policy forwarded to the shared converter.
    /// * `limits` - Conversion limits forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted list or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_list_or_else_with<T, F>(
        &self,
        default: F,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
        F: FnOnce() -> Vec<T>,
    {
        match self.to_list_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default())
            }
            result => result,
        }
    }
}

impl MultiValues {
    // ========================================================================
    // Get first value (as single value access)
    // ========================================================================

    impl_get_first_value! {
        /// Get the first boolean value.
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first boolean value; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false]);
        /// assert_eq!(values.get_first_bool().unwrap(), true);
        /// ```
        copy: get_first_bool, Bool, bool, DataType::Bool
    }

    impl_get_first_value! {
        /// Get the first character value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first character value; see `# Errors`.
        copy: get_first_char, Char, char, DataType::Char
    }

    impl_get_first_value! {
        /// Get the first int8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int8 value; see `# Errors`.
        copy: get_first_int8, Int8, i8, DataType::Int8
    }

    impl_get_first_value! {
        /// Get the first int16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int16 value; see `# Errors`.
        copy: get_first_int16, Int16, i16, DataType::Int16
    }

    impl_get_first_value! {
        /// Get the first int32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int32 value; see `# Errors`.
        copy: get_first_int32, Int32, i32, DataType::Int32
    }

    impl_get_first_value! {
        /// Get the first int64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int64 value; see `# Errors`.
        copy: get_first_int64, Int64, i64, DataType::Int64
    }

    impl_get_first_value! {
        /// Get the first int128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first int128 value; see `# Errors`.
        copy: get_first_int128, Int128, i128, DataType::Int128
    }

    impl_get_first_value! {
        /// Get the first uint8 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint8 value; see `# Errors`.
        copy: get_first_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_first_value! {
        /// Get the first uint16 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint16 value; see `# Errors`.
        copy: get_first_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_first_value! {
        /// Get the first uint32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint32 value; see `# Errors`.
        copy: get_first_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_first_value! {
        /// Get the first uint64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint64 value; see `# Errors`.
        copy: get_first_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_first_value! {
        /// Get the first uint128 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first uint128 value; see `# Errors`.
        copy: get_first_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_first_value! {
        /// Get the first float32 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float32 value; see `# Errors`.
        copy: get_first_float32, Float32, f32, DataType::Float32
    }

    impl_get_first_value! {
        /// Get the first float64 value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first float64 value; see `# Errors`.
        copy: get_first_float64, Float64, f64, DataType::Float64
    }

    impl_get_first_value! {
        /// Get the first string reference
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns a reference to the first
        /// string; see `# Errors`.
        ref: get_first_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    impl_get_first_value! {
        /// Get the first date value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first date value; see `# Errors`.
        #[cfg(feature = "chrono")]
        copy: get_first_date, Date, NaiveDate, DataType::Date
    }

    impl_get_first_value! {
        /// Get the first time value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first time value; see `# Errors`.
        #[cfg(feature = "chrono")]
        copy: get_first_time, Time, NaiveTime, DataType::Time
    }

    impl_get_first_value! {
        /// Get the first datetime value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first datetime value; see `# Errors`.
        #[cfg(feature = "chrono")]
        copy: get_first_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_first_value! {
        /// Get the first UTC instant value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first UTC instant
        /// value; see `# Errors`.
        #[cfg(feature = "chrono")]
        copy: get_first_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_first_value! {
        /// Get the first big integer value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big integer
        /// value; see `# Errors`.
        #[cfg(feature = "big-integer")]
        ref: get_first_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    impl_get_first_value! {
        /// Get the first big decimal value
        ///
        /// # Returns
        ///
        /// If types match and a value exists, returns the first big decimal
        /// value; see `# Errors`.
        #[cfg(feature = "big-decimal")]
        ref: get_first_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal, |v: &BigDecimal| v.clone()
    }

    impl_get_first_value! {
        /// Get the first Duration value
        ///
        /// # Returns
        ///
        /// The first duration when the stored type matches.
        copy: get_first_duration, Duration, Duration, DataType::Duration
    }

    impl_get_first_value! {
        /// Get the first Url value
        ///
        /// # Returns
        ///
        /// A clone of the first URL when the stored type matches.
        #[cfg(feature = "url")]
        ref: get_first_url, Url, Url, DataType::Url, |v: &Url| v.clone()
    }

    impl_get_first_value! {
        /// Get the first StringMap value
        ///
        /// # Returns
        ///
        /// A clone of the first string map when the stored type matches.
        ref: get_first_string_map, StringMap, HashMap<String, String>, DataType::StringMap, |v: &HashMap<String, String>| v.clone()
    }

    impl_get_first_value! {
        /// Get the first Json value
        ///
        /// # Returns
        ///
        /// A clone of the first JSON value when the stored type matches.
        #[cfg(feature = "json")]
        ref: get_first_json, Json, serde_json::Value, DataType::Json, |v: &serde_json::Value| v.clone()
    }

    // ========================================================================
    // Get all values (type checking)
    // ========================================================================

    impl_get_multi_values! {
        /// Get reference to all boolean values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the boolean value array; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let values = MultiValues::Bool(vec![true, false, true]);
        /// assert_eq!(values.get_bools().unwrap(), &[true, false, true]);
        /// ```
        slice: get_bools, Bool, bool, DataType::Bool
    }

    impl_get_multi_values! {
        /// Get reference to all character values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the character value array; see `# Errors`.
        slice: get_chars, Char, char, DataType::Char
    }

    impl_get_multi_values! {
        /// Get reference to all int8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int8 value array; see `# Errors`.
        slice: get_int8s, Int8, i8, DataType::Int8
    }

    impl_get_multi_values! {
        /// Get reference to all int16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int16 value array; see `# Errors`.
        slice: get_int16s, Int16, i16, DataType::Int16
    }

    impl_get_multi_values! {
        /// Get reference to all int32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int32 value array; see `# Errors`.
        slice: get_int32s, Int32, i32, DataType::Int32
    }

    impl_get_multi_values! {
        /// Get reference to all int64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int64 value array; see `# Errors`.
        slice: get_int64s, Int64, i64, DataType::Int64
    }

    impl_get_multi_values! {
        /// Get reference to all int128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the int128 value array; see `# Errors`.
        slice: get_int128s, Int128, i128, DataType::Int128
    }

    impl_get_multi_values! {
        /// Get reference to all uint8 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint8 value array; see `# Errors`.
        slice: get_uint8s, UInt8, u8, DataType::UInt8
    }

    impl_get_multi_values! {
        /// Get reference to all uint16 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint16 value array; see `# Errors`.
        slice: get_uint16s, UInt16, u16, DataType::UInt16
    }

    impl_get_multi_values! {
        /// Get reference to all uint32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint32 value array; see `# Errors`.
        slice: get_uint32s, UInt32, u32, DataType::UInt32
    }

    impl_get_multi_values! {
        /// Get reference to all uint64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint64 value array; see `# Errors`.
        slice: get_uint64s, UInt64, u64, DataType::UInt64
    }

    impl_get_multi_values! {
        /// Get reference to all uint128 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the uint128 value array; see `# Errors`.
        slice: get_uint128s, UInt128, u128, DataType::UInt128
    }

    impl_get_multi_values! {
        /// Get reference to all float32 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float32 value array; see `# Errors`.
        slice: get_float32s, Float32, f32, DataType::Float32
    }

    impl_get_multi_values! {
        /// Get reference to all float64 values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the float64 value array; see `# Errors`.
        slice: get_float64s, Float64, f64, DataType::Float64
    }

    impl_get_multi_values! {
        /// Get reference to all strings
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string array; otherwise
        /// returns an error
        vec: get_strings, String, String, DataType::String
    }

    impl_get_multi_values! {
        /// Get reference to all date values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the date value array; see `# Errors`.
        #[cfg(feature = "chrono")]
        slice: get_dates, Date, NaiveDate, DataType::Date
    }

    impl_get_multi_values! {
        /// Get reference to all time values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the time value array; see `# Errors`.
        #[cfg(feature = "chrono")]
        slice: get_times, Time, NaiveTime, DataType::Time
    }

    impl_get_multi_values! {
        /// Get reference to all datetime values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the datetime value array; see `# Errors`.
        #[cfg(feature = "chrono")]
        slice: get_datetimes, DateTime, NaiveDateTime, DataType::DateTime
    }

    impl_get_multi_values! {
        /// Get reference to all UTC instant values
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the UTC instant value array; see `# Errors`.
        #[cfg(feature = "chrono")]
        slice: get_instants, Instant, DateTime<Utc>, DataType::Instant
    }

    impl_get_multi_values! {
        /// Get reference to all big integers
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big integer array; see `# Errors`.
        #[cfg(feature = "big-integer")]
        vec: get_bigintegers, BigInteger, BigInt, DataType::BigInteger
    }

    impl_get_multi_values! {
        /// Get reference to all big decimals
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the big decimal array; see `# Errors`.
        #[cfg(feature = "big-decimal")]
        vec: get_bigdecimals, BigDecimal, BigDecimal, DataType::BigDecimal
    }

    impl_get_multi_values! {
        /// Get reference to all Duration values
        ///
        /// # Returns
        ///
        /// A slice containing all stored durations.
        slice: get_durations, Duration, Duration, DataType::Duration
    }

    impl_get_multi_values! {
        /// Get reference to all Url values
        ///
        /// # Returns
        ///
        /// A reference to the vector containing all stored URLs.
        #[cfg(feature = "url")]
        vec: get_urls, Url, Url, DataType::Url
    }

    impl_get_multi_values! {
        /// Get reference to all StringMap values
        ///
        /// # Returns
        ///
        /// A reference to the vector containing all stored string maps.
        vec: get_string_maps, StringMap, HashMap<String, String>, DataType::StringMap
    }

    impl_get_multi_values! {
        /// Get reference to all Json values
        ///
        /// # Returns
        ///
        /// A reference to the vector containing all stored JSON values.
        #[cfg(feature = "json")]
        vec: get_jsons, Json, serde_json::Value, DataType::Json
    }
}
