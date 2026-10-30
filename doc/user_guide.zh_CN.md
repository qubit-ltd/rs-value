# qubit-value 用户手册

[English version](user_guide.md) · [README](../README.zh_CN.md) · [设计文档](design.zh_CN.md) · [API 文档](https://docs.rs/qubit-value)

## 手册目标与读者

本手册面向使用 Rust 处理配置、元数据、协议字段或其他运行时值的开发者。当值的具体
类型只能在运行时确定，但调用方仍需要明确类型、受控转换和可预期错误时，可以使用
`qubit-value`。

本手册适用于 `qubit-value` 0.13，介绍的是值容器层本身，不会把它扩展成配置服务、schema
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

<!-- example:runtime-config run -->
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

要运行本手册中的全部示例，请添加以下依赖。实战场景本身只需要启用 `converter` 的
`qubit-value` 和 `qubit-datatype`；其余 crate 用于 Wire 与嵌入文档章节。

```toml
[dependencies]
qubit-value = { version = "0.13", features = ["all"] }
qubit-datatype = { version = "0.15", default-features = false, features = ["converter"] }
bigdecimal = "0.4"
qubit-budget = { version = "0.10", features = ["json"] }
qubit-json = "0.12"
qubit-redact = { version = "0.10", default-features = false }
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
| `json` | `Json` 和有界版本化 JSON Wire 编解码 |
| `natural-json` | `converter` + `json` 的便捷组合；两者同时启用时也提供自然 JSON 投影 |
| `redact` | `Value` 脱敏视图；应用还需要从 `qubit-redact` 导入 `Redact` |
| `all` | `converter`、`chrono`、`big-number`、`url`、`json`、`natural-json` 和 `redact` |

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

<!-- example:single-value run -->
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

<!-- example:multi-values run -->
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

#### 首项读取与源码兼容性

`get_first`、`get_first_or` 和 `get_first_or_else` 返回一个标量首项，要求
`T: StrictValueRead`。`get_first_ref` 借用一个标量首项，要求目标同时支持从 `Value` 和
`MultiValues` 转换。这些 bound 会在编译期拒绝集合目标。需要完整集合时请使用对应 API：

| 旧调用 | 改用 |
| --- | --- |
| `get_first::<Vec<i32>>()` | `get::<i32>()` |
| `get_first_ref::<[i32]>()` | `get_slice::<i32>()` |
| `get_first_or::<Vec<i32>>(default)` | `get_or::<i32>(default)` |
| `get_first_or_else::<Vec<i32>, _>(factory)` | `get_or_else::<i32, _>(factory)` |
| 仅以 `TryFrom<&MultiValues>` 约束的标量包装函数 | 将 bound 改为 `T: StrictValueRead` |

更严格的 bound 会影响曾使用集合目标或旧 bound 泛型包装函数的调用方，属于源码兼容性变化。
标量读取、unset 默认值以及 `get_first_or_else` 的惰性 fallback 行为保持不变。

### 使用显式策略转换

需要检查运行时类型的适配器可以使用 `Value::view()` 和 `MultiValues::view()`，分别获得
`ValueRef` 和 `MultiValuesRef`。字符串、map、JSON tree 和集合 slice 都保持借用。
集合视图提供 `data_type()`、`len()`、`is_empty()` 和 `get(index)`；按索引读取返回可选的
`ValueRef`，无需构造临时 owned `Value`。匹配这些非穷尽视图时要保留 wildcard 分支。
启用 `converter` 后，可以直接把 `ValueRef` 传给 `DataConverter`。

启用 `converter` 后，`to` 使用 `qubit-datatype` 的共享转换契约。如果默认的严格策略不适合，
使用 `to_with` 指定策略。

<!-- example:conversion run -->
```rust
use qubit_value::Value;

let text = Value::new("42".to_owned());
let number: u32 = text.to()?;
assert_eq!(number, 42);

let fallback: u16 = Value::new_unset(qubit_datatype::DataType::UInt16)
    .to_or(8080u16)?;
assert_eq!(fallback, 8080);
```

<a id="shared-conversion-session"></a>
### 让多次读取共享转换限制

每次调用 `_with` 都会创建独立会话。相关读取需要共用预算时，使用 `_in` 方法传入同一个
会话。成功消耗的 item 会在会话中累计；下面第三次转换会超过共享的两项限制：

<!-- example:shared-conversion-session run -->
```rust
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionResource;
use qubit_datatype::ConversionSession;
use qubit_value::Value;
use qubit_value::ValueError;

let policy = ConversionPolicy::strict();
let limits = ConversionLimits::builder()
    .operation_limits(ConversionOperationLimits::builder().max_items(2).build())
    .build();
let mut session = ConversionSession::new(&policy, &limits);
assert_eq!(Value::from("41").to_in::<u16>(&mut session)?, 41);
assert_eq!(Value::from("42").to_in::<u16>(&mut session)?, 42);
let error = Value::from("43").to_in::<u16>(&mut session).unwrap_err();
assert!(matches!(error, ValueError::Conversion(ref source)
    if source.resource() == Some(ConversionResource::Items)));
assert_eq!(session.items_used(), 2);
```

`Value::to_in` 会复用调用方的会话。`MultiValues` 和 `ValueContainer` 也提供对应的
`to_first_in`、`to_list_in` 入口。会话不是整批自动回滚的事务：成功消耗会累计；失败转换
可能保留此前已经成功计入的 item 或 input 消耗。具体失败计量遵循 `qubit-datatype` 合同，
调用方不应假设整个批次都会回滚。

如果转换策略将源值判定为缺失，例如配置了“空白即缺失”，`to_or` 也可以使用默认值；但它
不会掩盖普通类型不匹配或非法转换。完整的源/目标矩阵和策略说明请参阅
[`qubit-datatype` API 文档](https://docs.rs/qubit-datatype/latest/qubit_datatype/)。

默认值参数使用 `IntoValueDefault<T>`，例如可直接用字符串字面量作为 `String` 的默认值，
无需提前分配。适配仅在确实需要回退时执行。集合某项被策略判定为缺失时不会使用默认值，
读取首项时的第零项也不例外；从具体空集合读取首项同样不能回退。

### 保留名称但不改变值语义

<!-- example:named-values run -->
```rust
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value};

