// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Single Value Container
//!
//! Provides type-safe storage and access functionality for single values.
// qubit-style: allow multiple-public-types

use std::cmp::Ordering;
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
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::DataConversionError;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionTarget;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::DataFormat;
use qubit_datatype::DataType;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::InvalidValueReason;
use qubit_datatype::NumberRef;
use qubit_datatype::NumericComparisonPolicy;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_json::value::JsonValueEncodeErrorKind;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_json::value::JsonValueEncoder;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Deserialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::Serialize;
#[cfg(all(feature = "converter", feature = "json"))]
use serde::de::DeserializeOwned;
#[cfg(feature = "url")]
use url::Url;

use super::internal::ValueRepr;
use super::value_ref::ValueRef;
use crate::IntoValueDefault;
use crate::NumericComparisonError;
use crate::ValueError;
use crate::ValueMissing;
#[cfg(feature = "json")]
use crate::identity::hash_json;
#[cfg(feature = "json")]
use crate::identity::preflight_json;
#[cfg(feature = "json")]
use crate::value::value_identity::hash_value_payload_with_json_budget;
use crate::value_error::ValueResult;

/// Single typed runtime value with private storage representation.
///
/// Construction and access are expressed through methods and conversions. The
/// concrete enum representation is private so storage optimizations do not
/// become part of the public API.
///
/// # Examples
///
/// ```
/// use qubit_value::Value;
///
/// let value = Value::from(42_i32);
/// assert_eq!(value.get_int32().unwrap(), 42);
/// ```
#[must_use]
#[derive(Clone)]
pub struct Value {
    /// Private typed storage backing the stable public accessor API.
    pub(crate) repr: ValueRepr,
}

impl fmt::Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.view().fmt(formatter)
    }
}

/// Implements named scalar constructors from the shared value table.
macro_rules! impl_value_constructors {
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
        impl Value {
            /// Creates an unset value with an explicit declared type.
            ///
            /// # Parameters
            ///
            /// * `data_type` - Runtime type retained while the value is unset.
            ///
            /// # Returns
            ///
            /// An unset scalar retaining `data_type`.
            #[allow(non_snake_case)]
            #[inline(always)]
            pub const fn Unset(data_type: DataType) -> Self {
                Self::new_unset(data_type)
            }

            /// Creates an unset value with an explicit declared type.
            ///
            /// # Parameters
            ///
            /// * `data_type` - Runtime type retained while the value is unset.
            ///
            /// # Returns
            ///
            /// An unset scalar retaining `data_type`.
            #[inline(always)]
            pub const fn new_unset(data_type: DataType) -> Self {
                Self { repr: ValueRepr::Unset(data_type) }
            }

            $(
                #[doc = concat!("Creates a ", $value_doc, ".")]
                ///
                /// # Parameters
                ///
                /// * `value` - Concrete payload stored by the returned scalar.
                ///
                /// # Returns
                ///
                /// A typed scalar containing `value`.
                $(#[$cfg])*
                #[allow(non_snake_case)]
                #[inline(always)]
                pub fn $variant(value: $type) -> Self {
                    Self { repr: ValueRepr::$variant(value_storage_new!($variant, value)) }
                }
            )+
        }
    };
}

for_each_value_type!(impl_value_constructors);

