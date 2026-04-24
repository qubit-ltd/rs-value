use qubit_common::lang::DataType;

use crate::Value;
use crate::error::{ValueError, ValueResult};

use super::multi_values_core::MultiValues;

impl MultiValues {
    /// Convert to a single [`Value`] by taking the first element.
    ///
    /// If there is no element, returns `Value::Empty(self.data_type())`.
    ///
    /// # Returns
    ///
    /// Returns the first element wrapped as [`Value`], or an empty value
    /// preserving the current data type.
    pub fn to_value(&self) -> Value {
        match self {
            MultiValues::Empty(dt) => Value::Empty(*dt),
            MultiValues::Bool(v) => v
                .first()
                .copied()
                .map(Value::Bool)
                .unwrap_or(Value::Empty(DataType::Bool)),
            MultiValues::Char(v) => v
                .first()
                .copied()
                .map(Value::Char)
                .unwrap_or(Value::Empty(DataType::Char)),
            MultiValues::Int8(v) => v
                .first()
                .copied()
                .map(Value::Int8)
                .unwrap_or(Value::Empty(DataType::Int8)),
            MultiValues::Int16(v) => v
                .first()
                .copied()
                .map(Value::Int16)
                .unwrap_or(Value::Empty(DataType::Int16)),
            MultiValues::Int32(v) => v
                .first()
                .copied()
                .map(Value::Int32)
                .unwrap_or(Value::Empty(DataType::Int32)),
            MultiValues::Int64(v) => v
                .first()
                .copied()
                .map(Value::Int64)
                .unwrap_or(Value::Empty(DataType::Int64)),
            MultiValues::Int128(v) => v
                .first()
                .copied()
                .map(Value::Int128)
                .unwrap_or(Value::Empty(DataType::Int128)),
            MultiValues::UInt8(v) => v
                .first()
                .copied()
                .map(Value::UInt8)
                .unwrap_or(Value::Empty(DataType::UInt8)),
            MultiValues::UInt16(v) => v
                .first()
                .copied()
                .map(Value::UInt16)
                .unwrap_or(Value::Empty(DataType::UInt16)),
            MultiValues::UInt32(v) => v
                .first()
                .copied()
                .map(Value::UInt32)
                .unwrap_or(Value::Empty(DataType::UInt32)),
            MultiValues::UInt64(v) => v
                .first()
                .copied()
                .map(Value::UInt64)
                .unwrap_or(Value::Empty(DataType::UInt64)),
            MultiValues::UInt128(v) => v
                .first()
                .copied()
                .map(Value::UInt128)
                .unwrap_or(Value::Empty(DataType::UInt128)),
            MultiValues::IntSize(v) => v
                .first()
                .copied()
                .map(Value::IntSize)
                .unwrap_or(Value::Empty(DataType::IntSize)),
            MultiValues::UIntSize(v) => v
                .first()
                .copied()
                .map(Value::UIntSize)
                .unwrap_or(Value::Empty(DataType::UIntSize)),
            MultiValues::Float32(v) => v
                .first()
                .copied()
                .map(Value::Float32)
                .unwrap_or(Value::Empty(DataType::Float32)),
            MultiValues::Float64(v) => v
                .first()
                .copied()
                .map(Value::Float64)
                .unwrap_or(Value::Empty(DataType::Float64)),
            MultiValues::BigInteger(v) => v
                .first()
                .cloned()
                .map(Value::BigInteger)
                .unwrap_or(Value::Empty(DataType::BigInteger)),
            MultiValues::BigDecimal(v) => v
                .first()
                .cloned()
                .map(Value::BigDecimal)
                .unwrap_or(Value::Empty(DataType::BigDecimal)),
            MultiValues::String(v) => v
                .first()
                .cloned()
                .map(Value::String)
                .unwrap_or(Value::Empty(DataType::String)),
            MultiValues::Date(v) => v
                .first()
                .copied()
                .map(Value::Date)
                .unwrap_or(Value::Empty(DataType::Date)),
            MultiValues::Time(v) => v
                .first()
                .copied()
                .map(Value::Time)
                .unwrap_or(Value::Empty(DataType::Time)),
            MultiValues::DateTime(v) => v
                .first()
                .copied()
                .map(Value::DateTime)
                .unwrap_or(Value::Empty(DataType::DateTime)),
            MultiValues::Instant(v) => v
                .first()
                .copied()
                .map(Value::Instant)
                .unwrap_or(Value::Empty(DataType::Instant)),
            MultiValues::Duration(v) => v
                .first()
                .copied()
                .map(Value::Duration)
                .unwrap_or(Value::Empty(DataType::Duration)),
            MultiValues::Url(v) => v
                .first()
                .cloned()
                .map(Value::Url)
                .unwrap_or(Value::Empty(DataType::Url)),
            MultiValues::StringMap(v) => v
                .first()
                .cloned()
                .map(Value::StringMap)
                .unwrap_or(Value::Empty(DataType::StringMap)),
            MultiValues::Json(v) => v
                .first()
                .cloned()
                .map(Value::Json)
                .unwrap_or(Value::Empty(DataType::Json)),
        }
    }

