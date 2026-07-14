# `rs-value` `v0.8.0` 契约收敛设计

## 文档信息

- 文档名称：`rs-value v0.8.0 契约收敛设计`
- 文档版本：`v1.0`
- 创建日期：`2026-07-14`
- 目标版本：`0.8.0`
- 关联评审：`doc/review_2026-07-11.zh_CN.md`

## 1. 背景

`rs-value` 已经形成两个明确的生产消费边界：`rs-metadata` 使用
`Value` 承载元数据值，`rs-config` 使用 `MultiValues` 承载配置属性。
当前主要问题不再是类型覆盖不足，而是以下契约尚未收敛：

1. 非有限浮点通过 `serde_json` 序列化为 `null`，无法往返。
2. `Empty(DataType)` 与具体类型的空集合没有明确区分。
3. 下游为数值分类和自然 JSON 投影重复穷举公共枚举。
4. typed setter/adder 与泛型 API 重复，公共 API 和维护代码过多。
5. `Value` 与 `MultiValues` 的类型映射散落在多个文件中。
6. rich type 依赖无法按 feature 裁剪。
7. 默认转换入口重复构造 `DataConversionOptions`。

本次允许破坏性变更，以 `0.8.0` 为边界一次性收敛这些契约。

## 2. 目标

1. 使 JSON 序列化严格遵守 RFC 8259，并杜绝非有限浮点的静默丢失。
2. 将 unset 与 set-but-empty 建模为两种明确且可观察的状态。
3. 为常用数值分类和自然 JSON 投影提供公共操作。
4. 删除所有被泛型 API 完全覆盖的 typed setter/adder。
5. 使用一个私有类型表维护 `Value` 与 `MultiValues` 的共同映射。
6. 对齐 `rs-datatype 0.5.0` 的 feature 和结构化错误体系。
7. 保留 `NamedValue` 与 `NamedMultiValues`。
8. 更新 `rs-config`、`rs-metadata`，验证真实下游迁移。

## 3. 非目标

1. 本次不引入 `ValueWireV1` 等新的 wire adapter；现有 externally-tagged
   Serde schema 继续使用，并通过 golden tests 冻结。
2. 本次不把 `DataType` 改为开放枚举，也不添加 `#[non_exhaustive]`。
3. 本次不引入任意业务对象或递归动态对象模型。
4. 本次不删除或 feature-gate `NamedValue`、`NamedMultiValues`。
5. 单元素 `MultiValues::String` 被视为标量文本来源的启发式规则不在本次
   范围内；该问题需要单独设计来源形态后再修改。
6. 本次不把 `rs-metadata` 的完整数值比较策略迁入 `rs-value`；只提供通用
   分类能力，专用比较策略仍由消费方负责。

## 4. 依赖与 feature 设计

### 4.1 rs-datatype 依赖

`rs-datatype 0.5` 已发布，`rs-value` 直接使用 crates.io 依赖：

```toml
qubit-datatype = {
    version = "0.5",
    default-features = false,
}
```

不再使用临时相对路径，确保本地验证和最终发布解析同一依赖来源。

### 4.2 rs-value features

```toml
[features]
default = []
chrono = ["dep:chrono", "qubit-datatype/chrono"]
big-number = [
    "dep:bigdecimal",
    "dep:num-bigint",
    "qubit-datatype/big-number",
]
url = ["dep:url", "qubit-datatype/url"]
json = ["dep:serde_json", "qubit-datatype/json"]
converter = [
    "chrono",
    "big-number",
    "url",
    "json",
    "qubit-datatype/converter",
]
all = ["converter"]
```

约束如下：

1. 默认 feature 为空，与 `rs-datatype` 对齐。
2. `all` 是完整功能的稳定便捷入口。
3. `converter` 沿用 `rs-datatype` 当前语义，启用全部 rich types。
4. `serde` 和 `thiserror` 属于容器基础契约，保持必选依赖。
5. `DataType` 的 27 个描述符始终存在；关闭某个 rich feature 只会移除
   对应具体 Rust 存储 variant 和 API。
6. `Empty(DataType)` 仍可表达一个当前构建无法填入具体 rich value 的
   声明类型；这是 feature 裁剪下可接受的 typed-unset 状态。