impl Value {
    /// Hashes this value while applying `budget` to a JSON payload.
    ///
    /// # Type Parameters
    ///
    /// * `H` - Hasher receiving the semantic value identity.
    /// * `R` - Resource identifier used by the JSON budget.
    /// * `Q` - Quantity type used by the JSON budget.
    ///
    /// # Parameters
    ///
    /// * `state` - Hasher that receives the same identity representation as
    ///   [`Hash::hash`].
    /// * `budget` - Mutable JSON traversal budget, used only when this value
    ///   contains a JSON payload.
    ///
    /// # Returns
    ///
    /// `Ok(())` after the complete semantic identity is hashed.
    ///
    /// # Errors
    ///
    /// Returns [`qubit_budget::MeasuredBudgetError`] when the JSON payload
    /// exceeds a configured limit. On error, neither `state` nor the committed
    /// portion of `budget` is modified. A hasher panic also drops the
    /// staged budget transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::hash_map::DefaultHasher;
    ///
    /// use qubit_budget::{ResourceLimit, StructureLimits};
    /// use qubit_budget::json::{JsonResource, JsonValueBudget, JsonValueLimits};
    /// use qubit_value::Value;
    ///
    /// let value = Value::Json(serde_json::json!([null]));
    /// let structure = StructureLimits::<JsonResource, usize>::builder().nodes_limit(
    ///     ResourceLimit::new(JsonResource::Nodes, 1_usize),
    /// ).build();
    /// let mut budget = JsonValueBudget::new(
    ///     JsonValueLimits::builder().structure_limits(structure).build(),
    /// );
    /// let mut hasher = DefaultHasher::new();
    ///
    /// assert!(value.hash_with_json_budget(&mut hasher, &mut budget).is_err());
    /// drop(hasher);
    /// // The rejected value did not consume committed budget state.
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
            ValueRepr::Json(value) => {
                let mut transaction = budget.transaction();
                preflight_json(value, &mut transaction)?;
                std::mem::discriminant(&self.repr).hash(state);
                hash_json(value, state);
                transaction.commit()
            }
            _ => {
                std::mem::discriminant(&self.repr).hash(state);
                hash_value_payload_with_json_budget(&self.repr, state, budget)
            }
        }
    }

    /// Borrows the stable semantic view of this value.
    ///
    /// # Returns
    ///
    /// A non-owning view that hides private storage representation details.
    #[must_use = "the borrowed value view should be used"]
    #[inline(always)]
    pub fn view(&self) -> ValueRef<'_> {
        match &self.repr {
            ValueRepr::Unset(data_type) => ValueRef::Unset(*data_type),
            ValueRepr::Bool(value) => ValueRef::Bool(*value),
            ValueRepr::Char(value) => ValueRef::Char(*value),
            ValueRepr::Int8(value) => ValueRef::Int8(*value),
            ValueRepr::Int16(value) => ValueRef::Int16(*value),
            ValueRepr::Int32(value) => ValueRef::Int32(*value),
            ValueRepr::Int64(value) => ValueRef::Int64(*value),
            ValueRepr::Int128(value) => ValueRef::Int128(*value),
            ValueRepr::UInt8(value) => ValueRef::UInt8(*value),
            ValueRepr::UInt16(value) => ValueRef::UInt16(*value),
            ValueRepr::UInt32(value) => ValueRef::UInt32(*value),
            ValueRepr::UInt64(value) => ValueRef::UInt64(*value),
            ValueRepr::UInt128(value) => ValueRef::UInt128(*value),
            ValueRepr::Float32(value) => ValueRef::Float32(*value),
            ValueRepr::Float64(value) => ValueRef::Float64(*value),
            #[cfg(feature = "big-integer")]
            ValueRepr::BigInteger(value) => ValueRef::BigInteger(value),
            #[cfg(feature = "big-decimal")]
            ValueRepr::BigDecimal(value) => ValueRef::BigDecimal(value),
            ValueRepr::String(value) => ValueRef::String(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Date(value) => ValueRef::Date(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Time(value) => ValueRef::Time(value),
            #[cfg(feature = "chrono")]
            ValueRepr::DateTime(value) => ValueRef::DateTime(value),
            #[cfg(feature = "chrono")]
            ValueRepr::Instant(value) => ValueRef::Instant(value),
            ValueRepr::Duration(value) => ValueRef::Duration(value),
            #[cfg(feature = "url")]
            ValueRepr::Url(value) => ValueRef::Url(value.as_ref()),
            ValueRepr::StringMap(value) => ValueRef::StringMap(value),
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => ValueRef::Json(value),
        }
    }
}

/// Maps private scalar storage variants to their runtime data types.
macro_rules! value_data_type_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            ValueRepr::Unset(data_type) => *data_type,
            $($(#[$cfg])* ValueRepr::$variant(_) => $data_type,)+
        }
    };
}

// ============================================================================
// Getter method generation macro
// ============================================================================

/// Unified getter generation macro
///
/// Supports two modes:
/// 1. `copy:` - For types implementing the Copy trait, directly returns the
///    value
/// 2. `ref:` - For non-Copy types, returns a reference
///
/// # Documentation Comment Support
///
/// The macro automatically extracts preceding documentation comments, so
/// you can add `///` comments before macro invocations.
impl Value {
    /// Generic constructor method
    ///
    /// Creates a `Value` from any supported type, avoiding direct use of
    /// enum variants.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::new<T>(value)` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`, `&str`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the value to wrap
    ///
    /// # Parameters
    ///
    /// * `value` - Value to wrap.
    ///
    /// # Returns
    ///
    /// Returns a `Value` wrapping the given value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// // Basic types
    /// let v = Value::new(42i32);
    /// assert_eq!(v.get_int32().unwrap(), 42);
    ///
    /// let v = Value::new(true);
    /// assert_eq!(v.get_bool().unwrap(), true);
    ///
    /// // String
    /// let v = Value::new("hello".to_string());
    /// assert_eq!(v.get_string().unwrap(), "hello");
    /// ```
    #[inline(always)]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<Self>,
    {
        value.into()
    }