let mut timeout = NamedValue::new("timeout", Value::new(30u64));
assert_eq!(timeout.name(), "timeout");
let timeout_seconds: u64 = timeout.value().get()?;
assert_eq!(timeout_seconds, 30);
timeout.value_mut().set(45u64);

let mut ports = NamedMultiValues::new("ports", MultiValues::new([8080u16, 8081]));
ports.values_mut().add(8082u16)?;
let first_port: u16 = ports
    .values()
    .get_first()
    ?;
assert_eq!(first_port, 8080);
```

<a id="numeric-comparison"></a>
## 跨运行时类型比较数值

`rs-metadata` 的过滤条件可能要比较数值属性和查询值，即使二者使用不同的整数宽度。
`Value` identity 仍包含运行时变体：`Int32(1)` 与 `Int64(1)` 是不同的值，但在精确策略下，
`numeric_cmp` 会判定它们的数学值相等。`NumericComparisonPolicy` 应从 `qubit_datatype` 导入，
`qubit_value` 不会重新导出它。

过滤需要确定的数值顺序时使用 Exact。是否接受记录仍由应用决定；下面的例子接受相等值，并单独
检查 NaN 输入错误：

<!-- example:numeric-comparison run -->
```rust
use std::cmp::Ordering;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::{NumericComparisonError, Value};

