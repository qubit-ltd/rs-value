# qubit-value Architecture and Wire Design

[中文版](design.zh_CN.md) · [README](../README.md) · [User guide](user_guide.md) · [API documentation](https://docs.rs/qubit-value)

This document records the architectural boundaries and compatibility rules for
`qubit-value` 0.12. It is aimed at maintainers of this crate and authors of
protocols or downstream crates that build on its value model.

<a id="scope"></a>
## Scope

`qubit-value` is the typed runtime value layer shared by higher-level key-value
systems. It owns four responsibilities:

- representing one typed scalar, one homogeneous typed collection, and the
  explicit distinction between those shapes;
- providing strict reads and opt-in policy-based conversion;
- defining semantic equality and hashing for every supported runtime type;
- adapting runtime values to lossy Natural JSON or the type-preserving Wire V1
  contract.

The crate does not own property-name validation, schemas, source loading,
configuration precedence, persistence, transport framing, or distributed
identity. Those belong to downstream systems. `NamedValue` and
`NamedMultiValues` are lightweight name wrappers, not a general key-value
store.

The public containers and adapters are stable API surfaces. Their private
storage enums and serialization helpers are implementation details and may be
reorganized without changing observable behavior.

<a id="type-model"></a>
## Type Model

The model has two independent dimensions: runtime `DataType` and value shape.
Keeping them independent prevents collection length or JSON syntax from
silently changing a caller's contract.

| Public type | Role | Shape |
| --- | --- | --- |
| `Value` | Owns one concrete typed payload or `Unset(DataType)` | Scalar |
| `MultiValues` | Owns one homogeneous vector or an unset element type | Collection |
| `ValueContainer` | Wraps `Scalar(Value)` or `Collection(MultiValues)` | Explicit union |
| `NamedValue` | Associates a name with `Value` | Named scalar |
| `NamedMultiValues` | Associates a name with `MultiValues` | Named collection |

The closed internal value table is the single mapping between the 25 supported
`DataType` variants, Rust storage types, feature gates, Natural JSON classes,
and stable Wire V1 tags. It drives the scalar and collection representations,
constructors, borrowed projections, identity logic, and Wire DTOs. This keeps
the two container families aligned without making the generated storage enums
public.

`ValueContainer` never infers shape. In particular, a scalar `Int32(42)`, a
one-item `Int32` collection, an empty `Int32` collection, and an unset `Int32`
collection are four distinct states.

<a id="ownership-and-borrowed-views"></a>
## Ownership and Borrowed Views

`Value`, `MultiValues`, and `ValueContainer` own their payloads and expose
stable methods rather than their private representations. Cloning one of these
types clones owned payloads such as strings, maps, JSON trees, and collection
vectors.

`Value::view()` and `MultiValues::view()` return `ValueRef<'_>` and
`MultiValuesRef<'_>`. These non-exhaustive enums expose semantic variants while
borrowing non-copy payloads from the source. Copy-sized scalar values are
projected by value; strings, large values, maps, JSON trees, and collection
slices remain borrowed. Downstream matches must retain a wildcard arm because
the borrowed-view enums may grow.

The closed value table generates both views and their owned-to-borrowed
dispatch. `MultiValuesRef` exposes `data_type`, `len`, `is_empty`, and indexed
`get`, returning a `ValueRef` without building a temporary scalar container.
With `converter`, `DataConverter` accepts `ValueRef` directly. This lets
configuration Serde visitors convert original numeric types and borrow native
collections without first materializing a Natural JSON tree.

Wire ownership is also explicit:

| Adapter | Ownership | Intended use |
| --- | --- | --- |
| `ValueWireV1` | Owns a standalone versioned envelope | Store or transfer a complete Wire value |
| `ValueWirePayloadV1` | Owns an unversioned typed shape | Embed under an outer protocol version |
| `ValueWireRefV1<'a>` | Borrows a standalone envelope payload | Encode without cloning the runtime value |
| `ValueWirePayloadRefV1<'a>` | Borrows an unversioned payload | Embed and encode without cloning |

Borrowed Wire constructors are fallible. Before exposing a serializable view,
they validate invariants that JSON cannot represent, including non-finite
floats and an out-of-range `BigDecimal` scale. Their lifetime cannot outlive
the source runtime value.

<a id="value-semantics"></a>
## Value Semantics

Unset is a typed absence, not a typeless null. `Value::new_unset(T)` and
`MultiValues::new_unset(T)` preserve `T` so schema-aware callers can retain the
declared type before a concrete payload exists.

The following states are deliberately different:

| State | Meaning |
| --- | --- |
| `Value::new_unset(DataType::Json)` | No scalar value; declared type is `Json` |
| `MultiValues::new_unset(DataType::String)` | No collection; declared element type is `String` |
| `MultiValues::String(Vec::new())` | A concrete, homogeneous, empty collection |
| `Value::Json(serde_json::Value::Null)` | A concrete JSON scalar whose payload is JSON `null` |

Strict getters require the stored variant to match the requested Rust type.
They report a structured missing state or `TypeMismatch`; they never perform an
implicit conversion. Conversion methods are available only with `converter`
and delegate policy and resource accounting to `qubit-datatype`.

Defaulting is narrow. Strict `get_or` methods default an unset value, while
conversion defaults may also apply when the selected conversion policy reports
the source as missing. A concrete empty collection, type mismatch, or ordinary
invalid conversion is not silently replaced unless the relevant API explicitly
defines that behavior.

`set` replaces the current scalar or complete collection. `MultiValues::add`
is fallible because appended values must have the same element type. Shape is
preserved independently of item count throughout access, conversion, and Wire
serialization.

<a id="identity-and-hashing"></a>
## Identity and Hashing

Equality and hashing form one semantic contract across `Value`,
`MultiValues`, and `ValueContainer`:

- the runtime variant and scalar-versus-collection shape participate in
  identity;
- collection order and length participate in identity;
- `+0.0` and `-0.0` are one floating-point identity, and all NaN bit patterns
  are canonicalized to one identity so `Eq` and `Hash` remain lawful;
- `BigDecimal` hashing normalizes coefficient trailing zeroes and effective
  scale to agree with numeric equality;
- `StringMap` and JSON object identity are independent of map iteration order;
- JSON array order remains significant.

JSON equality and hashing use iterative traversal instead of recursive calls.
The budget-aware hashing APIs preflight the complete JSON payload in a staged
transaction. A budget error leaves both the caller's hasher and the committed
budget unchanged.

The resulting `Hash` output is for in-memory Rust collections. It is not a
persistent fingerprint, canonical protocol digest, or distributed-cache key;
it may change with the hasher, target, dependency versions, features, or crate
implementation.

<a id="natural-json-and-wire-json"></a>
## Natural JSON and Wire JSON

Natural JSON and Wire JSON solve different boundary problems.

| Property | Natural JSON | Wire V1 JSON |
| --- | --- | --- |
| Primary goal | Interoperate with ordinary JSON consumers | Reconstruct runtime type and shape |
| Type tags | Omitted | Preserved |
| Scalar/collection shape | Expressed only by JSON scalar/array syntax | Explicit `scalar` or `collection` tag |
| Unset | Projects to JSON `null` | Preserves `Unset(DataType)` |
| Round trip | Intentionally lossy | Type- and shape-preserving within V1 support |
| Entry point | `to_json_value*` | `ValueWire*V1` plus bounded encode/decode helpers |

Natural JSON is a projection into `serde_json::Value`. It sorts string-map and
JSON object keys for deterministic output, rejects non-finite floats, and
applies one conversion budget across a complete projection. It cannot recover
the original integer width, distinguish unset from concrete JSON null, or
recover a declared type from JSON alone.

Projection preparation borrows source text and admits it before allocating the
final JSON string. Numeric measurement uses bounded formatting without a heap
string; formatted rich values are cached for the final projection, so conversion
is charged once. Preparation failure produces no usable partial result. These
changes preserve the existing Natural JSON categories and resource limits.

Wire JSON uses explicit version, shape, and type tags. The DTOs implement
`Serialize`, but intentionally do not implement generic `Deserialize`.
Complete untrusted JSON documents must use bounded decode helpers; embedded
payloads use `ValueWireV1Seed` or `ValueWirePayloadV1Seed` under the outer
decoder's shared session.

<a id="wire-v1-compatibility"></a>
## Wire V1 Compatibility

A standalone V1 document has a numeric `version` field and one typed `value`.
An embedded payload omits only the outer version field. Representative values
are:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"string"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
```

V1 is a closed contract. The following are compatibility invariants:

- the version is the JSON number `1`;
- the shape is exactly one `scalar` or `collection` variant;
- the payload is exactly one known lower-case V1 type tag;
- unknown fields, shapes, tags, and unsupported feature-gated concrete
  payloads are rejected;
- `Int128`, `UInt128`, and `BigInteger` use canonical decimal text;
- `BigDecimal` uses a bounded coefficient/scale representation;
- `Duration` uses seconds and sub-second nanoseconds;
- floats must be finite;
- string-map keys and nested JSON object keys are emitted in lexicographic
  order under the supported canonical JSON configuration.

The pre-0.11 externally tagged representation is not V1 input. Existing V1
tags, shapes, or payload encodings must not be repurposed or extended in place.
A new runtime type or incompatible representation requires a new Wire version
and an explicit migration path.

The byte-stability statement applies to canonical JSON emitted with the
supported `serde_json` configuration. Other Serde formats may carry the DTOs,
but their byte representation is not part of the V1 JSON stability contract.

<a id="budget-and-preflight"></a>
## Budget and Preflight

Wire processing separates early rejection from authoritative accounting.

`ValueWireEncodePreflight` performs conservative lower-bound checks before a
downstream protocol sorts keys, formats payloads, or allocates its final output.
Successful `check_value`, `check_values`, and `check_container` calls
accumulate into one checker, allowing an outer object to charge several
embedded values to a shared budget. Each call is atomic: when it fails, the
node, payload-byte, and output-byte counters return to their pre-call state.

Preflight does not serialize, retain a sorting index, or prove that final
encoding will fit. Its estimates intentionally omit or undercount some final
syntax and formatting costs. The final `JsonEncodeSession` remains
authoritative and enforces structural measurements plus actual output bytes
during encoding.

`new_value_limits` is for an outer protocol that owns the output budget.
`new_u64_limits` adapts a downstream `u64` profile to native `usize`; limits
larger than `usize::MAX` saturate rather than truncate on narrower targets.

Standalone defaults limit both raw input or output to 1 MiB and apply the
shared value profile: depth 64, 100,000 nodes, 4,096 sequence items, 4,096 map
entries, 256 KiB keys, 256 KiB strings, 4,096-byte number text, and 1 MiB value
payload. Applications may pass stricter profiles. For embedded data, the outer
protocol must own one session so envelope and payload resources are charged
together.

<a id="feature-matrix"></a>
## Feature Matrix

The default feature set is empty. Features affect which concrete payloads and
APIs can be materialized, but every `DataType` can still appear in an unset
declaration.

| Feature | Architectural effect |
| --- | --- |
| `converter` | Enables policy-based scalar and collection conversion |
| `chrono` | Materializes `Date`, `Time`, `DateTime`, and `Instant` |
| `big-integer` | Materializes `BigInteger` |
| `big-decimal` | Materializes `BigDecimal` and its bounded Wire form |
| `big-number` | Compatibility alias for both big-number features |
| `url` | Materializes `Url` |
| `json` | Materializes `Json` and enables bounded JSON Wire helpers and preflight |
| `natural-json` | Convenience alias for `converter` plus `json`; enables Natural JSON projection |
| `redact` | Enables policy-aware redacted views |
| `all` | Enables every public feature family |

A producer and consumer exchanging a concrete optional payload must enable the
same required type feature. A decoder without that feature rejects the payload
instead of coercing it to another type. Feature-gated APIs must not become an
implicit part of a minimal-feature build.

<a id="error-model"></a>
## Error Model

Errors are structured by boundary:

| Error type | Boundary |
| --- | --- |
| `ValueError` | Strict access, conversion, and Natural JSON projection |
| `ValueMissing` | Typed reason carried by `ValueError::Missing` |
| `ValueWireEncodeError` | V1 validation, budget, JSON syntax/serialization, and writer I/O |
| `ValueWireDecodeError` | Bounded input, JSON syntax/data, unsupported version, and resource failures |
| `MeasuredBudgetError<JsonResource, usize>` | Direct preflight rejection before downstream encoding |

Missing storage, a concrete empty collection, a conversion classified as
missing, type mismatch, and invalid conversion remain distinguishable.
Collection conversion errors retain the failing source index. Natural JSON
projection limit errors retain the data type, optional collection index, and
the measured resource facts.

`ValueMissing` is a private-field fact object, classified by
`ValueMissingReason`. It records source type, requested target type, optional
source index, and the original conversion error when present. Storage
classification and conversion provenance are independent: an unset read may
still carry its original conversion failure. Callers use
`is_defaultable_for_strict_read()` or `is_defaultable_for_conversion()` rather
than assuming every missing result permits a fallback. A missing collection
item and a first-item read from a concrete empty collection never default.

Wire errors do not include raw input contents. Decode errors preserve safe
location and category information. Both `ValueError` and Wire error enums are
non-exhaustive where downstream code must tolerate future diagnostic variants;
downstream matches therefore need a fallback arm.

<a id="downstream-integration"></a>
## Downstream Integration

`rs-config` uses `ValueContainer` as its property payload so configuration
sources and readers preserve scalar-versus-collection shape. Strict reads use
`StrictValueRead` at the value boundary; `Config::get` remains a converting
read. Standalone property values use `ValueWireV1` or
`ValueWireRefV1`. Before encoding a complete configuration, one
`ValueWireEncodePreflight` accumulates conservative charges for all properties
using the configuration's `u64` limit profile.

`rs-metadata` stores scalar `Value` instances and embeds
`ValueWirePayloadV1`/`ValueWirePayloadRefV1` inside its own versioned metadata
and filter protocols. Its decoder uses `ValueWirePayloadV1Seed` so the outer
document owns one decode session. Metadata and filter encoders likewise reuse
one preflight checker across contained values.

Metadata's `get` family uses strict reads, while its `convert` family requests
conversion explicitly. Both downstream crates preserve `ValueError` sources
and reuse `IntoValueDefault`; callers must not infer identical read semantics
from the shared method name `get`.

These integrations define the intended layering:

1. the downstream crate owns names, domain rules, outer versioning, and the
   complete request budget;
2. `qubit-value` owns typed payload validation, scalar/collection semantics,
   and V1 payload encoding;
3. `qubit-budget` and `qubit-json` perform shared bounded JSON accounting and
   I/O.

New public behavior should be checked against both direct downstream crates,
especially changes to borrowed views, missing semantics, Wire DTOs, seeds,
preflight accumulation, or feature gates.

<a id="evolution-rules"></a>
## Evolution Rules

Maintainers should apply the following rules to future changes:

1. Preserve public method signatures and semantics within a compatible release;
   use semver checks and direct downstream tests to detect accidental breaks.
2. Add a runtime type through the central value table and update both owned
   containers, borrowed views, conversion behavior, identity, Natural JSON,
   Wire handling, feature tests, and documentation as one change.
3. Preserve `Eq`/`Hash` consistency. Treat changes to semantic equality as
   observable API changes even though concrete hash output is not stable.
4. Do not change V1 tags, shapes, or payload representations in place. Introduce
   a new version and document migration for incompatible protocol changes.
5. Keep generic `Deserialize` off the public Wire DTOs. New decode paths must
   preserve bounded complete-document or shared-session accounting.
6. Keep preflight conservative, cumulative, and per-call atomic. The final
   encoder remains the source of truth for resource admission.
7. Keep Natural JSON explicitly lossy and separate from Wire compatibility.
   Do not infer runtime type or shape during a Natural JSON round trip.
8. Keep the default feature set minimal. Optional concrete types require an
   explicit producer/consumer feature agreement.
9. Treat changes to default resource profiles and canonical JSON ordering as
   compatibility-sensitive behavior, with focused tests and release notes.
10. Keep English and Simplified Chinese README, user guide, and design
    documents semantically aligned whenever a contract changes.