7. `converter` 控制 `Value::to*`、`MultiValues::to*`、自然 JSON 投影、
   `DataConversion*` 错误分支，以及依赖结构化 conversion error 的 JSON
   helper；这些 API 不在仅启用 `json` 的构建中出现。
8. 单独启用 `json` 只提供 `Json` 存储 variant、`from_json_value()` 和对应
   Serde 支持。`from_serializable()`、`deserialize_json()` 与
   `to_json_value()` 由 `converter` 控制，避免在 `qubit-datatype/converter`
   未启用时引用不存在的错误类型。

`rs-config` 和 `rs-metadata` 显式启用 `qubit-value/all`，避免依赖默认
feature 的隐式行为。

## 5. 状态模型

### 5.1 定义

- **unset**：容器是 `Value::Empty(DataType)` 或
  `MultiValues::Empty(DataType)`，只有声明类型，没有具体值。
- **set-but-empty**：容器具有具体 variant，但内部值自身为空，例如
  `Value::String(String::new())` 或 `MultiValues::Int32(Vec::new())`。
- **set-and-nonempty**：容器具有具体 variant，且内部值非空。

### 5.2 公共 API

删除 `Value::is_empty()` 和 `MultiValues::is_empty()`，新增：

```rust
pub fn is_unset(&self) -> bool;
pub fn unset(&mut self);
```

状态示例：

| 值 | `is_unset()` | 内容自身是否为空 |
|---|---:|---:|
| `Value::Empty(DataType::Int32)` | `true` | 不适用 |
| `Value::String("")` | `false` | `get_string_ref()?.is_empty()` |
| `MultiValues::Empty(DataType::Int32)` | `true` | 不适用 |
| `MultiValues::Int32(vec![])` | `false` | `get_int32s()?.is_empty()` |
| `MultiValues::String(vec![""])` | `false` | 列表非空，第一个字符串为空 |

`count()` 只表示元素数量，因此 unset 和 set-but-empty 都可以返回 `0`；
调用方不得再用 `count() == 0` 判断 unset。

### 5.3 访问与默认值语义

| 操作 | unset | set-but-empty |
|---|---|---|
| `Value::get<T>()` | `ValueError::NoValue` | 返回具体值，包括空字符串/空 map/空 JSON 容器 |
| `MultiValues::get<T>()` | `ValueError::NoValue` | `Ok(Vec::new())` |
| typed list getter | `ValueError::NoValue` | `Ok(&[])` |
| `MultiValues::get_first<T>()` | `ValueError::NoValue` | `ValueError::NoValue` |
| `Value::to<T>()` | `DataConversionError::Missing` | 按具体值转换 |
| `MultiValues::to<T>()` | `DataConversionError::Missing` | `DataConversionError::EmptyCollection` |
| `MultiValues::to_list<T>()` | `DataConversionError::Missing` | `Ok(Vec::new())` |
| `*_or` / `*_list_or` | 返回默认值 | 不使用默认值，保留空结果或错误 |

这意味着 typed unset 不再伪装成空 slice/空 Vec。调用方需要默认值时使用
`*_or`；需要判断状态时使用 `is_unset()`。

### 5.4 clear 与 unset

1. `MultiValues::clear()` 清空内部 Vec，但保留具体 variant，因此结果是
   set-but-empty，并尽量保留 Vec capacity。
2. `MultiValues::unset()` 转为 `Empty(self.data_type())`。
3. `Value::clear()` 的既有行为等同于 unset；保留该方法以减少无关迁移，
   文档明确其语义，并新增同义但更明确的 `Value::unset()`。
4. `set_type()` 切换类型时产生 typed unset。

## 6. 数值分类公共操作

新增：

```rust
pub fn is_numeric(&self) -> bool;
```

`Value` 和 `MultiValues` 均遵守以下规则：

1. unset 返回 `false`，即使声明的 `DataType` 属于 numeric family。
2. 具体 numeric variant 返回 `true`；空 numeric Vec 仍是具体 numeric
   collection，因此返回 `true`。