let stored = Value::Int32(1);
let operand = Value::Int64(1);
assert_ne!(stored, operand);
let accepted = stored.numeric_cmp(&operand, NumericComparisonPolicy::Exact)? == Ordering::Equal;
assert!(accepted);
assert!(matches!(
    Value::Float64(f64::NAN).numeric_cmp(&operand, NumericComparisonPolicy::Exact),
    Err(NumericComparisonError::LeftNaN)
));
```

`numeric_cmp` 会针对左侧或右侧缺失值、左右侧非数值，以及左侧 NaN、右侧 NaN 或双方 NaN
返回 `NumericComparisonError`。检查顺序是先从左到右判断缺失值，再从左到右判断具体类型，最后分类
NaN。unset 不等于零。`Approximate` 不是通用容差或 epsilon 比较：有限浮点数可能使混合表示投影到
`f64`，不同配对的结果可能不满足传递性。因此不能用它实现 `Ord`、排序、分组或有序集合键。

<a id="wire-v1"></a>
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

V1 拒绝旧的裸枚举表示，例如 `{"Int32":42}`。独立文档必须使用上方的版本 envelope。嵌入式
payload `{"scalar":{"int32":42}}` 仅可与 payload seed/DTO 配合使用，不能作为独立 V1
文档；独立文档缺少 `version` 字段时无效。

### JSON 数字接受范围

Wire V1 只接受能由严格数字模型恢复为相同 `serde_json::Number` 的运行时 JSON 数字：负整数适合
`i64`，非负整数适合 `u64`，小数或指数值可转成有限 `f64` 且不会改变 Number equality。这会保护
`Value::Json` 使用的运行时 JSON identity。

其他依赖启用 `serde_json/arbitrary_precision` 时，`1.2500`、`1e1` 或不能由 `f64` 无损保留的小数，
会在构造或序列化 Wire 数据时被拒绝。错误不包含数字文本。V1 保留已接受的 JSON 值，不保留输入
数字 token 的拼写；decoder 已标准化的数字若满足相同规则，仍可接受。

需要精确宽整数或小数时，应使用显式 `BigInteger` 或 `BigDecimal` 运行时类型。嵌套 JSON 树中的任意数字
可在应用 schema 中约定为字符串。若允许舍入，应在构造 Wire 数据前主动转换为标准有限 `f64`；库不
会暗中舍入。已舍入的数据无法从 Wire 恢复，必须从原始数据源重新导入。

JSON 的 envelope、具名包装类型，以及 `Duration` 和 `BigDecimal` 富类型 payload 都要求使用带
字段名的对象。以前可能被接受的数组形式现在会被拒绝；请改用
`{"secs":1,"nanos":2}` 或 `{"coefficient":"123","scale":4}` 这样的对象。
集合 payload 和声明类型为 `Json` 的数组仍然有效。

各富类型的规范表示和边界见[Wire V1 富类型 payload 表](design.zh_CN.md#wire-v1-rich-payloads)。
下面通过完整往返展示 BigDecimal：JSON 对象是带版本 envelope 中的类型 payload；集合则在同一 tag
下使用 payload 数组。scale 会保留小数的原始表示（`123.4500`），这与 `BigDecimal`
identity/hash 对末尾零的归一化是两回事。

<!-- example:wire-rich-types run -->
```rust
use std::str::FromStr;
use bigdecimal::BigDecimal;
use qubit_value::{Value, ValueContainer, ValueWireV1};

