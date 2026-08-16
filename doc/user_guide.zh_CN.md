# qubit-value 用户手册

[English version](user_guide.md) · [README](../README.zh_CN.md) · [API 文档](https://docs.rs/qubit-value)

## 手册目标与读者

本手册面向使用 Rust 处理配置、元数据、协议字段或其他运行时值的开发者。当值的具体
类型只能在运行时确定，但调用方仍需要明确类型、受控转换和可预期错误时，可以使用
`qubit-value`。

本手册适用于 `qubit-value` 0.10，介绍的是值容器层本身，不会把它扩展成配置服务、schema
registry 或持久化数据库。需要基于 `Value` 实现的现成 key-value 容器时，请参阅文末的
[`rs-config`](https://github.com/qubit-ltd/rs-config) 和
[`rs-metadata`](https://github.com/qubit-ltd/rs-metadata)。

以下 Rust 代码片段省略了外层函数，并假定函数返回兼容的 `Result`，因此使用 `?` 传播错误，
而不是用 `expect` 隐藏错误。

## 问题与概念模型

一个运行时值有两个相互独立的属性：

1. 声明的 `DataType`，例如 `Int32`、`Duration` 或 `StringMap`；
2. 值的形态，即一个标量，或一个同类型集合。

`qubit-value` 将这两个属性都显式保留下来：

| 类型 | 保存内容 | 适用场景 |
| --- | --- | --- |
| `Value` | 一个带类型的标量，或 `Unset(DataType)` | 一个 key 对应一个值 |
| `MultiValues` | 一个同类型集合，或带类型的未设置集合 | 一个 key 接受多个值 |
| `ValueContainer` | `Scalar(Value)` 或 `Collection(MultiValues)` | 输入本身的形态很重要 |
| `NamedValue` | name 加 `Value` | 具名属性需要与值一起传递 |
| `NamedMultiValues` | name 加 `MultiValues` | 具名重复属性需要与集合一起传递 |

以下三种状态不应被合并：

- `Unset(DataType::String)` 表示类型已声明，但没有具体值；
- `MultiValues::String(vec![])` 表示一个已经存在但为空的具体集合；
- 启用 `json` feature 时，`Json(Null)` 表示一个具体的 JSON 值。

`ValueContainer` 也防止单元素集合变成标量：
`Collection(MultiValues::Int32(vec![42]))` 在所有 API 和 Wire 边界上都仍然是集合。

## 实际场景：读取一个运行时配置对象

假设服务在运行时收到 `host`、`port`、`timeout` 和 `debug` 这些标量配置属性，同时还收到重复
的 `tags` 输入。成功标准是：

- 文本形式的 port 可以转换为 `u16`，并在超出范围时返回错误；
- 未设置的 timeout 可以使用默认值，同时保留声明的类型；
- 即使只有一个 tag，tags 仍然保持集合形态；
- 稍后可以带着运行时类型和形态一起序列化。

核心流程如下：

```rust
use std::collections::HashMap;
use std::time::Duration;

use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value, ValueContainer};

let config = HashMap::from([
    ("host".to_owned(), Value::new("localhost".to_owned())),
    ("port".to_owned(), Value::new("8080".to_owned())),
    ("debug".to_owned(), Value::new(false)),
    (
        "timeout".to_owned(),
        Value::new_unset(DataType::Duration),
    ),
]);

let host: String = config["host"].get()?;
assert_eq!(host, "localhost");

let port: u16 = config["port"].to()?;
assert_eq!(port, 8080);

let debug: bool = config["debug"].get()?;
assert!(!debug);

let timeout: Duration = config["timeout"].get_or(Duration::from_secs(30))?;
assert_eq!(timeout, Duration::from_secs(30));

let tags = ValueContainer::Collection(MultiValues::new(["production"]));
assert!(tags.is_collection());
assert_eq!(tags.data_type(), DataType::String);
```

如果这个小型 map 需要升级为完整的通用配置对象，可以使用来自
[`rs-config`](https://github.com/qubit-ltd/rs-config) 的 `Config`。它基于 `Value` 构建，并提供
属性管理、类型化读取、多值读取、默认值、配置 section、转换策略、插值，以及可插拔的文件/环境
变量配置源等更全面的高级能力。

下一步可以通过 Wire V1 编码 `tags` 或更大的 `ValueContainer`。下面依次解释完整类型表、
feature 选择、错误以及序列化边界，并给出完整的往返示例。

## 安装与 feature 选择

核心依赖如下：

```toml
[dependencies]
qubit-value = { version = "0.10", features = ["all"] }
qubit-datatype = { version = "0.11", default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

默认 feature 集为空。根据应用需要添加类型族：

| Feature | 启用内容 |
| --- | --- |
| `converter` | `Value::to`、`MultiValues::to_list` 等转换 API |
| `chrono` | `Date`、`Time`、`DateTime` 和 `Instant` |
| `big-integer` | `BigInteger` |
| `big-decimal` | `BigDecimal` |
| `big-number` | `big-integer` 与 `big-decimal` 的兼容别名 |
| `url` | `Url` |
| `json` | `Json` 和 Wire JSON 解码/资源限制；自然 JSON 还需要 `converter` |
| `redact` | `Value` 脱敏视图；应用还需要从 `qubit-redact` 导入 `Redact` |
| `all` | `converter`、`chrono`、`big-number`、`url`、`json` 和 `redact` |

当具体值在不同构建之间交换时，生产方和消费方必须对所需 feature 达成一致。没有启用
`chrono` 的构建仍可以理解 `Unset(DataType::Date)` 这样的声明，但无法物化具体 `Date`
payload。

## 支持的 `DataType`

`DataType` 由 `qubit-datatype` 定义，是本 crate 使用的完整运行时类型词汇。当前共有 25 个
变体：

| `DataType` | Rust 表示 | Feature | 说明 |
| --- | --- | --- | --- |
| `Bool` | `bool` | — | 布尔值 |
| `Char` | `char` | — | Unicode 字符 |
| `Int8` | `i8` | — | 8 位有符号整数 |
| `Int16` | `i16` | — | 16 位有符号整数 |
| `Int32` | `i32` | — | 32 位有符号整数 |
| `Int64` | `i64` | — | 64 位有符号整数 |
| `Int128` | `i128` | — | 128 位有符号整数；Wire 使用十进制文本 |
| `UInt8` | `u8` | — | 8 位无符号整数 |
| `UInt16` | `u16` | — | 16 位无符号整数 |
| `UInt32` | `u32` | — | 32 位无符号整数 |
| `UInt64` | `u64` | — | 64 位无符号整数 |
| `UInt128` | `u128` | — | 128 位无符号整数；Wire 使用十进制文本 |
| `Float32` | `f32` | — | 32 位浮点数；Wire 要求有限值 |
| `Float64` | `f64` | — | 64 位浮点数；Wire 要求有限值 |
| `String` | `String` | — | UTF-8 文本 |
| `Date` | `chrono::NaiveDate` | `chrono` | 日历日期 |
| `Time` | `chrono::NaiveTime` | `chrono` | 一天中的时间 |
| `DateTime` | `chrono::NaiveDateTime` | `chrono` | 日期和本地时间 |
| `Instant` | `chrono::DateTime<chrono::Utc>` | `chrono` | UTC 时间点 |
| `BigInteger` | `num_bigint::BigInt` | `big-integer` | 任意精度整数 |
| `BigDecimal` | `bigdecimal::BigDecimal` | `big-decimal` | 精确小数，Wire scale 有上限 |
| `Duration` | `std::time::Duration` | — | Wire 使用秒/纳秒；文本转换由策略决定 |
| `Url` | `url::Url` | `url` | 已解析 URL |
| `StringMap` | `HashMap<String, String>` | — | 字符串到字符串的 map |
| `Json` | `serde_json::Value` | `json` | 任意 JSON 结构 |

Feature 列表示存储具体 Rust 值时 `qubit-value` 所需的 feature。`StringMap` 是原生 map 类型，
不需要 `json`；`Json` 变体才需要 `json`。

## 核心工作流

### 构造和检查单值

Rust 类型已经确定时使用类型化构造函数；只有声明了类型但尚无值时，使用
`Value::new_unset`。

```rust
use qubit_datatype::DataType;
use qubit_value::Value;

let mut value = Value::new(8080i32);
let port: i32 = value.get()?;
assert_eq!(port, 8080);
assert_eq!(value.data_type(), DataType::Int32);

value.unset();
assert!(value.is_unset());
assert_eq!(value.data_type(), DataType::Int32);

value.set_type(DataType::String);
value.set("8080");
assert_eq!(value.get_string()?, "8080");
```

`get<T>()` 是严格读取。存储的变体必须与 `T` 一致，不会猜测“应该把整数文本解析出来”。
`get_or` 只会在值未设置时使用默认值，类型不匹配仍然是错误。

### 构造和修改同类型集合

在对应转换实现存在时，`MultiValues::new`、`set` 和 `add` 支持 vector、数组、切片、借用
vector 以及借用字符串集合。

```rust
use qubit_value::MultiValues;

let mut ports = MultiValues::new([8080i32, 8081, 8082]);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.set([9000, 9001]);
assert_eq!(ports.len(), 2);

let first: i32 = ports.get_first()?;
assert_eq!(first, 9000);
```

`add` 必须检查元素类型是否一致，因此会失败；`set` 替换整个集合，也可以改变元素类型。
通过 `is_unset()` 和 `is_empty()` 可以区分未设置集合和具体空集合。

### 使用显式策略转换

启用 `converter` 后，`to` 使用 `qubit-datatype` 的共享转换契约。如果默认的严格策略不适合，
使用 `to_with` 指定策略。

```rust
use qubit_value::Value;

let text = Value::new("42".to_owned());
let number: u32 = text.to()?;
assert_eq!(number, 42);

let fallback: u16 = Value::new_unset(qubit_datatype::DataType::UInt16)
    .to_or(8080u16)?;
assert_eq!(fallback, 8080);
```

如果转换策略将源值判定为缺失，例如配置了“空白即缺失”，`to_or` 也可以使用默认值；但它
不会掩盖普通类型不匹配或非法转换。完整的源/目标矩阵和策略说明请参阅
[`qubit-datatype` API 文档](https://docs.rs/qubit-datatype/latest/qubit_datatype/)。

### 保留名称但不改变值语义

```rust
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value};

let mut timeout = NamedValue::new("timeout", Value::new(30u64));
assert_eq!(timeout.name(), "timeout");
assert_eq!(timeout.value().get()?, 30);
timeout.value_mut().set(45u64);

let mut ports = NamedMultiValues::new("ports", MultiValues::new([8080u16, 8081]));
ports.values_mut().add(8082u16)?;
let first_port: u16 = ports
    .values()
    .get_first()
    ?;
assert_eq!(first_port, 8080);
```

## Wire V1：通过 JSON 保留类型和形态

### 什么时候使用 Wire V1

运行时 `Value` 类型本身不直接实现 Serde。需要在 JSON 边界恢复精确的运行时类型和形态时，
选择版本化 Wire 适配器；只需要普通 JSON 时，使用自然 JSON（`to_json_value`）。

| 需求 | 选择 |
| --- | --- |
| 将 `Int32(42)` 恢复为 `Int32`，而不是无类型 JSON 数字 | Wire V1 |
| 区分标量 `42` 和集合 `[42]` | Wire V1 |
| 保留 `Unset(DataType::String)` | Wire V1 |
| 为 JSON API 生成普通 `null`、数字、字符串、对象和数组 | 自然 JSON |

### Envelope、shape 和 payload

`ValueWireV1` 是独立使用的 V1 envelope，包含数字版本 `version: 1` 和带类型的 `value`
shape。shape 只有两种：`scalar` 和 `collection`；payload key 使用 `DataType` 变体的小写
Wire 名称。

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"string"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

`ValueWirePayloadV1` 表示不带外层版本字段的同一 typed shape；只有在另一个协议已经拥有版本
envelope 时才使用它。`ValueWireRefV1` 和 `ValueWirePayloadRefV1` 可以在不克隆源值的情况下
序列化借用值。

V1 是封闭格式。现有 tag、shape 和 payload 表示不能原地扩展；未来新增运行时类型必须使用
新的 wire 版本。在支持的规范化 JSON 配置下，字符串 map key 和嵌套 JSON object key 会按
字典序输出。

### 端到端示例：从 `ValueContainer` 到 JSON 再恢复

下面的示例创建一个显式标量值，将它转换为拥有所有权的 Wire DTO，序列化为 JSON，在输入和
语义资源限制下解码，最后恢复原来的 container。

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonEncodeLimits, JsonResource, JsonValueLimits};
use qubit_budget::ResourceLimit;
use qubit_budget::StructureLimits;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;

let original = ValueContainer::Scalar(Value::new(8080i32));
let wire = ValueWireV1::try_from(original.clone())?;
let structure = StructureLimits::<StructureResource, usize>::new()
    .with_depth_limit(ResourceLimit::new(JsonResource::Depth, 32))
    .with_nodes_limit(ResourceLimit::new(JsonResource::Nodes, 128));
let values = JsonValueLimits::<JsonResource, usize>::new().with_structure_limits(structure);
let encode_limits = JsonEncodeLimits::<JsonResource, usize>::new()
    .with_output_bytes_limit(ResourceLimit::new(
        JsonResource::OutputBytes,
        64 * 1024,
    ))
    .with_value_limits(values);
let encoded = wire.to_json_vec_with_limits(encode_limits)?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"int32":8080}}}"#
);

let decode_limits = JsonDecodeLimits::<JsonResource, usize>::new()
    .with_input_bytes_limit(ResourceLimit::new(
        JsonResource::InputBytes,
        64 * 1024,
    ))
    .with_value_limits(values);
let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits)?;
let restored: ValueContainer = decoded.into();

assert_eq!(restored, original);
assert!(restored.is_scalar());
assert_eq!(restored.data_type(), qubit_datatype::DataType::Int32);
```

解码入口使用 `qubit-budget` 提供的通用 JSON/Serde adapter。
`ValueWireV1::default_json_decode_limits()` 与 `default_json_encode_limits()` 分别提供
V1 默认定向 profile；应用自行控制输入、输出或 value 预算时，传入对应的
`JsonDecodeLimits` 或 `JsonEncodeLimits`。

### 借用 Wire 编码

如果源值在序列化调用期间一直有效，使用借用适配器可以避免不必要的 clone。

```rust
use qubit_value::{Value, ValueWireRefV1};

let value = Value::new("service-a".to_owned());
let borrowed = ValueWireRefV1::from_value(&value)?;
let encoded = borrowed.to_json_vec()?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"string":"service-a"}}}"#
);
```

对于已经版本化的外层协议，可以使用 `ValueWirePayloadRefV1::from_value`、`from_values` 或
`from_container`，然后调用借用 payload 的 `to_json_vec()` 或 `to_json_writer()`。这些构造器会在返回可序列化 payload 前校验有限浮点、
有界的 `BigDecimal scale` 以及保留的 JSON object key，因此它们是可能失败的。

### 嵌入值与共享 `JsonDecodeSession`

`decode_json_slice_with_limits` 用于完整的顶层 Wire 文档。如果值嵌套在更大的 JSON 文档中，
应使用 `qubit-budget` 的 Serde adapter 处理完整外层文档，让同一个 session 计费所有 JSON 节点。

```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession, JsonResource};
use qubit_json::text::JsonTextDecoder;
use qubit_budget::ResourceLimit;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;
use serde::Deserialize;

#[derive(Deserialize)]
struct Request {
    value: ValueWireV1,
}

let input = br#"{"value":{"version":1,"value":{"collection":{"int32":[1,2]}}}}"#;
let limits = JsonDecodeLimits::<JsonResource, usize>::new().with_input_bytes_limit(
    ResourceLimit::new(JsonResource::InputBytes, 64 * 1024),
);
let mut session = JsonDecodeSession::new(limits);
let request: Request = JsonTextDecoder::new(&mut session).decode(input)?;
let restored: ValueContainer = request.value.into();
assert!(restored.is_collection());
```

外层 object 和嵌入的 V1 envelope 都属于同一个通用 JSON 文档预算。复用一个 `JsonDecodeSession` 可以
累计同一 request 中多个嵌入值的用量，并拒绝完整文档后的 trailing content。

### Wire 的类型和输入边界

- `Int128`、`UInt128` 和 `BigInteger` 使用 canonical 十进制字符串，避免 JSON number 丢失精度。
- `BigDecimal` 使用精确的 coefficient/scale payload，scale 的绝对值不能超过 V1 上限。
- `Duration` 使用 `{ "secs": ..., "nanos": ... }` payload，且 nanos 小于一秒。
- `Float32` 和 `Float64` 可以在内存中保存非有限值，但 V1 会拒绝 NaN 和无穷，因为 JSON 没有
  对应的数字字面量。
- `Json(null)` 是具体 JSON 值，与 `Unset(Json)` 不同。
- 具体扩展类型只有在接收方启用对应 feature 时才能解码；不支持的 payload 会被拒绝，不会
  被猜测成其他类型。
- 未知字段、未知类型、错误的 scalar/collection shape、不是数字 `1` 的版本，以及 0.10 之前
  的外部标签文档都会被拒绝。

## 自然 JSON

同时启用 `converter` 和 `json` 后，自然 JSON 将运行时值投影为普通的 `serde_json::Value`。下面的
示例展示几种常见值实际生成的 JSON 字符串：

```rust
use std::collections::HashMap;

use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value};

assert_eq!(Value::new(42i32).to_json_value()?.to_string(), "42");
assert_eq!(
    Value::new("localhost".to_owned())
        .to_json_value()?
        .to_string(),
    r#""localhost""#,
);
assert_eq!(
    Value::new_unset(DataType::String)
        .to_json_value()?
        .to_string(),
    "null",
);
assert_eq!(
    MultiValues::new([8080i32, 8081])
        .to_json_value()?
        .to_string(),
    "[8080,8081]",
);
assert_eq!(
    Value::new(HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ]))
    .to_json_value()?
    .to_string(),
    r#"{"a":"1","z":"26"}"#,
);
```

生成的字符串依次是 `42`、`"localhost"`、`null`、`[8080,8081]` 和
`{"a":"1","z":"26"}`。标量保持标量形态，未设置值变成 `null`，具体集合始终变成数组，
字符串 map 的 key 按字典序输出。

对于单个 map 值，也可以这样构造：

```rust
use qubit_value::Value;

let value = Value::new(std::collections::HashMap::from([
    ("host".to_owned(), "localhost".to_owned()),
]));
let json = value.to_json_value()?;
assert_eq!(json.to_string(), r#"{"host":"localhost"}"#);
```

自然 JSON 刻意丢弃运行时 `DataType` 标签。未设置值会投影为 `null`，每个具体集合都会投影
为数组，单元素集合也不例外。必须恢复 `DataType`、unset 状态或形态时，应改用 Wire V1。

## 错误与诊断

值操作返回 `ValueResult<T>`，它是 `Result<T, ValueError>` 的别名。主要错误类别如下：

| 错误 | 含义 |
| --- | --- |
| `ValueError::Missing` | 值未设置、集合为空，或转换后没有值 |
| `ValueError::TypeMismatch` | 严格 `get<T>()` 请求了不同类型 |
| `ValueError::Conversion` | 标量转换不支持或值非法 |
| `ValueError::ListConversion` | 集合转换失败，并保留源索引 |
| `ValueWireEncodeError` | 值违反 V1 编码规则，例如包含非有限浮点 |
| `ValueWireDecodeError` | JSON、版本、shape、feature 或资源限制校验失败 |

应把缺失值和非法值分开处理。对于有意缺失的配置属性可以使用默认值；对于错误的 port 或
类型不匹配，不应使用默认值掩盖问题。如果诊断输出包含字符串 map 或 JSON object，并且应用
有敏感字段，应使用显式的 `redact` 视图；普通 `Debug` 格式化不会自动脱敏。

## 排障

### `get<T>()` 返回类型不匹配

检查 `value.data_type()`，当源类型必须完全一致时使用类型化 getter。如果确实需要转换，启用
`converter`，调用 `to<T>()` 并选择符合应用要求的策略。

### 没有使用默认值

确认容器确实是 unset。具体的空 `MultiValues` 不是未设置集合。还要检查转换策略是否把源值
归类为缺失；普通非法转换不会触发 `to_or` 默认值。

### Wire 解码拒绝值

按下面顺序检查：

1. 输入是否是一个完整 JSON 文档，而不是片段；
2. `version` 是否为数字 `1`；
3. `scalar`/`collection` shape 是否与目标容器一致；
4. 接收方构建是否启用了具体类型所需的 feature；
5. 输入和解码后的结构是否符合传入的 `JsonDecodeLimits` profile；
6. 值是否包含非有限浮点或不符合边界的 payload。

### JSON 边界丢失了类型信息

这是自然 JSON 的预期行为。接收方必须恢复 `DataType`、unset 状态或形态时，将
`to_json_value()` 替换为 `ValueWireV1`。

## 限制与最佳实践

- 具体的 `chrono`、大数、URL 或 JSON payload 跨边界传输时，明确约定 Wire 生产方和消费方的
  feature 集。
- 输入契约包含标量/集合差异时，使用 `ValueContainer`，不要根据集合长度推断形态。
- 对不可信的完整 JSON 输入使用带资源限制的 Wire 解码入口。值嵌入外层文档时，按完整外层
  文档计入输入，并共享一个 budget。
- 不要把 `Eq`/`Hash` 输出当作持久化指纹。hash 行为用于进程内 Rust 集合，可能随 hasher、
  平台、版本、feature 或实现变化。
- 普通 `Debug` 输出不会自动脱敏。启用 `redact` 和策略 crate 时，应显式创建脱敏视图。
- 自然 JSON 是互操作投影，不是无损的运行时值格式。

## 基于 `Value` 构建的容器

`Value` 是可复用的值层，不是完整的 key-value 产品。两个兄弟 crate 直接建立在它之上：

- [`rs-config`](https://github.com/qubit-ltd/rs-config) 适用于应用配置属性，以及从配置文件、
  环境变量等来源进行面向配置的读取。
- [`rs-metadata`](https://github.com/qubit-ltd/rs-metadata) 适用于附着在资源或记录上的类型化
  元数据和属性值，以及过滤和查询场景。

除了类型化存储和转换原语之外，还需要 key 管理和领域操作时，应使用这些 crate。

## 延伸阅读

- [中文 README](../README.zh_CN.md)
- [English README](../README.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-value)
- [`qubit-datatype` 转换文档](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