3. 调用方如果只关心声明类型，使用
   `value.data_type().is_numeric()`。
4. `is_integer()`、`is_float()`、`is_big_number()` 等细分类不在
   `rs-value` 重复定义，统一使用新版 `DataType` 方法。

`rs-metadata` 的 `is_numeric_value()` 和 big-number 分类穷举应迁移到这些
公共操作；具体数值提取和比较仍保留在 `rs-metadata`。

## 7. JSON 契约

### 7.1 非有限浮点

RFC 8259 的 number grammar 不允许 `NaN`、`+Infinity`、`-Infinity`。
本 crate 采用严格、无损的默认策略：

1. `Value`/`MultiValues` 的 Serde 序列化遇到非有限浮点时返回错误。
2. 禁止把非有限浮点写成裸 token、`null` 或字符串。
3. JSON 反序列化拒绝裸非有限 token；`null` 不映射为浮点。
4. 自然 JSON 投影同样返回结构化错误：
   `InvalidValueReason::NonFinite`。
5. 多值投影使用 `DataListConversionError` 保留原始
   `source_index`。

Serde 的 `Serializer` 没有可移植的格式识别能力，因此对
`Serialize`/`Deserialize` 的非有限值校验会作用于所有 Serde 格式，而不只
是 `serde_json`。这是有意的协议收敛：默认 Serde wire 不承载非有限浮点。
未来若确有二进制格式保留 IEEE 特殊值的需求，应通过显式 wrapper/opt-in
协议提供，不能改变默认 JSON 契约。

内存中的 `Value::Float32/Float64` 仍允许由 Rust API 构造非有限值；限制发生
在 Serde 和自然 JSON 边界。

`from_serializable()` 也是 JSON 边界，不能直接沿用
`serde_json::to_value()` 对非有限浮点转为 `null` 的行为。实现必须使用
crate 私有的严格 JSON value serializer，在一次遍历中检查任意嵌套位置的
`f32/f64`：非有限值映射为 `InvalidValueReason::NonFinite`，其他 Serde 失败
仍映射为 JSON `Serialization` 错误。不能通过“先转成 `serde_json::Value` 再
扫描”实现，因为转换后的 `null` 已无法与源数据中的合法 `null` 区分。

### 7.2 tagged wire 与自然 JSON

两种 JSON 目的必须明确区分：

```rust
serde_json::to_value(&Value::Int32(42))?; // {"Int32": 42}
Value::Int32(42).to_json_value()?;        // 42
```

- Serde tagged wire 用于 `rs-metadata` 等需要保留 `DataType` 的协议。
- 自然 JSON 用于 `rs-config` 结构化反序列化等数据投影。

tagged wire 中 `Int128`/`UInt128` 的 payload 使用十进制字符串，而不是 JSON
number。这样既保留完整 128 位值域，也能通过 `serde_json` 和 Serde 的内部缓冲枚举
表示。反序列化只接受该十进制字符串形式；这是 0.8.0 的明确 wire 契约。

新增：

```rust
#[cfg(feature = "converter")]
pub fn to_json_value(&self) -> ValueResult<serde_json::Value>;
```

`Value` 的自然 JSON 映射：

| Value family | JSON 表示 |
|---|---|
| `Empty` | `null` |
| `Bool` | boolean |
| `Int8..Int64`, `UInt8..UInt64`, `IntSize`, `UIntSize` | number |
| finite `Float32`, `Float64` | number |
| non-finite float | error |
| `Int128`, `UInt128`, `BigInteger`, `BigDecimal` | string |
| `Char`, `String` | string |
| date/time/instant | string，沿用现有 config 格式 |
| `Duration` | rounded milliseconds with `ms`，沿用现有 config 格式 |
| `Url` | string |
| `StringMap` | object with string values |
| `Json` | identity |

`MultiValues` 先逐项使用同一标量映射，再按 cardinality 组合：

| 状态 | JSON 表示 |
|---|---|
| unset | `null` |
| set-but-empty | `[]` |
| 一个元素 | scalar/object |
| 多个元素 | array |

这里的 cardinality 规则仅定义自然 JSON shape，不参与字符串 collection
delimiter 的转换启发式。

### 7.3 下游迁移