let original = ValueContainer::from(Value::BigDecimal(BigDecimal::from_str("123.4500")?));
let wire = ValueWireV1::try_from(original.clone())?;
let bytes = wire.to_json_vec()?;
assert_eq!(
    serde_json::from_slice::<serde_json::Value>(&bytes)?,
    serde_json::json!({"version":1,"value":{"scalar":{"bigdecimal":{"coefficient":"1234500","scale":4}}}})
);
let decoded = ValueWireV1::decode_json_slice(&bytes)?;
let restored: ValueContainer = decoded.into();
assert_eq!(restored, original);
```

V1 是封闭格式。现有 tag、shape 和 payload 表示不能原地扩展；未来新增运行时类型必须使用
新的 wire 版本。在支持的规范化 JSON 配置下，字符串 map key 和嵌套 JSON object key 会按
字典序输出。

具体 `Instant` payload 的 UTC 年份必须在 0 到 9999 之间，以确保每个编码时间戳都能由 V1
RFC3339 decoder 读取。负年份和五位数年份现在会返回 `ValueWireEncodeError::InstantYearOutOfRange`；
这是对过去可编码、却无法解码的值收紧接受范围。此类值应保留在内存中，或在跨越 V1 边界前
转换为应用显式版本化的表示；普通四位数年份无需迁移。

`ValueWireEncodePreflight` 使用保守下界。它现在将 BigDecimal 系数计入字符串 payload，但估算值
可能小于精确十进制文本长度。JSON object 会先做自身准入，再创建排序索引，因此因自身结构限额
被拒绝的 object 不会产生该分配。Preflight 与最终 `JsonEncodeSession` 记账减少了部分可避免工作，
但不会限制所有编码准备分配。

### 端到端示例：从 `ValueContainer` 到 JSON 再恢复

下面的示例创建一个显式标量值，将它转换为拥有所有权的 Wire DTO，序列化为 JSON，在输入和
语义资源限制下解码，最后恢复原来的 container。

<!-- example:wire-round-trip run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonEncodeLimits};
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;

let original = ValueContainer::Scalar(Value::new(8080i32));
let wire = ValueWireV1::try_from(original.clone())?;
let encode_limits = JsonEncodeLimits::builder()
    .max_output_bytes(64 * 1024)
    .max_depth(32)
    .max_nodes(128)
    .build();
let encoded = wire.to_json_vec_with_limits(encode_limits)?;

assert_eq!(
    encoded,
    br#"{"version":1,"value":{"scalar":{"int32":8080}}}"#
);

let decode_limits = JsonDecodeLimits::builder()
    .max_input_bytes(64 * 1024)
    .max_depth(32)
    .max_nodes(128)
    .build();
let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, decode_limits)?;
let restored: ValueContainer = decoded.into();

assert_eq!(restored, original);
assert!(restored.is_scalar());
assert_eq!(restored.data_type(), qubit_datatype::DataType::Int32);
```

解码入口使用 `qubit-budget` 的 JSON adapter。Wire DTO 刻意只实现 `Serialize`，不实现通用的
`Deserialize`，因为普通 Serde deserializer 无法同时约束不可信输入的原始大小和结构资源。
`NamedValue` 和 `NamedMultiValues` 是用于嵌入外层文档的包装类型，因此实现了通用
`Deserialize`；该实现校验 V1 schema 及 scalar/collection shape，但资源记账继承调用方提供的
deserializer。处理完整且不可信的 JSON 文档时，应使用它们有界的 decode helper；作为嵌套值时，
则应通过有资源限制的外层 deserializer 解码。
`ValueWireV1::default_json_decode_limits()` 与 `default_json_encode_limits()` 分别提供
V1 默认定向 profile；应用自行控制输入、输出或 value 预算时，传入对应的
`JsonDecodeLimits` 或 `JsonEncodeLimits`。

如果其他依赖统一启用了 `serde_json/arbitrary_precision`，应使用这些有界 helper 或
`JsonDecoder::decode_unique_*` 路径。通用 Serde reader 可能把 JSON number 呈现为 serde_json 的私有
map 编码。唯一键 decoder 会构造中间树，将对象键按普通用户键处理，并执行配置的 `max_depth`；它
没有额外固定的 127 层容器上限。中间树会增加内存开销，目标类型反序列化错误没有原始源码坐标。
配置深度允许时，130 层值可以解码；直接调用 `serde_json::from_slice` 则可能触及 Serde 默认递归限制。

解码器允许的嵌套深度不代表值的全部后续操作都无需递归。JSON 载荷的相等与哈希采用迭代遍历，但克隆、普通
`Debug` 格式化和析构仍沿用底层递归路径。处理不可信输入时应保持嵌套深度受限；提高解码深度前，也应考虑这些
生命周期操作的线程栈开销。直接在内存中构造的 JSON 不经过解码器深度限制。借用读取可以避免不必要的克隆，
但不会消除源值最终析构的递归路径。本库不提供跨平台通用的安全深度数值。

### 借用 Wire 编码

如果源值在序列化调用期间一直有效，使用借用适配器可以避免不必要的 clone。

<!-- example:borrowed-wire run -->
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
`from_container`，然后调用借用 payload 的 `to_json_vec()` 或 `to_json_writer()`。这些构造器会在返回可序列化 payload 前校验有限浮点和
有界的 `BigDecimal scale`，因此它们是可能失败的。JSON object key 不再有 serde_json Number
marker 保留规则。

