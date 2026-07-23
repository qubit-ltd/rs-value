# qubit-value 用户指南

## 依赖

```toml
qubit-value = { version = "0.10", features = ["all"] }
qubit-redact = { version = "0.3", default-features = false }
```

默认 feature 集为空。不需要全部类型族时，只启用 `chrono`、`big-integer`、
`big-decimal`、`url`、`json`、`converter` 或 `redact`。`big-number`
继续作为两个大数类型族的兼容别名。

## 运行时形态

`Value` 保存一个带类型标量，`MultiValues` 保存一个同类型集合。
`ValueContainer` 显式保留 `Scalar` 或 `Collection`；单元素集合不会变成标量。
`Unset(DataType)` 与具体值、具体空集合均不同。

## 按策略脱敏

`redact` feature 为 `Value` 实现 `qubit_redact::Redact`。调用方应从该 trait
所属 crate 导入它，并显式格式化脱敏视图：

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
let policy = RedactionPolicy::empty_builder()
    .raise("api_key", Sensitivity::Secret)
    .build()
    .expect("redaction policy should build");
let output = format!("{:?}", value.redacted_with(&policy));

assert!(!output.contains("raw-secret"));
assert!(output.contains("visible"));
```

字符串 map 会按每个 key 对对应 value 分类。同时启用 `redact` 和 `json` 后，
JSON 对象和数组会被递归遍历；敏感 key 对应非字符串值时，整个值都会被替换。
没有 key 上下文的标量仍保留普通 `Debug` 格式。普通 `Value` 格式化不会隐式
脱敏，因此诊断输出必须显式使用脱敏视图。

## 保留类型的 Wire V1

直接 Serde 使用 `ValueWireV1`：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"int32"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

V1 的兼容性承诺覆盖上述 JSON 对象结构。其他 Serde 格式可以使用，但其
格式相关表示不属于稳定契约。

V1 是封闭格式。现有 tag、shape 和 payload 表示不得改变；未来新增运行时类型
必须使用新的 wire 版本，而不能扩展 V1。

这个结构性保证不意味着所有 feature 集都支持所有具体类型。具体扩展类型的
tag 只有在接收方启用对应 feature 时才能反序列化：日期/时间使用 `chrono`，
大数使用 `big-integer` 或 `big-decimal`，URL 使用 `url`，JSON 使用 `json`。
交换这些 payload 的生产者与消费者应约定所需 feature；不支持的具体 tag 会被
拒绝。`unset` payload 仍可以保留声明的 `DataType`，但不要求构建启用能够存储
该类型具体值的 feature。

`Value` 只接受 scalar，`MultiValues` 只接受 collection，`ValueContainer`
接受两者。信封必须包含数字版本 `1`；未知字段、未知类型、错误 shape 和所有
0.10 之前的 payload 都会被拒绝。宽整数使用 canonical 十进制字符串；
`BigDecimal` 使用精确的 `{"coefficient":"...","scale":i64}` payload；
`Duration` 使用 secs/nanos；非有限浮点会被拒绝。`Json(null)` 与
`Unset(Json)` 不同。

`Value`、`MultiValues`、`ValueContainer` 可通过 `From` 转成
`ValueWireV1`；`ValueWireV1` 可转回 `ValueContainer`。

`ValueWireV1::decode_json_slice()` 和
`ValueWireV1::decode_json_slice_with_limits()` 只接受完整的顶层 V1 文档，并在
解析前执行字节数限制。当 value 嵌入更大的 JSON 文档时，应先用完整外层输入
长度调用 `ValueWireLimits::check_json_bytes()`，再执行该文档自己的 Serde
decoder。

## 自然 JSON

同时启用 `converter` 与 `json` 后，`to_json_value()` 生成不含运行时类型标签
的普通业务 JSON。如果接收方必须恢复精确的数据类型和形态，应使用 Wire V1。
