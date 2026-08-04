# qubit-value User Guide

## Dependency

```toml
qubit-value = { version = "0.10", features = ["all"] }
qubit-redact = { version = "0.4", default-features = false }
```

The default feature set is empty. Enable only `chrono`, `big-integer`,
`big-decimal`, `url`, `json`, `converter`, or `redact` when the application
does not need all families. `big-number` remains a compatibility alias for
both big number families.

## Runtime shapes

`Value` stores one typed scalar. `MultiValues` stores one homogeneous typed
collection. `ValueContainer` preserves an explicit `Scalar` or `Collection`
shape; a one-item collection never becomes a scalar. `Unset(DataType)` is
different from a concrete value and from a concrete empty collection.

## Policy-aware redaction

The `redact` feature implements `qubit_redact::Redact` for `Value`. Import the
trait from its owning crate and explicitly format a redacted view:

```rust
use std::collections::HashMap;

use qubit_redact::{
    Redact as _,
    RedactionPolicy,
    Sensitivity,
};
use qubit_value::Value;

let value = Value::StringMap(HashMap::from([
    ("api_key".to_owned(), "raw-secret".to_owned()),
    ("label".to_owned(), "visible".to_owned()),
]));
let mut builder = RedactionPolicy::default().to_builder();
builder
    .fields()
    .raise("api_key", Sensitivity::Secret)
    .expect("redaction field should be valid");
let policy = builder.build().expect("redaction policy should build");
let output = format!("{:?}", value.redacted_with(&policy));

assert!(!output.contains("raw-secret"));
assert!(output.contains("visible"));
```

String maps classify each value by its key. With both `redact` and `json`,
JSON objects and arrays are traversed recursively; a sensitive key with a
non-string value is replaced as a whole. Scalars without key context retain
ordinary `Debug` formatting. Ordinary `Value` formatting is not implicitly
redacted, so diagnostics must explicitly use a redacted view.

## Type-preserving Wire V1

Direct Serde uses `ValueWireV1`:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"int32"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

The canonical JSON V1 representation is byte-stable for the same value under
the supported `serde_json` version and configuration. Every Serde serializer
receives `StringMap` entries in ascending lexicographic (dictionary) key order;
nested JSON object keys follow the same recursive ordering. Other Serde formats
are outside the V1 byte-level stability contract.

V1 is closed. Existing tags, shapes, and payload representations cannot
change, and a future runtime data type requires a new wire version instead of
extending V1.

This structural guarantee does not make every concrete type available under
every feature set. A concrete rich-type tag can be deserialized only when the
receiving build enables its feature: `chrono` for date/time values,
`big-integer` or `big-decimal` for big numbers, `url` for URLs, and `json` for
JSON values. Producers and consumers exchanging those payloads should agree on
the required features; an unsupported concrete tag is rejected. An `unset`
payload may still preserve a declared `DataType` without enabling the feature
needed to hold a concrete value of that type.

`Value` accepts scalar only, `MultiValues` accepts collection only, and
`ValueContainer` accepts either. The envelope requires numeric version `1` and
rejects unknown fields, unknown types, wrong shapes, and all pre-0.10 payloads.
Wide integers use canonical decimal strings. `BigDecimal` uses an exact
`{"coefficient":"...","scale":i64}` payload. `Duration` uses
secs/nanos. Non-finite floats are rejected. `Json(null)` is concrete and
distinct from `Unset(Json)`.

Borrowed payloads must be created with `ValueWirePayloadRefV1::from_value`,
`from_values`, `from_container`, or `TryFrom`. These fallible constructors
validate finite floats and bounded `BigDecimal` scales before serialization;
the internal payload representation is private, so an unchecked wire shape
cannot be constructed by callers.

Owned adapters are available through `From<Value>`, `From<MultiValues>`, and
`From<ValueContainer>` for `ValueWireV1`, and `From<ValueWireV1>` for
`ValueContainer`.

`ValueWireV1::decode_json_slice()` and
`ValueWireV1::decode_json_slice_with_limits()` accept complete top-level V1
documents and enforce a byte budget before parsing. When a value is embedded
in a larger JSON document, call `ValueWireLimits::check_json_bytes()` with the
complete outer input length before invoking that document's Serde decoder.

## Natural JSON

With both `converter` and `json`, `to_json_value()` emits ordinary application
JSON without runtime type tags and recursively orders object keys. Use Wire V1
whenever the receiver must reconstruct the exact runtime type and shape.