    /// Generic getter method.
    ///
    /// Performs a strict typed read of the stored value as `T`.
    ///
    /// `get<T>()` performs strict type matching. It does not do cross-type
    /// conversion.
    ///
    /// For example, `Value::Int32(42).get::<i64>()` fails, while
    /// `Value::Int32(42).to::<i64>()` succeeds.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::get<T>()` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to retrieve
    ///
    /// # Returns
    ///
    /// Returns the stored value when its type matches `T`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with the
    /// requested type, or [`ValueError::TypeMismatch`] when the stored type
    /// differs from `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    ///
    /// // Through type inference
    /// let num: i32 = value.get().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// // Explicitly specify type parameter
    /// let num = value.get::<i32>().unwrap();
    /// assert_eq!(num, 42);
    ///
    /// // Different type
    /// let text = Value::String("hello".to_string());
    /// let s: String = text.get().unwrap();
    /// assert_eq!(s, "hello");
    ///
    /// // Boolean value
    /// let flag = Value::Bool(true);
    /// let b: bool = flag.get().unwrap();
    /// assert_eq!(b, true);
    /// ```
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get<T>(&self) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        T::try_from(self)
    }

    /// Generic getter method with a default value.
    ///
    /// Returns the supplied default only when this value is unset. Type
    /// mismatches and conversion errors are still returned as errors.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict read and default value.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used only when `self` is unset.
    ///
    /// # Returns
    ///
    /// The stored value, or `default` when the value is unset.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`.
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Strictly reads this value or calls `default` only when it is unset.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type for the strict read and fallback value.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only when this value is unset.
    ///
    /// # Returns
    ///
    /// The stored value, or the callback result for an unset value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::TypeMismatch`] when the stored type differs from
    /// `T`; the callback is not invoked in that case.
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        for<'a> T: TryFrom<&'a Self, Error = ValueError>,
        F: FnOnce() -> T,
    {
        match self.get() {
            Err(ValueError::Missing(missing)) if missing.is_unset() => Ok(default()),
            result => result,
        }
    }

    /// Converts the stored value to another supported data type.
    ///
    /// This method delegates to the authoritative conversion contract in
    /// [`qubit-datatype`](https://docs.rs/qubit-datatype/latest/qubit_datatype/).
    /// The enabled rich-type features determine which source and target
    /// families are available. An unset value is reported as a structured
    /// missing-value conversion error.
    ///
    /// Unlike [`Self::get`], this method permits conversions supported by
    /// [`qubit_datatype::DataConverter`] and applies
    /// [`qubit_datatype::ConversionPolicy`] and
    /// [`qubit_datatype::ConversionLimits`].
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type supported by the shared conversion layer.
    ///
    /// # Returns
    ///
    /// The converted value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error when the value is unset, the
    /// conversion is unsupported, or the source is invalid for `T`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.to::<i64>().unwrap(), 42);
    /// assert_eq!(value.to::<String>().unwrap(), "42");
    /// ```
    #[inline(always)]
    #[cfg(feature = "converter")]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        self.to_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Converts this value to `T`, or returns `default` when storage is unset
    /// or conversion reports a missing value.
    ///
    /// Conversion failures from concrete values are preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset or conversion-
    ///   missing storage.
    ///
    /// # Returns
    ///
    /// The converted value, or `default` for an unset or conversion-missing
    /// value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error for concrete values that cannot be
    /// converted to `T`.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts this value to `T`, or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked only when conversion reports a missing
    ///   value.
    ///
    /// # Returns
    ///
    /// The converted value, or the callback result for an unset or
    /// conversion-missing value.
    ///
    /// # Errors
    ///
    /// Preserves conversion errors from concrete values without invoking the
    /// callback.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to() {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => Ok(default()),
            result => result,
        }
    }

    /// Converts this value to `T` using the provided conversion policy and
    /// limits.
    ///
    /// This method uses the shared [`qubit_datatype`] conversion layer
    /// directly, so policy settings such as string trimming, blank string
    /// handling, and boolean aliases are applied consistently with other
    /// value containers.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type to convert to.
    ///
    /// # Parameters
    ///
    /// * `policy` - Conversion policy forwarded to the shared converter.
    /// * `limits` - Conversion limits forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// Returns the converted value on success.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::ValueError`] when the value is missing, unsupported,
    /// or invalid for `T` under the provided policy and limits.
    #[inline(always)]
    #[cfg(feature = "converter")]
    pub fn to_with<T>(&self, policy: &ConversionPolicy, limits: &ConversionLimits) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        super::value_converters::convert_with_data_converter_with(self, policy, limits)
    }

    /// Converts this value to `T` while charging an existing conversion
    /// session.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type supported by the shared conversion layer.
    ///
    /// # Parameters
    ///
    /// * `session` - Caller-owned session providing policy, limits, and budget.
    ///
    /// # Returns
    ///
    /// The converted value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error when the value is missing,
    /// unsupported, invalid, or exceeds the session budget.
    #[inline(always)]
    #[cfg(feature = "converter")]
    pub fn to_in<T>(&self, session: &mut ConversionSession<'_>) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        super::value_converters::convert_with_data_converter_in(self, session)
    }

    /// Converts this value to `T` using conversion policy and limits, or
    /// returns `default` when storage is unset or conversion reports a
    /// missing value.
    ///
    /// Conversion failures from concrete values are preserved.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset or conversion-
    ///   missing storage.
    /// * `policy` - Conversion policy forwarded to the shared converter.
    /// * `limits` - Conversion limits forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted value, or `default` for an unset or conversion-missing
    /// value.
    ///
    /// # Errors
    ///
    /// Returns a mapped conversion error for concrete values that cannot be
    /// converted under the provided policy and limits.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts this value with the provided policy and limits, or calls
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
    /// * `default` - Callback invoked only for a missing source value.
    /// * `policy` - Conversion policy forwarded to the shared converter.
    /// * `limits` - Conversion limits forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted value, or the callback result for an unset or
    /// conversion-missing value.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    #[cfg(feature = "converter")]
    pub fn to_or_else_with<T, F>(
        &self,
        default: F,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_with(policy, limits) {
            Err(ValueError::Missing(missing)) if missing.is_defaultable_for_conversion() => Ok(default()),
            result => result,
        }
    }

    /// Generic setter method
    ///
    /// Replaces the current value with any supported input value.
    ///
    /// This operation updates the stored type to `T` when needed. It does not
    /// perform runtime type-mismatch validation against the previous variant.
    ///
    /// # Supported Generic Types
    ///
    /// `Value::set<T>(value)` currently supports the following `T`:
    ///
    /// - `bool`
    /// - `char`
    /// - `i8`, `i16`, `i32`, `i64`, `i128`
    /// - `u8`, `u16`, `u32`, `u64`, `u128`
    /// - `f32`, `f64`
    /// - `String`, `&str`
    /// - `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`
    /// - `BigInt`, `BigDecimal`
    /// - `Duration`
    /// - `Url`
    /// - `HashMap<String, String>`
    /// - `serde_json::Value`
    ///
    /// # Type Parameters
    ///
    /// * `T` - Input type convertible into [`Value`].
    ///
    /// # Parameters
    ///
    /// * `value` - The value to set
    ///
    /// # Compile-time restriction
    ///
    /// Unsupported input types fail to compile because they do not implement
    /// `Into<Value>`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let mut value = Value::Unset(DataType::Int32);
    ///
    /// // Through type inference
    /// value.set(42i32);
    /// assert_eq!(value.get_int32().unwrap(), 42);
    ///
    /// // Explicitly specify type parameter
    /// value.set::<i32>(100);
    /// assert_eq!(value.get_int32().unwrap(), 100);
    ///
    /// // String type
    /// let mut text = Value::Unset(DataType::String);
    /// text.set("hello".to_string());
    /// assert_eq!(text.get_string().unwrap(), "hello");
    /// ```
    #[inline(always)]
    pub fn set<T>(&mut self, value: T)
    where
        T: Into<Self>,
    {
        *self = value.into();
    }

    /// Get the data type of the value
    ///
    /// # Returns
    ///
    /// Returns the data type corresponding to this value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert_eq!(value.data_type(), DataType::Int32);
    ///
    /// let empty = Value::Unset(DataType::String);
    /// assert_eq!(empty.data_type(), DataType::String);
    /// ```
    ///
    /// ```compile_fail
    /// #![deny(unused_must_use)]
    /// use qubit_value::Value;
    ///
    /// Value::new(42_i32).data_type();
    /// ```
    #[must_use = "the runtime data type should be used"]
    #[inline(always)]
    pub fn data_type(&self) -> DataType {
        for_each_value_type!(value_data_type_match, self)
    }

    /// Tests whether this container has no concrete value.
    ///
    /// # Returns
    ///
    /// Returns `true` only for [`Value::Unset`]. An empty string, map, or JSON
    /// container is still a concrete value and returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let value = Value::Int32(42);
    /// assert!(!value.is_unset());
    ///
    /// let empty = Value::Unset(DataType::String);
    /// assert!(empty.is_unset());
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn is_unset(&self) -> bool {
        matches!(self.repr, ValueRepr::Unset(_))
    }

    /// Tests whether a concrete value belongs to the numeric type family.
    ///
    /// An unset value returns `false`, even when its declared type is numeric.
    ///
    /// # Returns
    ///
    /// `true` for concrete numeric variants; otherwise `false`.
    #[inline(always)]
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        !self.is_unset() && self.data_type().is_numeric()
    }

    /// Removes the concrete value while preserving its declared data type.
    #[inline(always)]
    pub fn unset(&mut self) {
        *self = Value::new_unset(self.data_type());
    }

    /// Set the data type
    ///
    /// If the new type differs from the current type, clears the value
    /// and sets the new type.
    ///
    /// # Parameters
    ///
    /// * `data_type` - The data type to set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use qubit_datatype::DataType;
    /// use qubit_value::Value;
    ///
    /// let mut value = Value::Int32(42);
    /// value.set_type(DataType::String);
    /// assert!(value.is_unset());
    /// assert_eq!(value.data_type(), DataType::String);
    /// ```
    #[inline(always)]
    pub fn set_type(&mut self, data_type: DataType) {
        if self.data_type() != data_type {
            *self = Value::new_unset(data_type);
        }
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
impl Value {
    /// Projects this typed value to its natural JSON representation.
    ///
    /// This differs from the tagged [`crate::ValueWireV1`] representation: for
    /// example, `Value::Int32(42)` projects to the JSON number `42`.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this value.
    ///
    /// # Errors
    ///
    /// Returns a structured conversion error for values JSON cannot represent,
    /// including non-finite floating-point values and inexact durations.
    #[inline(always)]
    pub fn to_json_value(&self) -> ValueResult<serde_json::Value> {
        self.to_json_value_with(ConversionPolicy::default_ref(), ConversionLimits::default_ref())
    }

    /// Projects this typed value using explicit conversion policy and limits.
    ///
    /// # Parameters
    ///
    /// * `policy` - Controls duration units and precision-loss behavior.
    /// * `limits` - Bounds conversion resource consumption.
    ///
    /// # Returns
    ///
    /// The natural JSON representation of this value.
    ///
    /// # Errors
    ///
    /// Returns a structured conversion error when JSON projection or duration
    /// formatting violates the requested policy or limits.
    #[inline(always)]
    pub fn to_json_value_with(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<serde_json::Value> {
        crate::json::value_to_json_value_with(self, policy, limits)
    }
}

/// Implements one strict typed getter from the shared value table.
macro_rules! impl_get_value {
    // Copy type: directly dereference and return
    ($(#[$attr:meta])* copy: $method:ident, $variant:ident, $type:ty, $data_type:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the value is unset with"]
        #[doc = "the requested type, or [`ValueError::TypeMismatch`] when the"]
        #[doc = "stored data type differs."]
        #[must_use = "the strict value read result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$type> {
            match &self.repr {
                ValueRepr::$variant(v) => Ok(*v),
                ValueRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetScalar {
                        data_type: *dt,
                    }))
                }
                ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: *dt,
                }),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };

    // Reference type: use conversion function to return reference,
    // fixing lifetime issues
    ($(#[$attr:meta])* ref: $method:ident, $variant:ident, $ret_type:ty, $data_type:expr, $conversion:expr) => {
        $(#[$attr])*
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = ""]
        #[doc = "Returns [`ValueError::Missing`] when the value is unset with"]
        #[doc = "the requested type, or [`ValueError::TypeMismatch`] when the"]
        #[doc = "stored data type differs."]
        #[must_use = "the strict value read result should be handled"]
        #[inline(always)]
        pub fn $method(&self) -> ValueResult<$ret_type> {
            match &self.repr {
                ValueRepr::$variant(v) => {
                    let conv_fn: fn(&_) -> $ret_type = $conversion;
                    Ok(conv_fn(v))
                },
                ValueRepr::Unset(dt) if *dt == $data_type => {
                    Err(ValueError::Missing($crate::ValueMissing::UnsetScalar {
                        data_type: *dt,
                    }))
                }
                ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: *dt,
                }),
                _ => Err(ValueError::TypeMismatch {
                    expected: $data_type,
                    actual: self.data_type(),
                }),
            }
        }
    };
}

