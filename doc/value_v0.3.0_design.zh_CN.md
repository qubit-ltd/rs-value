# `rust-value` `v0.3.0` 设计文档

## 文档信息

- 文档名称：`rust-value v0.3.0 设计文档`
- 文档版本：`v1.0`
- 创建日期：`2026-04-09`
- 当前版本：`0.2.0`
- 目标版本：`0.3.0`

## 1. 背景

`qubit-value` 当前已经能稳定承载基础标量、字符串、日期时间和大数类型，但对更完整的配置场景仍有明显缺口：

1. 缺少 `isize` / `usize`。
2. 缺少 `std::time::Duration`。
3. 缺少 `url::Url`。
4. 缺少通用键值映射结构。
5. 缺少自定义枚举和复杂结构的统一扩展入口。

这些缺口已经开始影响上层 crate 的结构化配置能力，尤其是 `qubit-config` 和 `qubit-http` 的配置映射。

## 2. 目标

`v0.3.0` 的目标不是把 `qubit-value` 演化成“可容纳任意业务对象”的动态对象系统，而是在保持现有“通用、可序列化、跨模块复用”的边界下，补齐一组高频且可移植的数据类型，并提供对复杂结构的受控扩展能力。

## 3. 设计原则

1. 优先支持跨领域、跨 crate 通用的数据类型。
2. 不把 HTTP、数据库或业务私有结构直接引入 `Value` / `MultiValues` 的核心枚举。
3. 对复杂结构提供统一 escape hatch，但不让 escape hatch 反过来替代基础类型建模。
4. 尽量保持 `Value` / `MultiValues` 的类型可枚举、可序列化、可测试。

## 4. `v0.3.0` 新增数据类型需求

### 4.1 平台整数类型

新增原生支持：

1. `isize`
2. `usize`

要求：

1. `Value` / `MultiValues` / `NamedValue` / `NamedMultiValues` 全链路支持。
2. `get<T>()`、`set<T>()`、`add<T>()`、`get_first<T>()` 的泛型入口可直接工作。
3. `serde` 序列化后能够稳定往返。

### 4.2 标准库与通用生态类型

新增原生支持：

1. `std::time::Duration`
2. `url::Url`

要求：

1. `Duration` 作为独立数据类型建模，而不是退化成字符串。
2. `Url` 作为独立数据类型建模，而不是退化成字符串。
3. 保持 `serde` round-trip 能力。
4. 提供清晰的字符串转换规则，便于上层配置系统从文本来源解析。

### 4.3 简单映射结构

新增原生支持：

1. `HashMap<String, String>`

原因：

1. 这是配置场景里最常见、最可移植的 map 形态。
2. 它足以覆盖“默认 headers 的中间表达”“标签集合”“扩展属性”等通用需求。
3. 相比直接引入 `HeaderMap` 一类领域对象，它不带协议语义，复用边界更干净。

要求：

1. `Value` 支持单值 map。
2. `MultiValues` 支持 map 列表。
3. `serde` round-trip 稳定。

## 5. 自定义枚举与复杂结构的支持策略

### 5.1 结论

`qubit-value v0.3.0` 不应为任意自定义 enum、struct、或领域对象继续扩展新的原生枚举变体；应采用“两层支持”策略：

1. 对高频、通用、跨领域的数据类型提供原生变体。
2. 对任意自定义 enum 和复杂结构，提供基于 `serde_json::Value` 的通用承载能力。

也就是说：

1. `HashMap<String, String>` 原生支持。
2. `HeaderMap` 不原生支持。
3. 复杂 `HashMap`、嵌套对象、自定义 enum、自定义 struct，通过 JSON 能力支持。

### 5.2 为什么不把 JSON 字符串作为唯一方案

仅使用 JSON 字符串有几个明显问题：

1. 值进入容器后退化为普通 `String`，丢失类型边界。
2. 无法在 `Value` 层区分“普通字符串”和“结构化 JSON 文本”。
3. 上层每次读取都要重复 parse，错误会推迟到业务侧。
4. 不利于调试、校验和类型发现。

因此，`v0.3.0` 的推荐方案不是“只支持 JSON 字符串”，而是：

1. 原生支持 `HashMap<String, String>` 这类简单、稳定、通用的数据结构。
2. 额外新增 `Json(serde_json::Value)` 这一 escape hatch，用于承载复杂结构。
3. JSON 字符串仅作为配置来源或外部接口输入格式，不作为 `qubit-value` 内部唯一表示。

### 5.3 自定义 enum 支持方式

自定义 enum 的支持不通过给每个 enum 增加独立变体实现，而通过以下能力实现：

1. 当 enum 适合文本表达时，可通过字符串和约定转换实现。
2. 当 enum 具有更复杂的 serde 表达时，可通过 `Json(serde_json::Value)` 存储。
3. 对外提供基于 `serde` 的辅助 API，例如“从可序列化对象写入 JSON 值”和“从 JSON 值反序列化为目标类型”。

这条路径同样适用于自定义 struct、复杂 map、以及未来不希望原生纳入 `Value` 的领域对象。

## 6. 建议新增的核心能力

### 6.1 数据类型枚举扩展

`qubit_common::lang::DataType` 需要同步新增：

1. `IntSize`
2. `UIntSize`
3. `Duration`
4. `Url`
5. `StringMap`
6. `Json`

命名可以在实现阶段微调，但语义层面需要以上能力。

### 6.2 API 扩展

建议新增或补齐：

1. `Value::new(Duration)`
2. `Value::new(Url)`
3. `Value::new(HashMap<String, String>)`
4. `Value::from_json_value(serde_json::Value)`
5. `Value::from_serializable<T: Serialize>(value: &T) -> ValueResult<Self>`
6. `Value::deserialize<T: DeserializeOwned>(&self) -> ValueResult<T>`

`MultiValues` 同理补齐对应泛型支持。

## 7. 非目标

以下内容不在 `v0.3.0` 原生支持范围内：

1. `http::HeaderMap`
2. 任意业务 crate 的私有结构
3. 递归 `HashMap<String, Value>` 这类完整动态对象系统
4. 通过不断增加枚举变体去覆盖所有第三方 crate 类型

## 8. 与上层 crate 的协作方式

### 8.1 对 `qubit-config`

`qubit-config` 可直接受益于：

1. `usize` / `isize`
2. `Duration`
3. `Url`
4. `HashMap<String, String>`
5. `Json`

这样 `Config -> 结构化配置对象` 的映射可以更少依赖手工字符串解析。

### 8.2 对 `qubit-http`

`qubit-http` 仍不应直接依赖 `HeaderMap` 进入 `qubit-value`。推荐链路为：

1. `rust-config` 读取 `HashMap<String, String>` 或 `Json`
2. `rust-http` 在适配层把它转换为 `HeaderMap`

这样既能满足配置需求，又不破坏 `qubit-value` 的通用性边界。

## 9. 验收标准

1. `usize/isize/Duration/Url/HashMap<String, String>/Json` 全部具备单值、多值、具名值支持。
2. 泛型 API 与具名 API 都能工作。
3. 所有新增类型均具备 `serde` round-trip 测试。
4. 自定义 enum 至少可以通过 serde JSON 路径完成往返。
5. 文档中明确声明：`HeaderMap` 等领域对象不纳入原生变体，复杂结构通过 JSON 路径承载。