`rs-config::property_to_json_value()` 改为调用
`MultiValues::to_json_value()` 并返回 `ConfigResult<serde_json::Value>`。
原来把非有限浮点替换为 `null` 的分支删除，错误通过结构化
`ValueError`/`ConfigError` 传播。

## 8. 单一类型表

### 8.1 权威边界

1. `rs-datatype::DataType` 是 27 个协议类型的唯一权威。
2. `rs-value` 建立一个私有表，作为 `DataType` 到 Rust 存储表示的唯一
   映射。
3. 类型表不是公共宏，不允许下游依赖其展开细节。

### 8.2 每行元数据

每个类型表条目包含：

1. feature gate；
2. enum variant；
3. Rust 存储类型；
4. `DataType` variant；
5. copy/owned 属性；
6. JSON 投影类别；
7. enum variant 的公共文档和必要的 Serde adapter 元数据；
8. 保留 typed getter 所需的 method identifiers（仅在确实用于生成 getter
   时记录）。

概念示例：

```text
Bool       => bool,      DataType::Bool,       copy,  json_bool
Date       => NaiveDate, DataType::Date,       copy,  json_string, chrono
BigInteger => BigInt,    DataType::BigInteger, owned, json_string, big-number
```

### 8.3 生成范围

类型表通过 consumer macros 生成完整 item 或完整 match，不依赖“宏在 enum
variant 列表中展开”的不稳定写法。生成范围包括：

1. `Value` 与 `MultiValues` enum variants；
2. `From<T>`/collection `From` 实现；
3. `data_type()`；
4. `MultiValues::count()`、`clear()`、`add()`、`to_value()`；
5. `Value` 到 `DataConverter` 的桥接；
6. 自然 JSON 的机械映射分派；
7. feature gate 的一致传播。

variant 的简短公共文档由表中静态元数据生成；复杂转换规则、错误策略和
面向用户的长文档保持手写，避免宏吞噬业务逻辑。

### 8.4 文件组织

计划新增 crate 私有的 `src/value_type_table.rs`，替代并删除
`src/multi_values/multi_values_type_table.rs`。删除 typed API 后，
`multi_values_adders.rs` 与 `multi_values_setters.rs` 不再有职责，也一并删除。
`value_accessors.rs` 保留 typed getters、borrowed getters 和 JSON 辅助 API。

## 9. typed setter/adder 删除范围

### 9.1 规则

以下 27 个类型族全部适用删除规则：

| singular stem | plural stem |
|---|---|
| `bool` | `bools` |
| `char` | `chars` |
| `int8` | `int8s` |
| `int16` | `int16s` |
| `int32` | `int32s` |
| `int64` | `int64s` |
| `int128` | `int128s` |
| `uint8` | `uint8s` |
| `uint16` | `uint16s` |
| `uint32` | `uint32s` |
| `uint64` | `uint64s` |
| `uint128` | `uint128s` |
| `intsize` | `intsizes` |
| `uintsize` | `uintsizes` |
| `float32` | `float32s` |
| `float64` | `float64s` |
| `biginteger` | `bigintegers` |
| `bigdecimal` | `bigdecimals` |
| `string` | `strings` |
| `date` | `dates` |
| `time` | `times` |
| `datetime` | `datetimes` |
| `instant` | `instants` |
| `duration` | `durations` |
| `url` | `urls` |
| `string_map` | `string_maps` |
| `json` | `jsons` |

对每一行删除：

1. `Value::set_{singular}`；
2. `MultiValues::set_{singular}`；
3. `MultiValues::set_{plural}`；
4. `MultiValues::set_{plural}_slice`；
5. `MultiValues::add_{singular}`；
6. `MultiValues::add_{plural}`；
7. `MultiValues::add_{plural}_slice`。

合计删除 `27 + 81 + 81 = 189` 个方法。`0.8.0` 直接删除，不保留
`#[deprecated]` 过渡期，也不增加 `typed-api` 兼容 feature。

### 9.2 替代 API

```rust
value.set(42_i32);
values.set(42_i32);
values.set(vec![1_i32, 2]);
values.set(&slice[..]);
values.add(42_i32)?;
values.add(vec![1_i32, 2])?;
values.add(&slice[..])?;
```