四类 Wire DTO 和两种具名包装类型的显式限额 `to_json_vec_with_limits` 与
`to_json_writer_with_limits` helper 也会自动执行保守 payload preflight。最终 encoder 仍权威检查完整
payload 或 envelope。下界 preflight 错误可以提前拒绝；通过 preflight 不保证编码成功，也不限制全部准备阶段分配。

### 嵌入值与共享 `JsonDecodeSession`

`decode_json_slice_with_limits` 用于完整的顶层 Wire 文档。如果值嵌套在更大的 JSON 文档中，
应使用 `qubit-budget` 的 Serde adapter 处理完整外层文档，让同一个 session 计费所有 JSON 节点。

<!-- example:shared-decode-session run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
use qubit_json::decode::JsonDecoder;
use qubit_value::{ValueContainer, ValueWireV1Seed};

let input = br#"{"version":1,"value":{"collection":{"int32":[1,2]}}}"#;
let limits = JsonDecodeLimits::builder()
    .max_input_bytes(64usize * 1024)
    .build();
let session = JsonDecodeSession::from_limits(limits);
let mut decoder = JsonDecoder::new(session);
let decoded = decoder.decode_unique_seed_utf8(ValueWireV1Seed::new(), input)?;
let restored: ValueContainer = decoded.into();
assert!(restored.is_collection());
```

如果值嵌在更大的外层 object 中，应在同一个 `MapAccess` 上调用
`next_value_seed`，传入 `ValueWireV1Seed`；如果外层协议已经拥有版本，则传入
`ValueWirePayloadV1Seed`。这样外层 object 和 V1 envelope 共享同一个 JSON 文档预算。继续使用
同一个 `JsonDecoder` 解码后续完整文档时，session 会累计记账，每次调用也都会拒绝 trailing content。

### 嵌入 payload 预算：共享一个外层 session

外层 struct 可以在字段的 `deserialize_with` 函数中调用
`ValueWirePayloadV1Seed`。这样 derive 生成的 visitor 会通过同一个 decoder session 为两个字段
累计计费。每个 32 字节字符串单独都能通过，但两个 payload 合计会超过 64 字节的 payload 预算。

<!-- example:embedded-payload-budget run -->
```rust
use qubit_budget::json::{JsonDecodeLimits, JsonDecodeSession};
use qubit_json::decode::JsonDecoder;
use qubit_value::{ValueWirePayloadV1, ValueWirePayloadV1Seed};
use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeSeed;

#[derive(Debug, Deserialize)]
struct Outer {
    #[serde(deserialize_with = "decode_payload")]
    first: ValueWirePayloadV1,
    #[serde(deserialize_with = "decode_payload")]
    second: ValueWirePayloadV1,
}

fn decode_payload<'de, D>(deserializer: D) -> Result<ValueWirePayloadV1, D::Error>
where
    D: Deserializer<'de>,
{
    ValueWirePayloadV1Seed::new().deserialize(deserializer)
}

let input = br#"{"first":{"scalar":{"string":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"second":{"scalar":{"string":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}"#;
let tight = JsonDecodeLimits::builder()
    .max_string_bytes(64usize)
    .max_payload_bytes(64usize)
    .build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(tight));
assert!(decoder.decode_unique_utf8::<Outer>(input).is_err());

let wide = JsonDecodeLimits::builder()
    .max_string_bytes(64usize)
    .max_payload_bytes(128usize)
    .build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(wide));
let outer = decoder.decode_unique_utf8::<Outer>(input)?;
assert_eq!(outer.first.container().data_type().as_str(), "string");
assert_eq!(outer.second.container().data_type().as_str(), "string");