impl Value {
    /// Creates a `Value` from a `serde_json::Value`.
    ///
    /// # Parameters
    ///
    /// * `json` - The JSON value to wrap.
    ///
    /// # Returns
    ///
    /// A `Value::Json` wrapping the given JSON value.
    #[inline(always)]
    #[cfg(feature = "json")]
    pub fn from_json_value(json: serde_json::Value) -> Self {
        Value::Json(json)
    }

    /// Creates a `Value` from any serializable value by converting it to JSON.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Any type implementing `Serialize`.
    ///
    /// # Parameters
    ///
    /// * `value` - The value to serialize into JSON.
    ///
    /// # Returns
    ///
    /// A `Value::Json` containing the serialized representation.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Conversion`] with
    /// A non-finite reason is returned when any nested float is non-finite, an
    /// out-of-range reason when an integer exceeds the strict JSON range, or a
    /// serialization reason for every other unsupported Serde representation.
    #[cfg(all(feature = "converter", feature = "json"))]
    pub fn from_serializable<T: ?Sized + Serialize>(value: &T) -> ValueResult<Self> {
        let json = JsonValueEncoder::new().encode(value).map_err(|error| {
            let reason = match error.kind() {
                JsonValueEncodeErrorKind::NonFiniteFloat => InvalidValueReason::NonFinite,
                JsonValueEncodeErrorKind::IntegerOutOfRange { .. } => InvalidValueReason::OutOfRange,
                _ => InvalidValueReason::Serialization {
                    format: DataFormat::Json,
                },
            };
            ValueError::from(DataConversionError::invalid(DataType::Json, DataType::Json, reason))
        })?;
        Ok(Value::Json(json))
    }

