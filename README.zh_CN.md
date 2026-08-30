# Qubit Value

[![Rust CI](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-value/coverage-badge.json)](https://qubit-ltd.github.io/rs-value/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Document](https://img.shields.io/badge/Document-English-blue.svg)](README.md)

`qubit-value` 为 Rust 应用提供统一的、类型安全的运行时值边界。当配置、元数据、
协议字段或用户输入的类型只能在运行时确定，但应用仍需要明确类型、受控转换和可预期
错误时，可以使用这个 crate。

## 它解决什么问题

如果没有统一的运行时值模型，每个 key-value 子系统通常都会重新定义自己的 `enum`、
转换规则、未设置语义和序列化格式，容易产生三个问题：

- 未设置值、明确为空的集合和 JSON `null` 被混为一谈；
- 一个元素的集合被意外当成标量；
- 值跨进程或存储边界传输时丢失运行时类型，或者接受了并未明确允许的转换。

`Value` 保存一个带类型的标量，`MultiValues` 保存一个同类型集合，
`ValueContainer` 显式保留标量或集合形态。`Unset(DataType)` 保留声明的类型，但明确表示
当前没有具体值。

## 快速开始：一个小型运行时配置 map

下面用一个简化版配置 map 展示典型流程：每个 key 保存一个不同的 `Value`，读取时根据场景
选择严格读取、显式转换或带类型的默认值。下面的代码假定位于一个返回兼容 `Result` 的函数
中，因此使用 `?` 传播值读取和转换错误。

```rust
use std::collections::HashMap;
use std::time::Duration;

use qubit_datatype::DataType;
use qubit_value::Value;

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
```

如果需要完整的通用配置对象，而不是自己组装 map，可以使用来自
[`rs-config`](https://github.com/qubit-ltd/rs-config) 的 `Config`。它基于 `Value` 构建，并在此
之上提供属性管理、类型化读取、多值读取、默认值、配置 section、转换策略、插值，以及可插拔的
文件/环境变量配置源等更全面的高级能力。

`get()` 是严格类型读取，不会静默转换。`to()` 使用 `qubit-datatype` 提供的共享转换规则；
转换失败仍会返回错误。`to()` 和 `to_or()` 需要启用 `converter` feature；`get_or()` 只为
未设置值提供 fallback，不执行转换。

需要显式策略和限制时使用 `to_with`。每次 `to_with` 调用都会创建全新的
`ConversionSession`，因此彼此独立的读取不会共享累计消耗：

```rust
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_value::Value;

let policy = ConversionPolicy::env_friendly();
let limits = ConversionLimits::default();
let first = Value::new(" 8080 ").to_with::<u16>(&policy, &limits)?;
let second = Value::new(" 8081 ").to_with::<u16>(&policy, &limits)?;
assert_eq!((first, second), (8080, 8081));
```

## 安装

在 `Cargo.toml` 中加入核心 crate 和类型定义 crate：

```toml
[dependencies]
qubit-value = { version = "0.10", features = ["converter"] }
qubit-datatype = { version = "0.11", default-features = false }
```

快速开始使用了 `Value::to`，因此这里启用 `converter`。默认 feature 集为空，请只启用实际
使用的类型族：

| Feature | 额外的 `DataType` 或能力 |
| --- | --- |
| `converter` | `Value::to` 等跨类型转换 API |
| `chrono` | `Date`、`Time`、`DateTime` 和 `Instant` |
| `big-integer` | 由 `num_bigint::BigInt` 支持的 `BigInteger` |
| `big-decimal` | 由 `bigdecimal::BigDecimal` 支持的 `BigDecimal` |
| `big-number` | 同时启用两个大数 feature 的兼容别名 |
| `url` | 由 `url::Url` 支持的 `Url` |
| `json` | 由 `serde_json::Value` 支持的 `Json`，以及有界版本化 JSON Wire 编解码 |
| `natural-json` | 通过 `Value::to_json_value` 投影自然 JSON；同时启用 `converter` 和 `json` |
| `redact` | 通过 `qubit-redact` 提供按策略脱敏的视图 |
| `all` | `converter`、`chrono`、`big-number`、`url`、`json`、`natural-json` 和 `redact` |

## 提供的能力

- `Value` 和 `MultiValues` 提供类型化构造函数、类型化 getter、泛型修改、借用读取和明确
  的 unset 状态。
- `ValueContainer::Scalar` 和 `ValueContainer::Collection` 保留形态；单元素集合仍然是集合。
- `get_or`/`to_or` 及其集合版本让 fallback 语义保持明确：未设置值可以使用默认值，类型不
  匹配和普通转换失败仍会报告错误。
- `NamedValue` 和 `NamedMultiValues` 为运行时值附加 key，不改变值本身的类型语义。
- `ValueWireV1` 提供带版本的类型保留 JSON 表示，并提供有界的 `to_json_vec()`、
  `to_json_writer()` 入口；定向 `_with_limits` 方法分别接收 `JsonDecodeLimits` 和
  `JsonEncodeLimits`。解码通过调用方配置的 `JsonDecodeSession` 完成准入；编码通过
  `JsonEncodeSession` 在线限制结构与输出字节。当接收方必须恢复精确的 `DataType`
  和形态时，应使用 Wire V1。
- 自然 JSON 工具方法生成普通的 `null`、标量、对象和数组；当边界只需要 JSON 语义时使用它。
- 当前运行时类型词汇包含 25 个 `DataType` 变体。具体的 feature-gated 值需要启用对应
  feature，但未设置状态仍可保留其类型声明。

这个 crate 不提供完整的配置存储、schema registry、文件格式或分布式缓存；它只提供这些系统
可以复用的类型化值层。`Eq`/`Hash` 适合进程内 Rust 集合，不能作为持久化指纹或分布式缓存
key。完整类型表、JSON 数字契约、错误、资源限制和 feature 兼容性请参阅用户手册。

## 基于 `Value` 构建的容器

两个兄弟 crate 直接利用这个值模型实现 key-value 容器：

- [`rs-config`](https://github.com/qubit-ltd/rs-config) 提供类型化配置属性、面向配置文件和
  环境变量的访问，以及按策略读取的能力。应用配置应优先考虑它。
- [`rs-metadata`](https://github.com/qubit-ltd/rs-metadata) 提供类型化元数据/属性存储和过滤。
  资源、记录或可检索的应用元数据适合使用它。

## 选择 Wire V1 还是自然 JSON

当接收方必须区分 `Int32(42)` 和 `String("42")`、保留标量与集合形态，或保留
`Unset(DataType)` 时，选择 Wire V1。典型文档如下：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

当边界只是普通业务 JSON，接收方只需要 JSON 语义时，选择 `to_json_value()`。Wire V1 是封闭的、
带版本的格式；自然 JSON 刻意不包含运行时类型标签。完整的 Wire 工作流、借用 payload、feature
兼容性和资源限制处理，请参阅用户手册。

自然 JSON 无法恢复 `DataType`、unset 状态或标量/集合形态。Wire V1 会拒绝非有限浮点、
不支持的类型和非法 payload，不会猜测输入含义。每个独立 Wire 操作应创建新的有界 session；
只有多个嵌入值属于同一个外层请求预算时，才应复用 session。

## 延伸阅读

- [中文用户手册](doc/user_guide.zh_CN.md)
- [English user guide](doc/user_guide.md)
- [API 文档](https://docs.rs/qubit-value)
- [`qubit-datatype` 转换契约](https://docs.rs/qubit-datatype/latest/qubit_datatype/)
- [English README](README.md)

## 测试

```bash
# 使用默认 feature 集运行测试
cargo test

# 使用项目声明的全部 feature 运行测试
cargo test --all-features

# 运行项目 CI 检查
./ci-check.sh

# 检查代码覆盖率
./coverage.sh
```

## 许可证

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

本项目基于 Apache License 2.0 授权。完整许可证文本请参阅
[LICENSE](LICENSE)。

## 贡献

欢迎贡献。请遵循 Rust API 指南，及时更新公共 API 文档与测试，并在提交
Pull Request 前运行 `./align-ci.sh`格式化代码，运行`./ci-check.sh`对齐CI要求。

## 作者

**Haixing Hu** - *Qubit Co. Ltd.*

仓库地址：[https://github.com/qubit-ltd/rs-value](https://github.com/qubit-ltd/rs-value)