let numeric_input = br#"{"version":1,"value":{"scalar":{"uint64":18446744073709551615}}}"#;
let limits = JsonDecodeLimits::builder().max_input_bytes(4096usize).build();
let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits));
let wire = decoder.decode_unique_seed_utf8(
    qubit_value::ValueWireV1Seed::new(),
    numeric_input,
)?;
let restored: qubit_value::ValueContainer = wire.into();
assert_eq!(restored.data_type(), qubit_datatype::DataType::UInt64);
```

### Wire 的类型和输入边界

- `Int128`、`UInt128` 和 `BigInteger` 使用 canonical 十进制字符串，避免 JSON number 丢失精度。
- `BigDecimal` 使用精确的 coefficient/scale payload，scale 的绝对值不能超过 V1 上限。
- `Duration` 使用 `{ "secs": ..., "nanos": ... }` payload，且 nanos 小于一秒。
- 具体 `Instant` payload 的 UTC 年份必须在 0 到 9999 之间。负年份和五位数年份现在返回
  `InstantYearOutOfRange`。这是对过去可编码、却无法解码的值收紧兼容范围；此类值应保留在内存中，
  或使用应用显式版本化的格式，普通四位数年份不变。
- `Float32` 和 `Float64` 可以在内存中保存非有限值，但 V1 会拒绝 NaN 和无穷，因为 JSON 没有
  对应的数字字面量。
- `Json(null)` 是具体 JSON 值，与 `Unset(Json)` 不同。
- 通用 `Json` 值使用 `qubit-json` 数字契约：负整数装入 `i64`，非负整数装入 `u64`，小数为
  有限 `f64`；serde_json 旧私有 Number marker key 是普通 key。
- 前端收到大于 `2^53 - 1` 的整数时，需要保持精度的 parser，并映射为 `BigInt` 或字符串。
  JavaScript 的 `n` 后缀不是合法 JSON。
- 具体扩展类型只有在接收方启用对应 feature 时才能解码；不支持的 payload 会被拒绝，不会
  被猜测成其他类型。
- 未知字段、未知类型、错误的 scalar/collection shape、不是数字 `1` 的版本、旧的裸枚举
  表示，以及缺少版本 envelope 的独立文档都会被拒绝。

## 自然 JSON

`natural-json` 是 `converter` + `json` 的便捷组合。为兼容现有调用者，显式启用这两个
feature 同样提供投影接口。

`to_json_value_with` 在构造最终 JSON 树之前检查整次操作。集合中所有元素共享 item、
输入字节、输出载荷字节、结构节点和结构载荷限额；Duration 格式化使用一个共享转换会话。
结构深度包含根节点，外层集合数组也占一个节点。输入字节计算借用字符串、map 键和值、
JSON 字符串与键；输出载荷计算 JSON 字符串、键和数字文本，不计算标点、转义、布尔值或
null 的编码字节。大数在格式化前检查系数和 scale 限额。有界测量与预检查会增加一次遍历，
以确保最终树的分配发生在所有限额检查之后。源字符串一直借用到最终 JSON string 分配；
富类型的格式化结果缓存复用，Duration 也只通过共享会话格式化一次。整数和 Float64 的测量不需要临时堆字符串。
Float32 为保持现有十进制投影语义，目前会先生成临时 String。

投影时也会检查直接在内存中构造的 `serde_json::Value` 数字范围：整数 token 必须落在 `i64` 或
`u64` 范围内，小数或指数 token 必须能表示为有限 `f64`。越界整数和非有限数字会返回转换错误。
任意精度数值应使用字符串表示；若需保留运行时类型，则使用对应的 Wire 大数 payload。

投影预算错误使用 `ValueError::JsonProjectionLimit`，保留资源、实际预算事实和可选集合
索引；普通转换错误仍保留原有类型。每次调用使用新预算。需要限制最终编码字节时，请使用
Wire 的 `JsonEncodeLimits` 或 rs-json 的有界编码器；`ConversionLimits` 不是 JSON 编码
长度上限。

### 显示值之前先脱敏具名秘密

如果要在日志或诊断信息中显示一个值，应先明确哪些具名字段属于敏感信息，再用
`qubit-redact` 生成展示文本。下面将 `api_key` 标为秘密，保留普通 label，并展示输出预算不足时
如何明确返回替代标记：

<!-- example:redacted-output run -->
```rust
use std::collections::HashMap;

use qubit_redact::RedactionPolicy;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_value::Value;

let value = Value::new(HashMap::from([
    ("api_key".to_owned(), "raw-secret".to_owned()),
    ("label".to_owned(), "visible".to_owned()),
]));
let policy = RedactionPolicy::builder()
    .fields(|fields| { fields.raise("api_key", Sensitivity::Secret); })?
    .build()?;
let output = Redactor::new(policy).redact_text(&value);
let text = output.text_or_marker("<redaction incomplete>");
assert!(!text.contains("raw-secret"));
assert!(text.contains("visible"));