    /// Merge another multiple values
    ///
    /// Append all values from another multiple values to the current multiple values
    ///
    /// # Parameters
    ///
    /// * `other` - The multiple values to merge
    ///
    /// # Returns
    ///
    /// If types match, returns `Ok(())`; otherwise returns an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// let mut a = MultiValues::Int32(vec![1, 2]);
    /// let b = MultiValues::Int32(vec![3, 4]);
    /// a.merge(&b).unwrap();
    /// assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);
    /// ```
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }

        match (self, other) {
            (MultiValues::Bool(v), MultiValues::Bool(o)) => v.extend_from_slice(o),
            (MultiValues::Char(v), MultiValues::Char(o)) => v.extend_from_slice(o),
            (MultiValues::Int8(v), MultiValues::Int8(o)) => v.extend_from_slice(o),
            (MultiValues::Int16(v), MultiValues::Int16(o)) => v.extend_from_slice(o),
            (MultiValues::Int32(v), MultiValues::Int32(o)) => v.extend_from_slice(o),
            (MultiValues::Int64(v), MultiValues::Int64(o)) => v.extend_from_slice(o),
            (MultiValues::Int128(v), MultiValues::Int128(o)) => v.extend_from_slice(o),
            (MultiValues::UInt8(v), MultiValues::UInt8(o)) => v.extend_from_slice(o),
            (MultiValues::UInt16(v), MultiValues::UInt16(o)) => v.extend_from_slice(o),
            (MultiValues::UInt32(v), MultiValues::UInt32(o)) => v.extend_from_slice(o),
            (MultiValues::UInt64(v), MultiValues::UInt64(o)) => v.extend_from_slice(o),
            (MultiValues::UInt128(v), MultiValues::UInt128(o)) => v.extend_from_slice(o),
            (MultiValues::Float32(v), MultiValues::Float32(o)) => v.extend_from_slice(o),
            (MultiValues::Float64(v), MultiValues::Float64(o)) => v.extend_from_slice(o),
            (MultiValues::String(v), MultiValues::String(o)) => v.extend_from_slice(o),
            (MultiValues::Date(v), MultiValues::Date(o)) => v.extend_from_slice(o),
            (MultiValues::Time(v), MultiValues::Time(o)) => v.extend_from_slice(o),
            (MultiValues::DateTime(v), MultiValues::DateTime(o)) => v.extend_from_slice(o),
            (MultiValues::Instant(v), MultiValues::Instant(o)) => v.extend_from_slice(o),
            (MultiValues::BigInteger(v), MultiValues::BigInteger(o)) => v.extend_from_slice(o),
            (MultiValues::BigDecimal(v), MultiValues::BigDecimal(o)) => v.extend_from_slice(o),
            (MultiValues::IntSize(v), MultiValues::IntSize(o)) => v.extend_from_slice(o),
            (MultiValues::UIntSize(v), MultiValues::UIntSize(o)) => v.extend_from_slice(o),
            (MultiValues::Duration(v), MultiValues::Duration(o)) => v.extend_from_slice(o),
            (MultiValues::Url(v), MultiValues::Url(o)) => v.extend_from_slice(o),
            (MultiValues::StringMap(v), MultiValues::StringMap(o)) => v.extend(o.iter().cloned()),
            (MultiValues::Json(v), MultiValues::Json(o)) => v.extend(o.iter().cloned()),
            (slot @ MultiValues::Empty(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }

        Ok(())
    }
}

impl Default for MultiValues {
    #[inline]
    fn default() -> Self {
        MultiValues::Empty(DataType::String)
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        match value {
            Value::Empty(dt) => MultiValues::Empty(dt),
            Value::Bool(v) => MultiValues::Bool(vec![v]),
            Value::Char(v) => MultiValues::Char(vec![v]),
            Value::Int8(v) => MultiValues::Int8(vec![v]),
            Value::Int16(v) => MultiValues::Int16(vec![v]),
            Value::Int32(v) => MultiValues::Int32(vec![v]),
            Value::Int64(v) => MultiValues::Int64(vec![v]),
            Value::Int128(v) => MultiValues::Int128(vec![v]),
            Value::UInt8(v) => MultiValues::UInt8(vec![v]),
            Value::UInt16(v) => MultiValues::UInt16(vec![v]),
            Value::UInt32(v) => MultiValues::UInt32(vec![v]),
            Value::UInt64(v) => MultiValues::UInt64(vec![v]),
            Value::UInt128(v) => MultiValues::UInt128(vec![v]),
            Value::Float32(v) => MultiValues::Float32(vec![v]),
            Value::Float64(v) => MultiValues::Float64(vec![v]),
            Value::String(v) => MultiValues::String(vec![v]),
            Value::Date(v) => MultiValues::Date(vec![v]),
            Value::Time(v) => MultiValues::Time(vec![v]),
            Value::DateTime(v) => MultiValues::DateTime(vec![v]),
            Value::Instant(v) => MultiValues::Instant(vec![v]),
            Value::BigInteger(v) => MultiValues::BigInteger(vec![v]),
            Value::BigDecimal(v) => MultiValues::BigDecimal(vec![v]),
            Value::IntSize(v) => MultiValues::IntSize(vec![v]),
            Value::UIntSize(v) => MultiValues::UIntSize(vec![v]),
            Value::Duration(v) => MultiValues::Duration(vec![v]),
            Value::Url(v) => MultiValues::Url(vec![v]),
            Value::StringMap(v) => MultiValues::StringMap(vec![v]),
            Value::Json(v) => MultiValues::Json(vec![v]),
        }
    }
}
