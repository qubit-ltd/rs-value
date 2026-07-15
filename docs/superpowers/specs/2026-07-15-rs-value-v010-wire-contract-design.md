# rs-value 0.10 值契约与 Wire V1 设计

## 背景与目标

`rs-value 0.9` 已经明确区分标量、集合、未设置值和具体空集合，
并为宽整数、大数、Duration 和非有限浮点数建立了严格的 Serde 规则。
本次修改解决三个仍然存在的契约问题：

1. `MultiValues::get_or` 和 `MultiValues::get_first_or` 在容器为
   `Unset` 时绕过严格类型检查；
2. `Value`、`MultiValues` 和 `ValueError` 是可扩展的公开 enum，
   但下游目前可以对它们进行穷尽匹配；
3. 运行时 enum 的派生 Serde 直接定义 wire 格式，运行时变体变化会隐式改变协议。

目标版本为 `qubit-value 0.10.0`。本次允许破坏性变更，不读取旧 wire，
也不提供旧格式迁移或兼容开关。

## 范围

本次范围包括：

- 修复两个默认严格读取方法并增加回归测试；
- 为 `Value`、`MultiValues` 和 `ValueError` 增加 `#[non_exhaustive]`；
- 引入公开且版本化的 `ValueWireV1`；
- 让 `Value`、`MultiValues` 和 `ValueContainer` 的直接 Serde 委托给
  `ValueWireV1`；
- 更新 `NamedValue`、`NamedMultiValues`、README 和用户指南中的 wire 契约；
- 更新 `rs-config`、`rs-metadata` 的依赖、错误映射和 wire 测试；
- 将 `qubit-value` 版本提升到 `0.10.0`。

本次不包括：

- 第四项泛型读取/转换目标 trait 门面；
- 旧 tagged Serde 格式的兼容读取；
- 新的数据类型、转换规则、比较规则或自然 JSON 规则；
- 删除当前下游未使用的 `NamedValue` 或 `NamedMultiValues`。

## 默认严格读取修复

严格读取必须先验证请求类型是否与容器声明类型一致，再决定是否使用默认值。

### `MultiValues::get_or`

- `Unset(T)` 且请求类型为 `T`：返回默认列表；
- `Unset(T)` 且请求类型不是 `T`：返回 `ValueError::TypeMismatch`；
- 具体空 `Vec<T>`：返回空列表，不使用默认值；
- 具体非空 `Vec<T>`：返回全部值；
- 具体其他类型：返回 `ValueError::TypeMismatch`。

实现应调用现有严格 `get`，并且只将匹配类型的 `NoValue` 映射为默认值。

### `MultiValues::get_first_or`

- `Unset(T)` 且请求类型为 `T`：返回默认标量；
- `Unset(T)` 且请求类型不是 `T`：返回 `ValueError::TypeMismatch`；
- 具体空 `Vec<T>`：返回 `ValueError::NoValue`，不使用默认值；
- 具体非空 `Vec<T>`：返回首元素；
- 具体其他类型：返回 `ValueError::TypeMismatch`。

由于具体空集合与匹配类型的 `Unset` 都会从 `get_first` 返回 `NoValue`，
默认值分支还必须确认原容器确实为 `Unset`。

## 公开 enum 演进契约

以下公开 enum 增加 `#[non_exhaustive]`：

- `Value`；
- `MultiValues`；
- `ValueError`。

这样可以保证未来新增运行时类型或错误变体时，下游必须保留兜底分支。
由于该属性本身会破坏现有穷尽匹配，因此与 wire 改造一起在 0.10 发布。

`ValueContainer` 本次不增加该属性。它只有稳定的 `Scalar` 和 `Collection`
两种形状，也不受 Cargo feature 合并影响；扩大到该类型不属于已确认范围。