typed getters 和 borrowed getters 保留，因为它们可以表达借用并避免泛型
getter 的 clone，不属于完全重复 API。

`NamedValue`、`NamedMultiValues` 没有另一套 inherent typed methods；通过
`Deref`/`DerefMut` 自动使用保留的泛型 API，因此无需额外删除清单。

## 10. setter 与错误契约

`Value::set()` 和 `MultiValues::set()` 对所有可编译输入都只执行替换，不可能
返回运行时错误，改为：

```rust
pub fn set<T>(&mut self, value: T)
where
    T: Into<Self>;
```

`MultiValues::add()` 仍可能遇到类型不匹配，继续返回 `ValueResult<()>`，并
统一错误字段：

```text
expected = 容器当前 DataType
actual   = 传入值的 DataType
```

`ValueError` 保留：

1. `NoValue`：严格 getter 访问 unset 或不存在的 first item；
2. `TypeMismatch`：严格 getter/add 类型不匹配；
3. `DataConversion(DataConversionError)`，仅在 `converter` 下存在；
4. `DataListConversion(DataListConversionError)`，仅在 `converter` 下存在。

删除没有生产者的 `IndexOutOfBounds`。conversion API 不再把
`Missing`/`EmptyCollection` 压缩成 `NoValue`，以保留新版
`rs-datatype` 的结构化语义。

`from_serializable()` 和 `deserialize_json()` 继续使用上述结构化 conversion
error，因此与其他 conversion API 一样由 `converter` 控制；不新增一套仅供
`json` feature 使用的平行错误类型。

## 11. 默认转换选项

以下三个入口改用共享默认引用：

```rust
Value::to()          -> self.to_with(DataConversionOptions::default_ref())
MultiValues::to()    -> self.to_with(DataConversionOptions::default_ref())
MultiValues::to_list()
                     -> self.to_list_with(DataConversionOptions::default_ref())
```

行为必须与 `DataConversionOptions::default()` 完全一致，但常用读取路径不再
重复构造 Boolean literals、collection delimiters 等 Vec。

## 12. 文档修改

1. 删除“Zero-Cost Abstraction”和“complete serde support”等无法由契约或
   benchmark 支撑的绝对表述。
2. 明确 `get` 是严格访问，`to` 是受 conversion options 控制的转换。
3. 明确 unset、set-but-empty、empty inner value 三者区别。
4. 明确 tagged wire 与自然 JSON 投影的区别。
5. 明确非有限浮点可以存在于内存，但不能进入默认 Serde/JSON 协议。
6. 记录 feature matrix 和 `all` 便捷 feature。
7. 记录 `IntSize`/`UIntSize` 是平台相关类型，不保证跨架构持久化值域一致。
8. README、README.zh_CN、crate-level rustdoc 和 doctests 同步迁移到泛型
   setter/adder。

## 13. 下游迁移

### 13.1 rs-config

1. 从 crates.io 依赖 `qubit-datatype 0.5`，依赖 `qubit-value 0.8` 并启用
   `qubit-value/all`；仅在 0.8 发布前的仓内验证命令中通过 Cargo patch 指向本地
   `rs-value`。
2. `Property`/`Config` setter 适配无返回值的泛型 `set()`，删除不可能发生的
   `ValueError` 映射。
3. 配置 missing/default/fallback 判断改用 `is_unset()`；显式空列表不再触发
   missing fallback。
4. 如 `Property::is_empty()` 仍有配置域价值，使用 `count() == 0` 实现并在
   文档中声明它会同时覆盖 unset 和 set-but-empty；不得用于 missing 判断。
5. `property_to_json_value()` 委托 `MultiValues::to_json_value()` 并传播
   non-finite 错误。
6. 删除对 `ValueError::IndexOutOfBounds` 和已删除 typed API 的处理。

### 13.2 rs-metadata

1. 从 crates.io 依赖 `qubit-datatype 0.5`，依赖 `qubit-value 0.8` 并启用
   `qubit-value/all`；仅在 0.8 发布前的仓内验证命令中通过 Cargo patch 指向本地
   `rs-value`。