    // ========================================================================
    // Type-checking getters (strict type matching)
    // ========================================================================

    impl_get_value! {
        /// Get boolean value
        ///
        /// # Returns
        ///
        /// If types match, returns the boolean value; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::Bool(true);
        /// assert_eq!(value.get_bool().unwrap(), true);
        /// ```
        copy: get_bool, Bool, bool, DataType::Bool
    }

    impl_get_value! {
        /// Get character value
        ///
        /// # Returns
        ///
        /// If types match, returns the character value; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::Char('A');
        /// assert_eq!(value.get_char().unwrap(), 'A');
        /// ```
        copy: get_char, Char, char, DataType::Char
    }

    impl_get_value! {
        /// Get int8 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int8 value; see `# Errors`.
        copy: get_int8, Int8, i8, DataType::Int8
    }

    impl_get_value! {
        /// Get int16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int16 value; see `# Errors`.
        copy: get_int16, Int16, i16, DataType::Int16
    }

    impl_get_value! {
        /// Get int32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int32 value; see `# Errors`.
        copy: get_int32, Int32, i32, DataType::Int32
    }

    impl_get_value! {
        /// Get int64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int64 value; see `# Errors`.
        copy: get_int64, Int64, i64, DataType::Int64
    }

    impl_get_value! {
        /// Get int128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the int128 value; see `# Errors`.
        copy: get_int128, Int128, i128, DataType::Int128
    }