let bounded = RedactionPolicy::builder()
    .limits(|limits| { limits.max_output_bytes(0); })?
    .build()?;
let output = Redactor::new(bounded).redact_text(&value);
assert_eq!(output.text_or_marker("<redaction incomplete>"), "<redaction incomplete>");
```

普通的 `Value` `Debug` 输出不会自动脱敏。裸标量没有业务字段名，字段规则无法据此匹配；脱敏
也不能仅凭内容识别所有秘密。应用应在掌握字段含义的位置明确标注敏感数据。

### 有界物化 Serde 数据

`Value::from_serializable` 会校验并物化严格 JSON 值，但不使用有限资源预算。
`JsonEncodeLimits` 和 `ConversionLimits` 不会限制它生成的中间 JSON tree。需要有界物化序列化
输入时，可使用调用方持有的 `JsonValueBudget` transaction 进行编码；只有编码成功后才提交，再将
生成的 JSON 值包装为 `Value`：

<!-- example:bounded-serializable run -->
```rust
use qubit_budget::json::JsonResource;
use qubit_budget::json::JsonValueBudget;
use qubit_budget::json::JsonValueLimits;
use qubit_json::value::JsonValueEncoder;
use qubit_value::Value;

let limits = JsonValueLimits::<JsonResource, usize>::builder()
    .max_depth(8)
    .max_nodes(64)
    .max_string_bytes(1024)
    .max_payload_bytes(4096)
    .build();
let mut budget = JsonValueBudget::new(limits);
let mut transaction = budget.transaction();
let owner = JsonValueEncoder::new().encode_in_transaction(&vec!["bounded"], &mut transaction)?;
transaction.commit()?;
let value = Value::from_json_value(owner.into_value());
assert_eq!(value.get_json()?.as_array().unwrap().len(), 1);
```

如果编码失败，应传播错误并丢弃 transaction，不要提交；部分预算状态不会被保留。

启用 `natural-json` 后，自然 JSON 将运行时值投影为普通的 `serde_json::Value`。下面的
示例展示几种常见值实际生成的 JSON 字符串：

<!-- example:natural-json run -->
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

<!-- example:natural-json-map run -->
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

<a id="errors-and-diagnostics"></a>
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

`ValueWireDecodeError::InvalidJson` 携带 `ValueWireJsonError`，只暴露 Serde 类别和行列元数据，
不会保留原始错误消息或输入文本。通过唯一键中间树解码时发生的 target 类型错误没有原始源码位置，
因此 line 和 column 为零。

```rust
use qubit_value::ValueWireDecodeError;
use serde_json::error::Category;

let source = serde_json::from_str::<u64>("\"not a number\"").unwrap_err();
let error = ValueWireDecodeError::from(source);
if let ValueWireDecodeError::InvalidJson(json) = error {
    assert_eq!(json.classify(), Category::Data);
    assert!(json.line() > 0);
    assert!(json.column() > 0);
}
```

`InvalidJson` 的载荷类型已从 `serde_json::Error` 改为 `ValueWireJsonError`。下游代码若显式写出
旧载荷类型、直接构造该变体或向下转型其错误源，应改为匹配安全类型，并通过
`classify()`、`line()` 和 `column()` 读取元数据。普通类型化集合现在会将每个可见元素计入脱敏 item
budget；以前能完整输出的数字集合，在较小预算下可能变为截断视图。

## 排障

### `get<T>()` 返回类型不匹配

检查 `value.data_type()`，当源类型必须完全一致时使用类型化 getter。如果确实需要转换，启用
`converter`，调用 `to<T>()` 并选择符合应用要求的策略。

<a id="default-values"></a>
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

两者的读取契约不同：`Config::get` 按 reader 策略转换，`Metadata::get` 严格读取，
`Metadata::convert` 显式请求转换；默认值都复用 `IntoValueDefault`。

## 延伸阅读

- [中文 README](../README.zh_CN.md)
- [English README](../README.md)
- [English user guide](user_guide.md)
- [API 文档](https://docs.rs/qubit-value)
- [`qubit-datatype` 转换文档](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
