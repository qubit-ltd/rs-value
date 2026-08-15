// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy-aware redaction for structured [`super::Value`] instances.

use std::fmt;

use qubit_redact::MaskingPolicy;
#[cfg(feature = "json")]
use qubit_redact::RedactionCompletion;
use qubit_redact::Sensitivity;
use qubit_redact::domain::DomainTruncated;
use qubit_redact::domain::Redact;
use qubit_redact::domain::RedactValue;
use qubit_redact::domain::RedactedMapResult;
use qubit_redact::domain::RedactedResult;
use qubit_redact::domain::RedactedValue;
use qubit_redact::policy::DomainTraversalAdmission;
use qubit_redact::policy::DomainValueAdmission;
use qubit_redact::policy::DomainValueScope;
use qubit_redact::policy::RedactionSession;

use super::Value;
use super::ValueRepr;
use crate::MultiValues;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::ValueContainer;
use crate::multi_values::MultiValuesRepr;

impl RedactValue for Value {
    /// Redacts string contents while replacing every other variant opaquely.
    fn redact_value<'a>(
        &'a self,
        level: Sensitivity,
        masking: &MaskingPolicy,
    ) -> RedactedValue<'a> {
        match &self.repr {
            ValueRepr::String(value) => value.redact_value(level, masking),
            _ => RedactedValue::opaque(level, masking),
        }
    }
}

impl RedactValue for MultiValues {
    /// Replaces a sensitive collection without formatting its contents.
    #[inline(always)]
    fn redact_value<'a>(
        &'a self,
        level: Sensitivity,
        masking: &MaskingPolicy,
    ) -> RedactedValue<'a> {
        RedactedValue::opaque(level, masking)
    }
}

/// Formats one typed collection after charging each item before access.
///
/// The exact slice length is checked before attempting the next structural
/// admission, so a collection that exactly fills its item budget completes
/// without a false marker. Once admission fails, the function writes one safe
/// marker and stops without advancing the iterator or formatting the rejected
/// item. The enclosing variant name is preserved in ordinary compact output.
///
/// # Parameters
///
/// * `variant` - Stable [`MultiValuesRepr`] variant name.
/// * `values` - Homogeneous values to format.
/// * `scope` - Active scope that owns the shared collection-item budget.
/// * `formatter` - Destination formatting context.
///
/// # Errors
///
/// Returns [`fmt::Error`] when the destination rejects the variant, an
/// admitted item, the truncation marker, or the closing delimiter.
fn fmt_debug_collection<T: fmt::Debug>(
    variant: &str,
    values: &[T],
    scope: &mut DomainValueScope<'_, '_>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    formatter.write_str(variant)?;
    formatter.write_str("(")?;
    let mut output = formatter.debug_list();
    let mut values = values.iter();
    loop {
        if scope.session().is_exhausted() || values.len() == 0 {
            break;
        }
        if scope.admit_collection_item()
            == DomainTraversalAdmission::LimitReached
        {
            output.entry(&DomainTruncated);
            break;
        }
        let Some(value) = values.next() else {
            break;
        };
        output.entry(value);
    }
    output.finish()?;
    formatter.write_str(")")
}

/// Formats string-map collection items through the shared redaction session.
///
/// Collection admission always precedes iterator advancement. Each admitted
/// map then performs its own node and entry accounting through
/// [`RedactedMapResult`], so all nested work shares the caller's budgets.
///
/// # Parameters
///
/// * `values` - String maps to redact in source order.
/// * `scope` - Active collection scope and shared session owner.
/// * `formatter` - Destination formatting context.
///
/// # Errors
///
/// Returns [`fmt::Error`] when the destination rejects safe output.
fn fmt_map_collection(
    values: &[std::collections::HashMap<String, String>],
    scope: &mut DomainValueScope<'_, '_>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    formatter.write_str("StringMap(")?;
    let mut output = formatter.debug_list();
    let mut values = values.iter();
    loop {
        if scope.session().is_exhausted() || values.len() == 0 {
            break;
        }
        if scope.admit_collection_item()
            == DomainTraversalAdmission::LimitReached
        {
            output.entry(&DomainTruncated);
            break;
        }
        let Some(value) = values.next() else {
            break;
        };
        output.entry(&RedactedMapResult::new(value, scope.session()));
    }
    output.finish()?;
    formatter.write_str(")")
}