    impl_get_value! {
        /// Get uint8 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint8 value; see `# Errors`.
        copy: get_uint8, UInt8, u8, DataType::UInt8
    }

    impl_get_value! {
        /// Get uint16 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint16 value; see `# Errors`.
        copy: get_uint16, UInt16, u16, DataType::UInt16
    }

    impl_get_value! {
        /// Get uint32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint32 value; see `# Errors`.
        copy: get_uint32, UInt32, u32, DataType::UInt32
    }

    impl_get_value! {
        /// Get uint64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint64 value; see `# Errors`.
        copy: get_uint64, UInt64, u64, DataType::UInt64
    }

    impl_get_value! {
        /// Get uint128 value
        ///
        /// # Returns
        ///
        /// If types match, returns the uint128 value; see `# Errors`.
        copy: get_uint128, UInt128, u128, DataType::UInt128
    }

    impl_get_value! {
        /// Get float32 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float32 value; see `# Errors`.
        copy: get_float32, Float32, f32, DataType::Float32
    }

    impl_get_value! {
        /// Get float64 value
        ///
        /// # Returns
        ///
        /// If types match, returns the float64 value; see `# Errors`.
        copy: get_float64, Float64, f64, DataType::Float64
    }

    impl_get_value! {
        /// Get string reference
        ///
        /// # Returns
        ///
        /// If types match, returns a reference to the string; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::String("hello".to_string());
        /// assert_eq!(value.get_string().unwrap(), "hello");
        /// ```
        ref: get_string, String, &str, DataType::String, |s: &String| s.as_str()
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get date value
        ///
        /// # Returns
        ///
        /// If types match, returns the date value; see `# Errors`.
        copy: get_date, Date, NaiveDate, DataType::Date
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get time value
        ///
        /// # Returns
        ///
        /// If types match, returns the time value; see `# Errors`.
        copy: get_time, Time, NaiveTime, DataType::Time
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get datetime value
        ///
        /// # Returns
        ///
        /// If types match, returns the datetime value; see `# Errors`.
        copy: get_datetime, DateTime, NaiveDateTime, DataType::DateTime
    }

    #[cfg(feature = "chrono")]
    impl_get_value! {
        /// Get UTC instant value
        ///
        /// # Returns
        ///
        /// If types match, returns the UTC instant value; see `# Errors`.
        copy: get_instant, Instant, DateTime<Utc>, DataType::Instant
    }

    #[cfg(feature = "big-integer")]
    impl_get_value! {
        /// Get big integer value.
        ///
        /// This method returns a cloned [`BigInt`]. Use
        /// [`Value::get_biginteger_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big integer value; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        /// use num_bigint::BigInt;
        ///
        /// let value = Value::BigInteger(BigInt::from(123456789));
        /// assert_eq!(value.get_biginteger().unwrap(), BigInt::from(123456789));
        /// ```
        ref: get_biginteger, BigInteger, BigInt, DataType::BigInteger, |v: &BigInt| v.clone()
    }

