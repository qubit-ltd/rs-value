# qubit-value User Guide

## Dependency

```toml
qubit-value = { version = "0.10", features = ["all"] }
```

The default feature set is empty. Enable only `chrono`, `big-number`, `url`,
`json`, or `converter` when the application does not need all families.

## Runtime shapes

`Value` stores one typed scalar. `MultiValues` stores one homogeneous typed
collection. `ValueContainer` preserves an explicit `Scalar` or `Collection`
shape; a one-item collection never becomes a scalar. `Unset(DataType)` is
different from a concrete value and from a concrete empty collection.

## Type-preserving Wire V1

Direct Serde uses `ValueWireV1`:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"int32"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

V1 compatibility covers this JSON object structure. Other Serde formats may
work, but their format-specific representations are outside the stability
contract.

`Value` accepts scalar only, `MultiValues` accepts collection only, and
`ValueContainer` accepts either. The envelope requires numeric version `1` and
rejects unknown fields, unknown types, wrong shapes, and all pre-0.10 payloads.
Wide integers and big numbers use canonical decimal strings. `Duration` uses
secs/nanos. Non-finite floats are rejected. `Json(null)` is concrete and
distinct from `Unset(Json)`.

Owned adapters are available through `From<Value>`, `From<MultiValues>`, and
`From<ValueContainer>` for `ValueWireV1`, and `From<ValueWireV1>` for
`ValueContainer`.

## Natural JSON

With `converter`, `to_json_value()` emits ordinary application JSON without
runtime type tags. Use Wire V1 whenever the receiver must reconstruct the
exact runtime type and shape.