/// Formats JSON collection items with structural and adapter byte admission.
///
/// The collection-item budget is charged before the slice iterator advances.
/// The JSON adapter then charges the exact encoded input for the admitted item;
/// a rejected collection item is never passed to that adapter. Complete and
/// non-empty truncated fragments are written verbatim. An exhausted fragment
/// is replaced by one structural marker and terminates iteration, so empty
/// adapter text cannot masquerade as an absent JSON item.
///
/// # Parameters
///
/// * `values` - JSON values to redact in source order.
/// * `scope` - Active collection scope and shared session owner.
/// * `formatter` - Destination formatting context.
///
/// # Errors
///
/// Returns [`fmt::Error`] when the destination rejects safe output.
#[cfg(feature = "json")]
fn fmt_json_collection(
    values: &[serde_json::Value],
    scope: &mut DomainValueScope<'_, '_>,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    formatter.write_str("Json([")?;
    let mut values = values.iter();
    let mut first = true;
    loop {
        if scope.session().is_exhausted() || values.len() == 0 {
            break;
        }
        if scope.admit_collection_item()
            == DomainTraversalAdmission::LimitReached
        {
            if !first {
                formatter.write_str(", ")?;
            }
            fmt::Debug::fmt(&DomainTruncated, formatter)?;
            break;
        }
        let Some(value) = values.next() else {
            break;
        };
        let redacted = scope.session().json().redact_value(value);
        if !first {
            formatter.write_str(", ")?;
        }
        match redacted.completion() {
            RedactionCompletion::Complete | RedactionCompletion::Truncated => {
                formatter.write_str(redacted.as_str())?;
                first = false;
            }
            RedactionCompletion::Exhausted => {
                fmt::Debug::fmt(&DomainTruncated, formatter)?;
                break;
            }
        }
    }
    formatter.write_str("])")
}

impl Redact for Value {
    /// Writes this value through the session without altering ordinary debug
    /// representation when no key context is available.
    ///
    /// The value node and its private representation field are admitted before
    /// the representation is inspected. String maps classify every entry by
    /// key, JSON uses exact adapter input charging, and scalar variants retain
    /// their ordinary debug representation. A rejected value or field emits a
    /// safe structural marker without reading the payload.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let DomainValueAdmission::Entered(mut scope) =
            session.enter_domain_value()
        else {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        };
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        }
        match &self.repr {
            ValueRepr::StringMap(values) => fmt::Debug::fmt(
                &RedactedMapResult::new(values, scope.session()),
                formatter,
            ),
            #[cfg(feature = "json")]
            ValueRepr::Json(value) => {
                let redacted = scope.session().json().redact_value(value);
                match redacted.completion() {
                    RedactionCompletion::Complete
                    | RedactionCompletion::Truncated => {
                        formatter.write_str(redacted.as_str())
                    }
                    RedactionCompletion::Exhausted => {
                        fmt::Debug::fmt(&DomainTruncated, formatter)
                    }
                }
            }
            _ => fmt::Debug::fmt(self, formatter),
        }
    }
}

impl Redact for MultiValues {
    /// Writes each admitted collection item through one shared session.
    ///
    /// The collection node and private representation field are charged before
    /// inspection. Every non-empty slice checks its exact remaining length,
    /// charges one collection item, and only then advances the iterator. Map
    /// and JSON items delegate to their adapters after admission; ordinary
    /// items retain their typed variant and debug representation.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let DomainValueAdmission::Entered(mut scope) =
            session.enter_domain_value()
        else {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        };
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        }
        match &self.repr {
            MultiValuesRepr::Unset(_) => fmt::Debug::fmt(self, formatter),
            MultiValuesRepr::Bool(values) => {
                fmt_debug_collection("Bool", values, &mut scope, formatter)
            }
            MultiValuesRepr::Char(values) => {
                fmt_debug_collection("Char", values, &mut scope, formatter)
            }
            MultiValuesRepr::Int8(values) => {
                fmt_debug_collection("Int8", values, &mut scope, formatter)
            }
            MultiValuesRepr::Int16(values) => {
                fmt_debug_collection("Int16", values, &mut scope, formatter)
            }
            MultiValuesRepr::Int32(values) => {
                fmt_debug_collection("Int32", values, &mut scope, formatter)
            }
            MultiValuesRepr::Int64(values) => {
                fmt_debug_collection("Int64", values, &mut scope, formatter)
            }
            MultiValuesRepr::Int128(values) => {
                fmt_debug_collection("Int128", values, &mut scope, formatter)
            }
            MultiValuesRepr::UInt8(values) => {
                fmt_debug_collection("UInt8", values, &mut scope, formatter)
            }
            MultiValuesRepr::UInt16(values) => {
                fmt_debug_collection("UInt16", values, &mut scope, formatter)
            }
            MultiValuesRepr::UInt32(values) => {
                fmt_debug_collection("UInt32", values, &mut scope, formatter)
            }
            MultiValuesRepr::UInt64(values) => {
                fmt_debug_collection("UInt64", values, &mut scope, formatter)
            }
            MultiValuesRepr::UInt128(values) => {
                fmt_debug_collection("UInt128", values, &mut scope, formatter)
            }
            MultiValuesRepr::Float32(values) => {
                fmt_debug_collection("Float32", values, &mut scope, formatter)
            }
            MultiValuesRepr::Float64(values) => {
                fmt_debug_collection("Float64", values, &mut scope, formatter)
            }
            #[cfg(feature = "big-integer")]
            MultiValuesRepr::BigInteger(values) => fmt_debug_collection(
                "BigInteger",
                values,
                &mut scope,
                formatter,
            ),
            #[cfg(feature = "big-decimal")]
            MultiValuesRepr::BigDecimal(values) => fmt_debug_collection(
                "BigDecimal",
                values,
                &mut scope,
                formatter,
            ),
            MultiValuesRepr::String(values) => {
                fmt_debug_collection("String", values, &mut scope, formatter)
            }
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Date(values) => {
                fmt_debug_collection("Date", values, &mut scope, formatter)
            }
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Time(values) => {
                fmt_debug_collection("Time", values, &mut scope, formatter)
            }
            #[cfg(feature = "chrono")]
            MultiValuesRepr::DateTime(values) => {
                fmt_debug_collection("DateTime", values, &mut scope, formatter)
            }
            #[cfg(feature = "chrono")]
            MultiValuesRepr::Instant(values) => {
                fmt_debug_collection("Instant", values, &mut scope, formatter)
            }
            MultiValuesRepr::Duration(values) => {
                fmt_debug_collection("Duration", values, &mut scope, formatter)
            }
            #[cfg(feature = "url")]
            MultiValuesRepr::Url(values) => {
                fmt_debug_collection("Url", values, &mut scope, formatter)
            }
            MultiValuesRepr::StringMap(values) => {
                fmt_map_collection(values, &mut scope, formatter)
            }
            #[cfg(feature = "json")]
            MultiValuesRepr::Json(values) => {
                fmt_json_collection(values, &mut scope, formatter)
            }
        }
    }
}