    #[cfg(feature = "big-decimal")]
    impl_get_value! {
        /// Get big decimal value.
        ///
        /// This method returns a cloned [`BigDecimal`]. Use
        /// [`Value::get_bigdecimal_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the big decimal value; see `# Errors`.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use std::str::FromStr;
        ///
        /// use bigdecimal::BigDecimal;
        /// use qubit_value::Value;
        ///
        /// let bd = BigDecimal::from_str("123.456").unwrap();
        /// let value = Value::BigDecimal(bd.clone());
        /// assert_eq!(value.get_bigdecimal().unwrap(), bd);
        /// ```
        ref: get_bigdecimal, BigDecimal, BigDecimal, DataType::BigDecimal, |v: &BigDecimal| v.clone()
    }

    impl_get_value! {
        /// Get Duration value
        ///
        /// # Returns
        ///
        /// If types match, returns the Duration value; see `# Errors`.
        copy: get_duration, Duration, Duration, DataType::Duration
    }

    #[cfg(feature = "url")]
    impl_get_value! {
        /// Get URL value.
        ///
        /// This method returns a cloned [`Url`]. Use [`Value::get_url_ref`] to
        /// borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the URL value; see `# Errors`.
        ref: get_url, Url, Url, DataType::Url, Url::clone
    }

    impl_get_value! {
        /// Get string map value.
        ///
        /// This method returns a cloned `HashMap<String, String>`. Use
        /// [`Value::get_string_map_ref`] to borrow the stored value without
        /// cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the string map value; see `# Errors`.
        ref: get_string_map, StringMap, HashMap<String, String>, DataType::StringMap,
            |v: &HashMap<String, String>| v.clone()
    }

    #[cfg(feature = "json")]
    impl_get_value! {
        /// Get JSON value.
        ///
        /// This method returns a cloned [`serde_json::Value`]. Use
        /// [`Value::get_json_ref`] to borrow the stored value without cloning.
        ///
        /// # Returns
        ///
        /// If types match, returns the JSON value; see `# Errors`.
        ref: get_json, Json, serde_json::Value, DataType::Json,
            |v: &serde_json::Value| v.clone()
    }

    /// Borrow the inner `BigInt` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored integer.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::BigInteger`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[cfg(feature = "big-integer")]
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_biginteger_ref(&self) -> ValueResult<&BigInt> {
        match &self.repr {
            ValueRepr::BigInteger(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::BigInteger => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::BigInteger,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigInteger,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `BigDecimal` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored decimal.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::BigDecimal`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[cfg(feature = "big-decimal")]
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_bigdecimal_ref(&self) -> ValueResult<&BigDecimal> {
        match &self.repr {
            ValueRepr::BigDecimal(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::BigDecimal => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::BigDecimal,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::BigDecimal,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `Url` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored URL.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::Url`, or [`ValueError::TypeMismatch`] when the stored data
    /// type differs.
    #[cfg(feature = "url")]
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_url_ref(&self) -> ValueResult<&Url> {
        match &self.repr {
            ValueRepr::Url(v) => Ok(v.as_ref()),
            ValueRepr::Unset(dt) if *dt == DataType::Url => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Url,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Url,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner `HashMap<String, String>` without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored string map.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::StringMap`, or [`ValueError::TypeMismatch`] when the stored
    /// data type differs.
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_string_map_ref(&self) -> ValueResult<&HashMap<String, String>> {
        match &self.repr {
            ValueRepr::StringMap(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::StringMap => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::StringMap,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::StringMap,
                actual: self.data_type(),
            }),
        }
    }

    /// Borrow the inner JSON value without cloning.
    ///
    /// # Returns
    ///
    /// A shared reference to the stored JSON value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when the value is unset with
    /// `DataType::Json`, or [`ValueError::TypeMismatch`] when the stored data
    /// type differs.
    #[cfg(feature = "json")]
    #[must_use = "the strict value read result should be handled"]
    #[inline(always)]
    pub fn get_json_ref(&self) -> ValueResult<&serde_json::Value> {
        match &self.repr {
            ValueRepr::Json(v) => Ok(v),
            ValueRepr::Unset(dt) if *dt == DataType::Json => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: self.data_type(),
            }),
        }
    }

    /// Deserialize the inner JSON value into a target type.
    ///
    /// Only works when `self` is `Value::Json(...)`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target type implementing `DeserializeOwned`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(T)` on success.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] when this value is
    /// `Value::Unset(DataType::Json)`,
    /// [`ValueError::TypeMismatch`] when this value has a non-JSON data type,
    /// or [`ValueError::Conversion`] when JSON deserialization fails.
    #[cfg(all(feature = "converter", feature = "json"))]
    pub fn deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T> {
        match &self.repr {
            ValueRepr::Json(v) => Deserialize::deserialize(v).map_err(|_| {
                ValueError::from(DataConversionError::invalid(
                    DataType::Json,
                    DataType::Json,
                    InvalidValueReason::Deserialization {
                        format: DataFormat::Json,
                    },
                ))
            }),
            ValueRepr::Unset(dt) if *dt == DataType::Json => {
                Err(ValueError::Missing(ValueMissing::UnsetScalar { data_type: *dt }))
            }
            ValueRepr::Unset(dt) => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: *dt,
            }),
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::Json,
                actual: self.data_type(),
            }),
        }
    }
}