2. 数值类型分类使用 `Value::is_numeric()` 或 `DataType` family methods。
3. 专用数值归一化和精确/近似比较策略保留在 `rs-metadata`。
4. tagged wire 的所有 variant 补充或保留 golden tests。
5. 非有限浮点进入 metadata wire 时必须返回序列化错误。

## 14. 测试设计

### 14.1 状态矩阵

对 `Value` 和每个启用的 `MultiValues` family 验证：

1. unset 的 `is_unset/get/to/to_list/*_or` 行为；
2. set-but-empty 的对应行为；
3. `clear()` 与 `unset()` 的状态差异；
4. tagged Serde 中 unset 与 empty collection 的不同表示；
5. 自然 JSON 中 `null` 与 `[]` 的不同表示。

### 14.2 JSON 与浮点

至少覆盖：

1. finite `f32/f64` tagged round-trip；
2. `NaN`、`+Infinity`、`-Infinity` 序列化失败；
3. `MultiValues` 中任意位置的非有限值失败，并保留自然 JSON 错误索引；
4. 裸 `NaN`/`Infinity` JSON 输入失败；
5. `null` 不能反序列化为浮点 variant；
6. `from_serializable()` 对顶层及嵌套 `NaN`/Infinity 失败，合法 `null`
   保持为 `null`；
7. 所有启用 variant 的 tagged golden fixture；
8. 所有自然 JSON 映射类别的 fixture。

### 14.3 Feature matrix

至少执行：

```text
cargo check --no-default-features
cargo check --no-default-features --features chrono
cargo check --no-default-features --features big-number
cargo check --no-default-features --features url
cargo check --no-default-features --features json
cargo check --no-default-features --features converter
cargo test --all-features
```

每个 feature 组合都应检查对应 variant/API 是否存在，且关闭 feature 时不残留
无条件 import 或依赖。`json`-only 构建应包含 `Json` variant 和
`from_json_value()`，但不包含 conversion helpers；`converter` 构建应包含
自然 JSON 投影和全部结构化 conversion helpers。

### 14.4 回归与下游

1. `rs-value`：fmt、clippy、tests、doctests、rustdoc、coverage。
2. `rs-config`：all-features tests 和 doctests。
3. `rs-metadata`：all-features tests 和 doctests。
4. 搜索三个仓库，确认没有已删除 typed setter/adder 的生产调用。
5. 确认 `DataConversionOptions::default()` 在 rs-value 三个默认入口中不再
   出现。

## 15. 实施顺序

1. 切换 crates.io `rs-datatype 0.5` 依赖并建立 feature matrix。
2. 引入共享类型表，先保持行为不变并验证全 feature 编译。
3. 实现 unset 状态模型并迁移默认值语义。
4. 实现严格 Serde 浮点校验和自然 JSON 投影。
5. 删除 189 个 typed setter/adder，修改泛型 setter 返回类型。
6. 清理 `ValueError::IndexOutOfBounds`，统一结构化 conversion errors。
7. 接入 `DataConversionOptions::default_ref()`。
8. 更新 `rs-config`、`rs-metadata`。
9. 更新中英文文档、golden tests 和 feature matrix tests。
10. 完成全量与下游验证。

## 16. 验收标准

1. 默认 feature 为空，`all` 可一次启用完整功能。
2. 关闭任意 rich feature 后，其直接依赖和具体 variant 不进入构建图。
3. 非有限浮点不能被默认 Serde/JSON 静默写为 `null`。
4. unset 与 set-but-empty 在 API、默认值逻辑、Serde 和自然 JSON 中均可区分。
5. 189 个 typed setter/adder 已直接删除，泛型迁移完成。
6. `Value::set()`、`MultiValues::set()` 返回 `()`，下游不再映射不可能错误。
7. 类型到 variant/DataType/converter/JSON 的机械映射来自一个私有类型表。
8. `NamedValue`、`NamedMultiValues` 功能保留。
9. 三个默认转换入口均使用 `DataConversionOptions::default_ref()`。
10. `rs-value`、`rs-config`、`rs-metadata` 的相关测试、lint、文档检查通过。