`rs-config` 当前对 `ValueError` 做穷尽匹配。它需要新增一个保留原始
`ValueError` source 和配置键名的通用错误变体，并用兜底分支映射未来错误。
现有四种错误继续使用当前的精细映射。

## Wire V1 协议

### 统一信封

所有类型保留直接 `Serialize`/`Deserialize`，但输出和输入统一通过公开
`ValueWireV1`：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

信封只有两个字段：

- `version`：必须是 JSON 数字 `1`；
- `value`：一个明确标记形状和数据类型的值。

信封拒绝未知字段、缺少字段、非数字版本和除 `1` 以外的版本。

### 标量

具体标量：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

未设置标量：

```json
{"version":1,"value":{"scalar":{"unset":"int32"}}}
```

`Value::Json(null)` 仍然是具体值：

```json
{"version":1,"value":{"scalar":{"json":null}}}
```

因此它不会与 `Unset(DataType::Json)` 混淆。

### 集合

具体集合：

```json
{"version":1,"value":{"collection":{"int32":[1,2]}}}
```

具体空集合：

```json
{"version":1,"value":{"collection":{"int32":[]}}}
```

未设置集合：

```json
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

形状不从集合长度推断。单元素集合始终使用 `collection`。

### 类型标签和载荷

类型标签使用 `DataType::as_str()` 对应的规范小写名称：

- `bool`、`char`；
- `int8`、`int16`、`int32`、`int64`、`int128`；
- `uint8`、`uint16`、`uint32`、`uint64`、`uint128`；
- `float32`、`float64`；
- `biginteger`、`bigdecimal`；
- `string`、`date`、`time`、`datetime`、`instant`；
- `duration`、`url`、`stringmap`、`json`。

载荷继续遵守现有严格规则：

- `int128` 和 `uint128` 使用规范十进制字符串；
- `biginteger` 和 `bigdecimal` 使用规范十进制字符串；
- Duration 使用 `{ "secs": u64, "nanos": u32 }`，且 `nanos < 1_000_000_000`；
- 非有限 `float32`/`float64` 无法序列化或反序列化；
- 集合元素逐项使用相同规则；
- `stringmap` 和 `json` 保留其结构化载荷。

### 类型入口约束

- `Value` 序列化为 `scalar`，反序列化时拒绝 `collection`；
- `MultiValues` 序列化为 `collection`，反序列化时拒绝 `scalar`；
- `ValueContainer` 根据其实际形状序列化，并接受两种形状；
- `Value` 与 `ValueContainer::Scalar` 的 wire 完全相同；
- `MultiValues` 与 `ValueContainer::Collection` 的 wire 完全相同；
- `NamedValue` 和 `NamedMultiValues` 保留当前外层字段，内部 `value`
  字段使用统一 V1 信封。

旧的外部标签格式，例如 `{"Int32":42}`、`{"Unset":"int32"}` 和
`{"Scalar":{"Int32":42}}`，必须被拒绝。

## 代码结构

新增 `src/value_wire.rs`，原因是版本化协议、运行时转换和 Serde 委托构成一个
独立边界，不应继续混入保存基础载荷适配器的 `src/wire.rs`。

`src/value_wire.rs` 负责：

- 公开 `ValueWireV1`；
- V1 版本验证；
- 私有 scalar/collection wire payload；
- `Value`、`MultiValues`、`ValueContainer` 与 `ValueWireV1` 的拥有型转换；
- `Value`、`MultiValues`、`ValueContainer` 的手写 Serde 实现；
- 序列化时使用借用 payload，避免对字符串、映射、JSON、大数和集合进行深度 clone。

`src/wire.rs` 继续只负责稳定基础载荷适配器，包括有限浮点数、宽整数、
规范大数和 Duration。

wire 类型表与运行时 `value_type_table.rs` 相互独立。wire 表明确列出 V1 支持的
类型、标签和载荷适配器。转换 match 必须保持穷尽；新增运行时变体时，如果没有
显式更新 V1 或决定创建 V2，编译必须失败。该重复是有意的协议防护，不应合并回
运行时类型表。

`ValueWireV1` 提供以下公开转换：

- `From<Value>`；
- `From<MultiValues>`；
- `From<ValueContainer>`；
- `From<ValueWireV1> for ValueContainer`。

直接还原为 `Value` 或 `MultiValues` 由各自的 `Deserialize` 负责 shape 校验。
公开 DTO 不新增一个仅用于 shape 错误的公共错误体系。

## 测试策略

所有行为修改按 TDD 实施。

### 严格默认读取

先增加失败测试，覆盖：

- 匹配类型的 `Unset` 使用默认值；
- 不匹配类型的 `Unset` 返回精确的 `TypeMismatch`；
- 具体空列表不触发 `get_or` 默认值；
- 具体空列表从 `get_first_or` 返回 `NoValue`。

### `#[non_exhaustive]`