/// Projects one stored value according to its type-table numeric strategy.
macro_rules! project_number_ref {
    (number_copy, $value:expr) => {
        Some(NumberRef::from(*$value))
    };
    (number_ref, $value:expr) => {
        Some(NumberRef::from($value))
    };
    (not_number, $value:expr) => {{
        let _ = $value;
        None
    }};
}

/// Generates the exhaustive numeric projection from the value type table.
macro_rules! value_number_ref_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            ValueRepr::Unset(_) => None,
            $(
                $(#[$cfg])*
                ValueRepr::$variant(value) => {
                    project_number_ref!($number_projection, value)
                }
            )+
        }
    };
}

impl Value {
    /// Tests whether this value is a concrete floating-point NaN.
    ///
    /// Non-floating-point values and unset values return `false`.
    ///
    /// # Returns
    ///
    /// `true` only for concrete `Float32` or `Float64` NaN values.
    #[inline(always)]
    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.as_number_ref().is_some_and(|value| value.is_nan())
    }

    /// Compares concrete numeric values across representation variants.
    ///
    /// This operation is separate from [`PartialEq`]: equality preserves enum
    /// representation identity, while numeric comparison compares mathematical
    /// values under an explicit policy.
    ///
    /// [`NumericComparisonPolicy::Approximate`] orders primitive infinities
    /// separately. When a finite primitive float participates, it attempts to
    /// project both operands to finite `f64` values; if either operand cannot
    /// be projected that way, comparison falls back to the exact path.
    /// Projected comparison is pair-dependent and not transitive across
    /// mixed representations. Do not use it to implement [`Ord`], sort or
    /// group values, or construct ordered-map or ordered-set keys. Use
    /// [`NumericComparisonPolicy::Exact`] for deterministic ordering.
    ///
    /// Validation is deterministic: missing operands are checked from left to
    /// right, followed by concrete operand types from left to right, and then
    /// NaN positions.
    ///
    /// # Parameters
    ///
    /// * `other` - Right numeric operand.
    /// * `policy` - Exact or approximate numeric comparison policy.
    ///
    /// # Returns
    ///
    /// The mathematical ordering of the two concrete, non-NaN numeric
    /// operands.
    ///
    /// # Errors
    ///
    /// Returns [`NumericComparisonError::LeftMissing`] or
    /// [`NumericComparisonError::RightMissing`] when the corresponding operand
    /// is unset. Returns [`NumericComparisonError::LeftNotNumeric`] or
    /// [`NumericComparisonError::RightNotNumeric`] when the corresponding
    /// concrete operand is not numeric. Returns
    /// [`NumericComparisonError::LeftNaN`],
    /// [`NumericComparisonError::RightNaN`], or
    /// [`NumericComparisonError::BothNaN`] according to the position of NaN
    /// operands. Missing operands are checked left-to-right, then concrete
    /// operand types are checked left-to-right, and finally NaN positions are
    /// classified. After these checks the lower-level comparator must be able
    /// to order the remaining numeric operands.
    pub fn numeric_cmp(
        &self,
        other: &Self,
        policy: NumericComparisonPolicy,
    ) -> Result<Ordering, NumericComparisonError> {
        if let ValueRepr::Unset(declared) = &self.repr {
            return Err(NumericComparisonError::LeftMissing { declared: *declared });
        }
        if let ValueRepr::Unset(declared) = &other.repr {
            return Err(NumericComparisonError::RightMissing { declared: *declared });
        }

        let left = self
            .as_number_ref()
            .ok_or_else(|| NumericComparisonError::LeftNotNumeric {
                actual: self.data_type(),
            })?;
        let right = other
            .as_number_ref()
            .ok_or_else(|| NumericComparisonError::RightNotNumeric {
                actual: other.data_type(),
            })?;

        match (left.is_nan(), right.is_nan()) {
            (true, true) => return Err(NumericComparisonError::BothNaN),
            (true, false) => return Err(NumericComparisonError::LeftNaN),
            (false, true) => return Err(NumericComparisonError::RightNaN),
            (false, false) => {}
        }

        match left.compare(right, policy) {
            Some(ordering) => Ok(ordering),
            None => unreachable!("validated non-NaN numeric values must be orderable"),
        }
    }

    /// Borrows this value as a lower-level numeric representation.
    ///
    /// # Returns
    ///
    /// A borrowed numeric representation for every concrete numeric variant,
    /// or `None` for unset and non-numeric variants.
    #[must_use]
    fn as_number_ref(&self) -> Option<NumberRef<'_>> {
        for_each_value_type!(value_number_ref_match, self)
    }
}
