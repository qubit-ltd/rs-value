# Qubit Value

[![CircleCI](https://circleci.com/gh/qubit-ltd/rs-value.svg?style=shield)](https://circleci.com/gh/qubit-ltd/rs-value)
[![Coverage Status](https://coveralls.io/repos/github/qubit-ltd/rs-value/badge.svg?branch=main)](https://coveralls.io/github/qubit-ltd/rs-value?branch=main)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Documentation](https://img.shields.io/badge/docs-English-blue.svg)](README.md)

基于 `qubit_common::lang::DataType` 的类型安全值容器框架，提供单值、多值与命名值
的统一抽象，支持泛型构造/获取/设置与类型转换，并完整支持 `serde` 序列化。

## 概述

Qubit Value 提供了以类型安全方式处理动态类型值的综合解决方案。它在静态类型和运行时
灵活性之间架起桥梁，为值的存储、检索和转换提供强大的抽象，同时保持 Rust 的安全保证。

> **配置对象支持**: 如果您需要基于不同类型多值设计的配置对象，建议使用
> [qubit-config](https://github.com/qubit-ltd/rs-config) crate，它提供了完整的
> 配置管理功能。您可以在 [GitHub](https://github.com/qubit-ltd/rs-config) 和
> [crates.io](https://crates.io/crates/qubit-config) 上找到更多信息。

## 特性

### 🎯 **核心设计**
- **枚举抽象**: 使用 `Value`/`MultiValues` 两个枚举覆盖所有支持的数据类型
- **类型安全**: 枚举变体携带静态类型；通过 `Result<T, ValueError>` 表达失败
- **零成本**: 基本类型直接存储；大量 API 返回引用避免拷贝
- **命名值**: `NamedValue`/`NamedMultiValues` 提供名称绑定，便于配置/标识场景
- **serde 支持**: 所有核心类型均实现 `Serialize`/`Deserialize`
- **大数支持**: 支持 `BigInt` 和 `BigDecimal` 类型，满足高精度计算需求
- **扩展类型**: 原生支持 `isize`/`usize`、`Duration`、`Url`、
  `HashMap<String, String>` 和 `serde_json::Value`

### 📦 **核心类型**
- **`Value`**: 单值容器，包含 `Empty(DataType)` 与 28 个变体，覆盖基本类型、字符串、
  日期时间、大数、平台整数、时长、URL、字符串映射和 JSON
- **`MultiValues`**: 多值容器，对应 `Vec<T>` 的枚举变体，含 `Empty(DataType)`
- **`NamedValue`**: 绑定名称的 `Value`，提供 `Deref/DerefMut` 直达内部值
- **`NamedMultiValues`**: 绑定名称的 `MultiValues`，提供 `Deref/DerefMut`，
  并可 `to_named_value()`
- **`ValueError` 与 `ValueResult<T>`**: 标准错误与结果别名

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
qubit-value = "0.4.2"
```

## 使用示例

### 单值操作

```rust
use qubit_value::{Value, ValueError};
use qubit_common::lang::DataType;
use num_bigint::BigInt;
use bigdecimal::BigDecimal;
use std::str::FromStr;

// 泛型构造与类型推断获取
let v = Value::new(8080i32);
let port: i32 = v.get()?;  // 从变量类型推断
assert_eq!(port, 8080);

// 具名获取（返回 Copy 或引用）
assert_eq!(v.get_int32()?, 8080);

// 函数参数中的类型推断
fn check_port(p: i32) -> bool { p > 1024 }
assert!(check_port(v.get()?));  // 从函数签名推断为 i32

// 通过 to<T>() 进行跨类型转换
assert_eq!(v.to::<i64>()?, 8080i64);
assert_eq!(v.to::<String>()?, "8080".to_string());

// 大数类型与类型推断
let big_int = Value::new(BigInt::from(12345678901234567890i64));
let num: BigInt = big_int.get()?;  // 类型推断

// 空值与类型管理
let mut any = Value::Int32(42);
any.clear();
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
any.set("hello")?;
assert_eq!(any.get_string()?, "hello");
```

### 扩展类型

```rust
use qubit_value::Value;
use std::time::Duration;
use url::Url;
use std::collections::HashMap;

// Duration（时长）
let v = Value::new(Duration::from_secs(30));
let d: Duration = v.get()?;
assert_eq!(d, Duration::from_secs(30));
// 字符串格式：<纳秒数>ns
let s: String = v.to()?;
assert_eq!(s, "30000000000ns");
let v2 = Value::String("30000000000ns".to_string());
let d2: Duration = v2.to()?;
assert_eq!(d2, Duration::from_secs(30));

// Url
let url = Url::parse("https://example.com").unwrap();
let v = Value::new(url.clone());
let got: Url = v.get()?;
assert_eq!(got, url);
// 从字符串解析
let v2 = Value::String("https://example.com".to_string());
let got2: Url = v2.to()?;
assert_eq!(got2, url);

// HashMap<String, String>（字符串映射）
let mut map = HashMap::new();
map.insert("host".to_string(), "localhost".to_string());
let v = Value::new(map.clone());
let got: HashMap<String, String> = v.get()?;
assert_eq!(got, map);

// JSON 逃生舱
let j = serde_json::json!({"key": "value"});
let v = Value::from_json_value(j.clone());
let got: serde_json::Value = v.get()?;
assert_eq!(got, j);

// 将任意可序列化类型存为 JSON
#[derive(serde::Serialize, serde::Deserialize)]
struct Config { host: String, port: u16 }
let cfg = Config { host: "localhost".to_string(), port: 8080 };
let v = Value::from_serializable(&cfg)?;
let restored: Config = v.deserialize_json()?;
```

### 多值操作

```rust
use qubit_value::{MultiValues, ValueError};
use qubit_common::lang::DataType;

// 泛型构造
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.count(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// 泛型获取与类型推断（克隆 Vec）
let nums: Vec<i32> = ports.get()?;

// 获取首元素
let first: i32 = ports.get_first()?;
assert_eq!(first, 8080);

// 泛型添加：单个 / Vec / 切片
ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.add(&[8086, 8087][..])?;

// 泛型设置：替换整个列表
ports.set(vec![9001, 9002])?;
assert_eq!(ports.get_int32s()?, &[9001, 9002]);

// 合并（类型需一致）
let mut a = MultiValues::Int32(vec![1, 2]);
let b = MultiValues::Int32(vec![3, 4]);
a.merge(&b)?;
assert_eq!(a.get_int32s()?, &[1, 2, 3, 4]);

// 转为单值（取首元素）
let single = a.to_value();
let first_val: i32 = single.get()?;
assert_eq!(first_val, 1);
```

### 命名值操作

```rust
use qubit_value::{NamedValue, NamedMultiValues, Value, MultiValues};

// 命名单值
let mut nv = NamedValue::new("timeout", Value::new(30i32));
assert_eq!(nv.name(), "timeout");
let timeout: i32 = nv.get()?;
assert_eq!(timeout, 30);

nv.set_name("read_timeout");
nv.set(45i32)?;
assert_eq!(nv.get_int32()?, 45);

// 命名多值
let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
nmv.add(8082)?;
let first_port: i32 = nmv.get_first()?;
assert_eq!(first_port, 8080);

// 命名多值 → 命名单值（取首元素）
let first_named = nmv.to_named_value();
assert_eq!(first_named.name(), "ports");
let val: i32 = first_named.get()?;
assert_eq!(val, 8080);
```

## API 参考

### 泛型 API

#### 构造
- **单值**: `Value::new<T>(t) -> Value`
- **多值**: `MultiValues::new<T>(Vec<T>) -> MultiValues`

`new` 支持的 `T`：`bool`、`char`、`i8`、`i16`、`i32`、`i64`、`i128`、
`u8`、`u16`、`u32`、`u64`、`u128`、`f32`、`f64`、`String`、`&str`、
`NaiveDate`、`NaiveTime`、`NaiveDateTime`、`DateTime<Utc>`、`BigInt`、
`BigDecimal`、`isize`、`usize`、`Duration`、`Url`、
`HashMap<String, String>`、`serde_json::Value`。

#### 获取
- **单值**: `Value::get<T>(&self) -> ValueResult<T>`
- **多值**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **首元素**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`

`get<T>()` 执行**严格类型匹配**——存储的变体必须与 `T` 完全一致。
跨类型转换请使用 `to<T>()`。

#### 设置
- **单值**: `Value::set<T>(&mut self, t) -> ValueResult<()>`
- **多值**:
  - `MultiValues::set<T, S>(&mut self, values: S) -> ValueResult<()>`，
    其中 `S` 可为 `T`、`Vec<T>` 或 `&[T]`
  - `MultiValues::add<T, S>(&mut self, values: S)` 支持 `T`、`Vec<T>` 或 `&[T]`

#### 类型转换
- **`Value::to<T>(&self) -> ValueResult<T>`** — 按 `ValueConverter<T>` 定义的规则
  将当前值转换为 `T`，支持跨类型转换，必要时进行范围检查。

**各目标类型支持的源变体：**

| 目标 `T` | 支持的源变体 |
|---|---|
| `bool` | `Bool`；整数变体（0=false，非零=true）；`String` |
| `i8` | `Int8`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `i16` | `Int16`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `i32` | `Int32`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `i64` | `Int64`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `i128` | `Int128`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `isize` | `IntSize`；`Bool`；`Char`；所有整数变体；`Float32/64`；`String`；`BigInteger/BigDecimal` |
| `u8` | `UInt8`；`Bool`；`Char`；所有整数变体（范围检查）；`String` |
| `u16` | `UInt8/16/32/64/128`；`Bool`；`Char`；有符号整数变体（范围检查）；`String` |
| `u32` | `UInt8/16/32/64/128`；`Bool`；`Char`；有符号整数变体（范围检查）；`String` |
| `u64` | `UInt8/16/32/64/128`；`Bool`；`Char`；有符号整数变体（范围检查）；`String` |
| `u128` | `UInt8/16/32/64/128`；`Bool`；`Char`；有符号整数变体（范围检查）；`String` |
| `usize` | `UIntSize`；`Bool`；`Char`；所有整数变体（范围检查）；`String` |
| `f32` | `Float32/64`；`Bool`；`Char`；所有整数变体；`String`；`BigInteger/BigDecimal` |
| `f64` | `Float64`；`Bool`；`Char`；所有数值变体；`String`；`BigInteger/BigDecimal` |
| `char` | `Char` |
| `String` | 所有变体（整数/浮点/bool/char/日期时间/`Duration`/`Url`/`StringMap`/`Json`） |
| `NaiveDate` | `Date` |
| `NaiveTime` | `Time` |
| `NaiveDateTime` | `DateTime` |
| `DateTime<Utc>` | `Instant` |
| `BigInt` | `BigInteger` |
| `BigDecimal` | `BigDecimal` |
| `Duration` | `Duration`；`String`（格式：`<纳秒数>ns`） |
| `Url` | `Url`；`String` |
| `HashMap<String, String>` | `StringMap` |
| `serde_json::Value` | `Json`；`String`（解析为 JSON）；`StringMap` |

### 具名 API

#### 单值
- **获取器**: `get_xxx()` 方法——`get_bool()`、`get_int32()`、`get_string()`、
  `get_duration()`、`get_url()`、`get_string_map()`、`get_json()` 等
- **设置器**: `set_xxx()` 方法——`set_bool()`、`set_int32()`、`set_string()`、
  `set_duration()`、`set_url()`、`set_string_map()`、`set_json()` 等

#### 多值
- **获取器**: `get_xxxs()` 方法——`get_int32s()`、`get_strings()`、
  `get_durations()`、`get_urls()`、`get_string_maps()`、`get_jsons()` 等
- **设置器**: `set_xxxs()` 方法——`set_int32s()`、`set_strings()` 等
- **添加器**: `add_xxx()` 方法——`add_int32()`、`add_string()`、
  `add_duration()`、`add_url()` 等
- **切片操作**: `*_slice` 变体——`set_int32s_slice()`、`add_strings_slice()` 等

### JSON 工具方法（`Value` 上）
- `Value::from_json_value(serde_json::Value) -> Value`
- `Value::from_serializable<T: Serialize>(value: &T) -> ValueResult<Value>`
- `Value::deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T>`

### 工具方法

#### 单值
- `data_type()` — 获取数据类型
- `is_empty()` — 检查是否为空
- `clear()` — 清空值（保留类型）
- `set_type()` — 更改类型

#### 多值
- `count()` — 获取元素数量
- `is_empty()` — 检查是否为空
- `clear()` — 清空所有值（保留类型）
- `set_type()` — 更改类型
- `merge()` — 与另一个多值合并（类型需一致）
- `to_value()` — 转换为单值（取首元素）

## 错误类型

```rust
use qubit_value::{ValueError, ValueResult};
use qubit_common::lang::DataType;

// 主要错误变体
ValueError::NoValue                           // 访问了空值
ValueError::TypeMismatch { expected, actual } // get<T>() 类型不匹配
ValueError::ConversionFailed { from, to }     // to<T>() 不支持的转换方向
ValueError::ConversionError(String)           // to<T>() 范围检查或解析失败
ValueError::IndexOutOfBounds { index, len }   // 多值索引越界
ValueError::JsonSerializationError(String)    // JSON 序列化失败
ValueError::JsonDeserializationError(String)  // JSON 反序列化失败
```

所有可能失败的操作均返回 `ValueResult<T> = Result<T, ValueError>`。

## 支持的数据类型

### 基本标量类型
- **有符号整数**: `i8`, `i16`, `i32`, `i64`, `i128`
- **无符号整数**: `u8`, `u16`, `u32`, `u64`, `u128`
- **平台整数**: `isize`, `usize`
- **浮点数**: `f32`, `f64`
- **其他**: `bool`, `char`

### 字符串
- `String`（直接存储）

### 日期/时间类型
- `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`（通过 `chrono`）

### 大数类型
- `BigInt`, `BigDecimal`（通过 `num-bigint` 和 `bigdecimal`）

### 扩展类型
- **`isize` / `usize`**: 平台相关整数
- **`Duration`**: `std::time::Duration`；字符串表示为 `<纳秒数>ns`
- **`Url`**: `url::Url`；字符串表示为 URL 文本
- **`HashMap<String, String>`**: 字符串映射；字符串表示为 JSON
- **`serde_json::Value`**: 用于复杂/自定义类型的 JSON 逃生舱

## 序列化支持

所有类型均实现 `Serialize`/`Deserialize`：
- `Value`、`MultiValues`、`NamedValue`、`NamedMultiValues`

序列化时保持完整的类型信息，反序列化时进行类型验证。

## 性能说明

- **引用返回**: `get_string()` 返回 `&str` 避免克隆
- **借用支持**: `Value::new()` 和 `set()` 接受 `&str`（自动转换为 `String`）
- **智能分发**: `MultiValues::set/add` 接受 `T`、`Vec<T>` 或 `&[T]`，
  内部自动分派最优路径

## 依赖项

```toml
[dependencies]
qubit-common = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
url = { version = "2.5", features = ["serde"] }
num-traits = "0.2"
num-bigint = { version = "0.4", features = ["serde"] }
bigdecimal = { version = "0.4", features = ["serde"] }
```

## 测试与代码覆盖率

本项目保持全面的测试覆盖，对所有功能进行详细验证。有关覆盖率报告和测试说明，
请参阅 [COVERAGE.zh_CN.md](COVERAGE.zh_CN.md) 文档。

## 许可证

Copyright (c) 2025 - 2026 Haixing Hu, Qubit Co. Ltd. All rights reserved.

根据 Apache 许可证 2.0 版（"许可证"）授权；
除非遵守许可证，否则您不得使用此文件。
您可以在以下位置获取许可证副本：

    http://www.apache.org/licenses/LICENSE-2.0

除非适用法律要求或书面同意，否则根据许可证分发的软件
按"原样"分发，不附带任何明示或暗示的担保或条件。
有关许可证下的特定语言管理权限和限制，请参阅许可证。

完整的许可证文本请参阅 [LICENSE](LICENSE)。

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 作者

**胡海星** - *Qubit Co. Ltd.*

---

有关 Qubit 开源项目的更多信息，请访问我们的
[GitHub 组织](https://github.com/qubit-ltd)。
