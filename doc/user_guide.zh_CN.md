# qubit-value 用户指南

## 依赖

```toml
qubit-value = { version = "0.10", features = ["all"] }
```

默认 feature 集为空。不需要全部类型族时，只启用 `chrono`、`big-integer`、
`big-decimal`、`url`、`json` 或 `converter`。`big-number` 继续作为两个大数
类型族的兼容别名。

## 运行时形态

`Value` 保存一个带类型标量，`MultiValues` 保存一个同类型集合。
`ValueContainer` 显式保留 `Scalar` 或 `Collection`；单元素集合不会变成标量。
`Unset(DataType)` 与具体值、具体空集合均不同。

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

`Value` 只接受 scalar，`MultiValues` 只接受 collection，`ValueContainer`
接受两者。信封必须包含数字版本 `1`；未知字段、未知类型、错误 shape 和所有
0.10 之前的 payload 都会被拒绝。宽整数和大数使用 canonical 十进制字符串；
`Duration` 使用 secs/nanos；非有限浮点会被拒绝。`Json(null)` 与
`Unset(Json)` 不同。

`Value`、`MultiValues`、`ValueContainer` 可通过 `From` 转成
`ValueWireV1`；`ValueWireV1` 可转回 `ValueContainer`。

## 自然 JSON

同时启用 `converter` 与 `json` 后，`to_json_value()` 生成不含运行时类型标签
的普通业务 JSON。如果接收方必须恢复精确的数据类型和形态，应使用 Wire V1。