在 `public_api_boundary_tests` 中创建临时外部 consumer，并分别尝试对
`Value`、`MultiValues`、`ValueError` 做穷尽匹配。增加属性前 consumer 应编译
成功，使新测试处于 RED；增加属性后应以 non-exhaustive match 诊断失败。

### Wire V1

golden tests 覆盖所有启用类型的标量和集合，并验证：

- 精确 V1 JSON；
- `ValueWireV1`、`Value`、`MultiValues`、`ValueContainer` 往返；
- fixture 的具体 `DataType` 集合与 `DataType::ALL` 完全一致；
- 版本缺失、错误版本、未知字段、未知类型和 shape 不匹配均失败；
- 旧 wire 明确失败；
- 宽整数、大数、Duration 和非有限浮点数边界保持严格；
- `Unset`、具体空集合、单元素集合和 `Json(null)` 保持不同状态。

更新 named wrapper、`rs-config` 和 `rs-metadata` 中所有直接 wire 断言。
自然 JSON 投影测试保持原样，用于证明本次没有改变另一条 JSON 边界。

## 下游修改

### rs-config

- `qubit-value` 依赖版本改为 `0.10`；
- `ConfigError` 增加未来 `ValueError` 的兜底变体；
- 保留当前 `NoValue`、`TypeMismatch`、单值转换和列表转换的精细映射；
- 更新 ValueContainer Serde 相关断言和文档；
- 不修改本轮暂缓的泛型约束。

### rs-metadata

- `qubit-value` 依赖版本改为 `0.10`；
- `Metadata` 和 filter wire 继续直接 derive Serde；
- 更新 Metadata、Condition 和 filter 的 V1 wire 断言；
- 数值比较、schema 校验和 `FromMetadataValue` 保持不变。

## 版本与文档

- `qubit-value` 包版本提升为 `0.10.0`；
- README、中文 README、用户指南和依赖示例统一改为 `0.10`；
- 文档明确区分 V1 类型保留 wire 与自然 JSON 投影；
- 文档明确旧格式不受支持；
- `rs-config` 和 `rs-metadata` 仅更新依赖要求，不额外提升它们自身包版本。

## 格式化与验证

不得直接运行 `cargo fmt`。每个发生修改的仓库使用其根目录的：

```bash
./align-ci.sh
./ci-check.sh
```

`align-ci.sh` 产生的项目规范化格式改动属于本次授权范围。当前 `rs-value`
工作区中已有的大范围非项目 rustfmt 格式差异将由该脚本按项目配置统一处理。

最终验证至少包括：

- `rs-value` 全 feature matrix；
- `rs-value` 全量 CI；
- `rs-config` 全量 CI 与全 feature 测试；
- `rs-metadata` 全量 CI 与测试；
- `rs-value` 覆盖率，确认新增协议分支和错误分支被执行；
- 三个仓库的 diff 检查，确认没有修改第四项泛型门面或其他无关行为。

未经用户明确要求，不执行 `git add`、`git commit` 或 `git push`。
