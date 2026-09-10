# qubit-value 架构与 Wire 设计

[English version](design.md) · [README](../README.zh_CN.md) · [用户手册](user_guide.zh_CN.md) · [API 文档](https://docs.rs/qubit-value)

本文记录 `qubit-value` 0.12 的架构边界与兼容性规则，面向本 crate 的维护者，以及基于这套
值模型设计协议或下游 crate 的开发者。

<a id="scope"></a>
## 范围

`qubit-value` 是供上层 key-value 系统复用的类型化运行时值层，承担四项职责：

- 表示单个类型化标量、同类型集合，以及两种形态之间的明确区别；
- 提供严格读取和按需启用的策略化转换；
- 为所有支持的运行时类型定义语义相等与 hash 规则；
- 将运行时值适配为有损的自然 JSON，或保留类型的 Wire V1 格式。

属性名校验、schema、数据源加载、配置优先级、持久化、传输 framing 和分布式 identity 不属于
本 crate 的职责，应由下游系统处理。`NamedValue` 和 `NamedMultiValues` 只是轻量 name 包装，
不是通用 key-value 存储。

公共容器和适配器属于稳定 API；私有存储 enum 与序列化 helper 是实现细节，只要可观察行为
保持不变，就可以调整其内部组织。

<a id="type-model"></a>
## 类型模型

这套模型有两个彼此独立的维度：运行时 `DataType` 与值的形态。二者分离后，集合长度或 JSON
语法就不会悄悄改变调用方契约。

| 公共类型 | 职责 | 形态 |
| --- | --- | --- |
| `Value` | 持有一个具体类型的 payload，或 `Unset(DataType)` | 标量 |
| `MultiValues` | 持有一个同类型 vector，或未设置的元素类型 | 集合 |
| `ValueContainer` | 包装 `Scalar(Value)` 或 `Collection(MultiValues)` | 显式 union |
| `NamedValue` | 将 name 与 `Value` 关联 | 具名标量 |
| `NamedMultiValues` | 将 name 与 `MultiValues` 关联 | 具名集合 |

内部封闭的 value table 统一映射 25 个 `DataType` 变体、Rust 存储类型、feature gate、自然
JSON 类别和稳定的 Wire V1 tag。标量与集合表示、构造函数、借用投影、identity 逻辑和 Wire
DTO 都由该表驱动，因此两组容器能够保持一致，而生成的存储 enum 不需要公开。

`ValueContainer` 从不推断形态。标量 `Int32(42)`、单元素 `Int32` 集合、空 `Int32` 集合和未
设置的 `Int32` 集合是四种不同状态。

<a id="ownership-and-borrowed-views"></a>
## 所有权与借用视图

`Value`、`MultiValues` 和 `ValueContainer` 持有 payload 的所有权，并通过稳定方法隐藏私有
表示。克隆这些类型时，字符串、map、JSON tree 和集合 vector 等 owned payload 也会被克隆。

`Value::view()` 和 `MultiValues::view()` 分别返回 `ValueRef<'_>` 和
`MultiValuesRef<'_>`。这两个 `non_exhaustive` enum 提供语义化变体，同时从源对象借用非
copy payload。可复制的标量按值投影；字符串、大值、map、JSON tree 和集合 slice 继续保持
借用。由于借用视图未来可以扩展，下游 match 必须保留 wildcard 分支。

两个视图及 owned-to-borrowed 分派都由封闭的 value table 生成。`MultiValuesRef` 提供
`data_type`、`len`、`is_empty` 和按索引读取的 `get`，直接返回 `ValueRef`，不构造临时标量容器。
启用 `converter` 时，`DataConverter` 可直接接收 `ValueRef`。配置 Serde visitor 因而能保留原始
数值类型并借用原生集合，无需先物化一棵自然 JSON tree。

Wire 层同样明确区分所有权：

| 适配器 | 所有权 | 用途 |
| --- | --- | --- |
| `ValueWireV1` | 持有完整的带版本 envelope | 存储或传输完整 Wire 值 |
| `ValueWirePayloadV1` | 持有不带版本的 typed shape | 嵌入由外层协议管理版本的文档 |
| `ValueWireRefV1<'a>` | 借用完整 envelope 的 payload | 不克隆运行时值直接编码 |
| `ValueWirePayloadRefV1<'a>` | 借用不带版本的 payload | 不克隆地嵌入和编码 |

借用 Wire 构造函数可能失败。它们在暴露可序列化视图前，会检查 JSON 无法表达的约束，包括
非有限浮点数和超出范围的 `BigDecimal` scale。借用适配器的生命周期不能超过源运行时值。

<a id="value-semantics"></a>
## 值语义

Unset 表示带类型的缺失，而不是无类型的 null。`Value::new_unset(T)` 和
`MultiValues::new_unset(T)` 会保留 `T`，使 schema-aware 调用方能在具体 payload 尚未出现时
继续保有声明类型。

以下状态刻意保持不同：

| 状态 | 含义 |
| --- | --- |
| `Value::new_unset(DataType::Json)` | 没有标量值；声明类型为 `Json` |
| `MultiValues::new_unset(DataType::String)` | 没有集合；声明元素类型为 `String` |
| `MultiValues::String(Vec::new())` | 已存在的、同类型的空集合 |
| `Value::Json(serde_json::Value::Null)` | 具体 JSON 标量，其 payload 为 JSON `null` |

严格 getter 要求存储变体与请求的 Rust 类型一致，只会报告结构化 missing 状态或
`TypeMismatch`，不会隐式转换。只有启用 `converter` 后才提供转换方法，具体策略与资源记账
委托给 `qubit-datatype`。

默认值的适用范围保持收敛。严格 `get_or` 只对 unset 值使用默认值；当所选转换策略把源值
判定为 missing 时，转换类 default API 也可以使用默认值。除非具体 API 明确定义，否则空的
具体集合、类型不匹配和普通非法转换都不会被静默替换。

`set` 替换当前标量或整个集合。`MultiValues::add` 可能失败，因为新增值必须与已有元素类型
一致。访问、转换和 Wire 序列化始终独立于元素数量保留形态。

<a id="identity-and-hashing"></a>
## Identity 与 Hash

`Value`、`MultiValues` 和 `ValueContainer` 共用一套相等与 hash 语义：

- 运行时变体以及标量/集合形态参与 identity；
- 集合顺序和长度参与 identity；
- `+0.0` 与 `-0.0` 视为同一浮点 identity，所有 NaN bit pattern 也规范化为同一 identity，
  从而维持 `Eq` 与 `Hash` 的一致性；
- `BigDecimal` hash 会规范化 coefficient 末尾的零与 effective scale，以匹配数值相等；
- `StringMap` 和 JSON object identity 不依赖 map 的迭代顺序；
- JSON array 的顺序仍然有意义。

JSON 相等与 hash 使用迭代遍历，避免递归调用。带 budget 的 hash API 会在 staged transaction
中预检查完整 JSON payload；若预算失败，调用方 hasher 与已提交 budget 都保持不变。

`Hash` 输出只适用于进程内 Rust 集合，不能作为持久化 fingerprint、规范协议 digest 或分布式
缓存 key。具体结果可能随 hasher、target、依赖版本、feature 或 crate 实现变化。

<a id="natural-json-and-wire-json"></a>
## 自然 JSON 与 Wire JSON

自然 JSON 和 Wire JSON 面向不同的边界问题。

| 属性 | 自然 JSON | Wire V1 JSON |
| --- | --- | --- |
| 首要目标 | 与普通 JSON 消费方互操作 | 恢复运行时类型和形态 |
| 类型 tag | 省略 | 保留 |
| 标量/集合形态 | 只通过 JSON scalar/array 语法体现 | 使用显式 `scalar` 或 `collection` tag |
| Unset | 投影为 JSON `null` | 保留 `Unset(DataType)` |
| Round trip | 刻意有损 | 在 V1 支持范围内保留类型和形态 |
| 入口 | `to_json_value*` | `ValueWire*V1` 与有界编解码 helper |

自然 JSON 将运行时值投影成 `serde_json::Value`。它会排序字符串 map 和 JSON object 的 key，
以得到确定输出；同时拒绝非有限浮点，并对完整投影使用同一 conversion budget。仅凭 JSON 无法
恢复原整数宽度、区分 unset 与具体 JSON null，或恢复声明类型。

投影准备阶段借用源文本，完成准入后才分配最终 JSON string。数字测量使用有界格式化，
不分配临时堆字符串；需要格式化的富类型缓存结果供最终投影复用，使转换只记账一次。
准备失败不会产生可用的部分结果。这些优化保持原有自然 JSON 类别和资源限制。

Wire JSON 使用明确的版本、shape 和类型 tag。DTO 实现 `Serialize`，但刻意不实现通用
`Deserialize`。完整的不可信 JSON 文档必须通过有界 decode helper 读取；嵌入式 payload 应在
外层 decoder 的共享 session 中使用 `ValueWireV1Seed` 或 `ValueWirePayloadV1Seed`。

<a id="wire-v1-compatibility"></a>
## Wire V1 兼容性

独立 V1 文档包含数字 `version` 字段和一个 typed `value`；嵌入式 payload 只省略外层版本
字段。以下是有代表性的值：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"string"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
```

V1 是封闭契约，以下各项属于兼容性不变量：

- 版本必须是 JSON number `1`；
- shape 必须且只能是一个 `scalar` 或 `collection` 变体；
- payload 必须且只能是一个已知的小写 V1 类型 tag；
- 未知字段、shape、tag，以及当前 feature 不支持的具体 payload 都会被拒绝；
- `Int128`、`UInt128` 和 `BigInteger` 使用 canonical 十进制文本；
- `BigDecimal` 使用有界 coefficient/scale 表示；
- `Duration` 使用秒和不足一秒的纳秒；
- 浮点数必须有限；
- 在受支持的 canonical JSON 配置下，字符串 map key 和嵌套 JSON object key 按字典序输出。

Externally tagged 表示不属于 V1 输入。已有 V1 tag、shape 或 payload 编码不得原地改义或
扩展；新增运行时类型或不兼容表示时，必须定义新的 Wire 版本。

字节稳定性只针对受支持 `serde_json` 配置产生的 canonical JSON。其他 Serde 格式可以承载这些
DTO，但其字节表示不属于 V1 JSON 稳定性契约。

<a id="budget-and-preflight"></a>
## Budget 与 Preflight

Wire 处理将早期拒绝与权威记账分为两个阶段。

`ValueWireEncodePreflight` 在下游协议排序 key、格式化 payload 或分配最终输出前执行保守的
下界检查。成功的 `check_value`、`check_values` 和 `check_container` 会累计到同一个 checker，
因此外层 object 可以让多个嵌入值共享预算。每次调用都具有原子性：失败时，node、payload byte
和 output byte counter 会恢复到调用前状态。

Preflight 不执行序列化、不保留排序 index，也不证明最终编码一定能通过。估算会刻意省略或少算
部分最终语法和格式化成本。最终 `JsonEncodeSession` 才是权威检查，会在编码期间校验结构
measurement 与实际输出字节。

三个 preflight counter 使用不同单位：`nodes` 统计 JSON value 节点，`payload_bytes` 统计解码后
字符串、key 和数字文本的字节（null 与 boolean 的 payload 为零），`output_bytes` 统计编码语法和
文本的输出下界。字符串和 key 使用 UTF-8 字节；字符串引号以及 object/array 的最少标点计入输出
下界。转义、canonical key 排序、外层 envelope 字段和富类型的精确格式化可能在后续增加字节，
所以 preflight 通过不能替代最终检查。

`new_value_limits` 适用于由外层协议拥有 output budget 的场景。`new_u64_limits` 把下游的
`u64` profile 适配为原生 `usize`；在较窄 target 上，大于 `usize::MAX` 的限制会饱和，而不会
截断。

独立文档的默认 profile 同时把原始输入或输出限制为 1 MiB，并应用共享 value 限制：深度 64、
100,000 个 node、4,096 个 sequence item、4,096 个 map entry、256 KiB key、256 KiB string、
4,096 byte number text，以及 1 MiB value payload。应用可以传入更严格的 profile。嵌入式数据
应由外层协议持有一个 session，将 envelope 与 payload 的资源统一记账。

<a id="feature-matrix"></a>
## Feature Matrix

默认 feature 集为空。feature 决定哪些具体 payload 和 API 可以被物化，但所有 `DataType` 仍可
出现在 unset 声明中。

| Feature | 架构影响 |
| --- | --- |
| `converter` | 启用按策略转换标量和集合 |
| `chrono` | 物化 `Date`、`Time`、`DateTime` 和 `Instant` |
| `big-integer` | 物化 `BigInteger` |
| `big-decimal` | 物化 `BigDecimal` 及其有界 Wire 表示 |
| `big-number` | 同时启用两个大数 feature 的兼容别名 |
| `url` | 物化 `Url` |
| `json` | 物化 `Json`，并启用有界 JSON Wire helper 和 preflight |
| `natural-json` | `converter` 加 `json` 的便捷别名；启用自然 JSON 投影 |
| `redact` | 启用按策略生成的脱敏视图 |
| `all` | 启用全部公共 feature 族 |

生产方和消费方交换具体的可选 payload 时，必须启用该类型所需的相同 feature。未启用对应
feature 的 decoder 会拒绝 payload，而不会强制转换成其他类型。受 feature 控制的 API 不应
隐式进入最小 feature 构建。

<a id="error-model"></a>
## 错误模型

错误按边界分层：

| 错误类型 | 边界 |
| --- | --- |
| `ValueError` | 严格读取、转换和自然 JSON 投影 |
| `ValueMissing` | `ValueError::Missing` 携带的类型化原因 |
| `ValueWireEncodeError` | V1 校验、预算、JSON 语法/序列化和 writer I/O |
| `ValueWireDecodeError` | 有界输入、JSON 语法/data、版本不支持和资源失败 |
| `MeasuredBudgetError<JsonResource, usize>` | 下游编码前直接执行的 preflight 拒绝 |

未设置存储、具体空集合、被转换策略判定为 missing、类型不匹配和非法转换保持可区分。集合
转换错误保留失败的源 index；自然 JSON 投影限额错误保留 data type、可选集合 index，以及
实际测得的资源信息。

`ValueMissing` 使用私有字段保存事实，由 `ValueMissingReason` 分类，同时记录源类型、请求的
目标类型、可选源索引和原始转换错误。存储分类与转换来源彼此独立：unset 读取也可能携带原始
转换失败。调用方应使用 `is_defaultable_for_strict_read()` 或
`is_defaultable_for_conversion()` 决定是否回退，不能认为所有 missing 都允许默认值。
集合某项缺失，以及从具体空集合读取首项，都不允许回退。

Fallback 遵循以下真值表：

| 源状态 | 严格 `get_or*` | 转换 `to_*_or*` |
| --- | --- | --- |
| 未设置的标量或集合 | 默认值 | 默认值 |
| 具体空集合读取首项 | 错误 | 错误 |
| 策略判定为 missing 的标量 | 不适用 | 默认值 |
| 策略判定为 missing 的集合元素（包括第 0 项） | 错误 | 错误 |
| 普通转换错误或类型错误 | 错误 | 错误 |

Wire 错误不包含原始输入内容；decode 错误只保留安全的位置和类别信息。`ValueError` 和 Wire
错误 enum 在需要允许未来增加诊断变体时使用 `non_exhaustive`，下游 match 因此必须有 fallback
分支。

<a id="downstream-integration"></a>
## 下游集成

`rs-config` 使用 `ValueContainer` 保存属性 payload，使配置源和 reader 能够保留标量/集合
形态。值层严格读取使用 `StrictValueRead`；`Config::get` 仍是转换读取。
独立属性值使用 `ValueWireV1` 或 `ValueWireRefV1`。
完整配置编码前，同一个 `ValueWireEncodePreflight` 使用配置的 `u64` 限额 profile，为所有属性
累计保守费用。

`rs-metadata` 保存标量 `Value`，并把 `ValueWirePayloadV1`/
`ValueWirePayloadRefV1` 嵌入自己的带版本 metadata 和 filter 协议。decoder 使用
`ValueWirePayloadV1Seed`，让外层文档持有唯一 decode session；metadata 与 filter encoder
同样会在多个内部值之间复用一个 preflight checker。

Metadata 的 `get` 系列执行严格读取，`convert` 系列显式请求转换。两个下游都保留
`ValueError` 错误来源，并复用 `IntoValueDefault`；不能因为方法同名 `get` 就认为读取语义相同。

这些集成体现了预期分层：

1. 下游 crate 负责 name、领域规则、外层版本和完整请求预算；
2. `qubit-value` 负责 typed payload 校验、标量/集合语义和 V1 payload 编码；
3. `qubit-budget` 与 `qubit-json` 负责共享的有界 JSON 记账和 I/O。

新增公共行为时，需要同时检查两个直接下游，尤其是借用视图、missing 语义、Wire DTO、seed、
preflight 累计行为或 feature gate 发生变化时。

<a id="evolution-rules"></a>
## 演进规则

后续变更应遵循以下规则：

1. 在兼容版本内保持公共方法签名和语义；使用 semver 检查及直接下游测试发现意外破坏。
2. 新增运行时类型时，通过中央 value table 同步更新两个 owned 容器、借用视图、转换行为、
   identity、自然 JSON、Wire 处理、feature 测试和文档。
3. 维持 `Eq`/`Hash` 一致性。即使具体 hash 输出不稳定，语义相等的变化仍属于可观察 API 变化。
4. 不得原地修改 V1 tag、shape 或 payload 表示；不兼容协议变更必须引入新版本。
5. 公共 Wire DTO 继续不实现通用 `Deserialize`；新增 decode 路径必须维持有界完整文档或共享
   session 记账。
6. Preflight 必须保持保守、可累计和单次调用原子性；最终 encoder 继续作为资源准入的事实来源。
7. 自然 JSON 应明确保持有损，并与 Wire 兼容性分离；不得在自然 JSON round trip 中推断运行时
   类型或形态。
8. 默认 feature 集保持最小；可选具体类型要求生产方和消费方明确约定 feature。
9. 默认资源 profile 和 canonical JSON 排序的变化属于兼容性敏感行为，需要 focused test 和
   release note。
10. 契约发生变化时，中英文 README、用户手册和设计文档必须同步保持语义一致。