impl Redact for ValueContainer {
    /// Delegates an admitted variant payload to the shared session.
    ///
    /// The wrapper first charges its own node. Each enum payload is read only
    /// after one field admission, then the child enters another value scope so
    /// node and depth limits accumulate instead of being reset.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let DomainValueAdmission::Entered(mut scope) =
            session.enter_domain_value()
        else {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        };
        match self {
            Self::Scalar(value) => {
                if scope.admit_field() == DomainTraversalAdmission::LimitReached
                {
                    return fmt::Debug::fmt(&DomainTruncated, formatter);
                }
                value.fmt_redacted(scope.session(), formatter)
            }
            Self::Collection(values) => {
                if scope.admit_field() == DomainTraversalAdmission::LimitReached
                {
                    return fmt::Debug::fmt(&DomainTruncated, formatter);
                }
                values.fmt_redacted(scope.session(), formatter)
            }
        }
    }
}

impl Redact for NamedValue {
    /// Uses the admitted wrapper name to classify its admitted value.
    ///
    /// The wrapper node, name field, and value field are charged in source
    /// order. A failed field admission stops before invoking the corresponding
    /// accessor. Classification is resolved directly for the already-admitted
    /// value field: sensitive values are masked without another structural
    /// charge, while pass-through values enter their own legitimate child
    /// scope through [`RedactedResult`].
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let DomainValueAdmission::Entered(mut scope) =
            session.enter_domain_value()
        else {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        };
        let mut output = formatter.debug_struct("NamedValue");
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return output.field("...", &DomainTruncated).finish();
        }
        output.field("name", &self.name());
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return output.field("...", &DomainTruncated).finish();
        }
        let sensitivity = scope.session().policy().sensitivity_for(self.name());
        match sensitivity {
            Some(level) => {
                let redacted = self
                    .value()
                    .redact_value(level, scope.session().policy().masking());
                output.field("value", &redacted).finish()
            }
            None => {
                let redacted =
                    RedactedResult::new(self.value(), scope.session());
                output.field("value", &redacted).finish()
            }
        }
    }
}

impl Redact for NamedMultiValues {
    /// Uses the admitted wrapper name to classify its admitted collection.
    ///
    /// Like [`NamedValue`], this wrapper performs all field admission before
    /// accessor calls. It resolves the already-admitted value directly, so a
    /// sensitive collection is masked without charging a duplicate keyed root
    /// and field; pass-through traversal still enters the collection itself.
    fn fmt_redacted(
        &self,
        session: &mut RedactionSession<'_>,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let DomainValueAdmission::Entered(mut scope) =
            session.enter_domain_value()
        else {
            return fmt::Debug::fmt(&DomainTruncated, formatter);
        };
        let mut output = formatter.debug_struct("NamedMultiValues");
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return output.field("...", &DomainTruncated).finish();
        }
        output.field("name", &self.name());
        if scope.admit_field() == DomainTraversalAdmission::LimitReached {
            return output.field("...", &DomainTruncated).finish();
        }
        let sensitivity = scope.session().policy().sensitivity_for(self.name());
        match sensitivity {
            Some(level) => {
                let redacted = self
                    .values()
                    .redact_value(level, scope.session().policy().masking());
                output.field("value", &redacted).finish()
            }
            None => {
                let redacted =
                    RedactedResult::new(self.values(), scope.session());
                output.field("value", &redacted).finish()
            }
        }
    }
}
