# Qubit Value

[![Rust CI](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-value/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-value/coverage-badge.json)](https://qubit-ltd.github.io/rs-value/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-value.svg?color=blue)](https://crates.io/crates/qubit-value)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Documentation](https://img.shields.io/badge/docs-English-blue.svg)](README.md)

基于 `qubit_datatype::DataType` 的类型安全值容器框架，提供单值、多值与命名值
的统一抽象，支持严格访问、泛型修改、可配置转换与带类型标签的 Serde 表示。

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
- **借用访问**: 存储类型不是 `Copy` 时，类型化 getter 返回引用
- **命名值**: `NamedValue`/`NamedMultiValues` 提供名称绑定，便于配置/标识场景
- **两类 JSON 边界**: 带类型标签的 Serde 保留数据类型；自然 JSON 投影生成普通的
  `null`、标量、对象与数组
- **便捷默认值**: `get_or`、`to_or` 和列表默认值 API 支持标量默认值、
  借用字符串字面量、数组、切片、vector 和借用的 vector
- **灵活集合输入**: `MultiValues::new/set/add` 支持直接数组、切片、vector、
  借用的 vector 和借用字符串集合
- **大数支持**: 可选的 `BigInt` 和 `BigDecimal` 变体
- **扩展类型**: 原生支持 `isize`/`usize`、`Duration`、`Url`、
  `HashMap<String, String>` 和 `serde_json::Value`

### 📦 **核心类型**
- **`Value`**: 单值容器，包含 `Empty(DataType)` 与 27 个具体变体，覆盖基本类型、字符串、
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
qubit-value = { version = "0.8", features = ["all"] }
```

默认 feature 集为空。可按需启用类型族，也可使用 `all` 这一便捷 feature：

| Feature | 启用内容 |
|---|---|
| `chrono` | 日期、时间、日期时间与 UTC 时刻变体 |
| `big-number` | `BigInt` 与 `BigDecimal` 变体 |
| `url` | URL 变体 |
| `json` | `serde_json::Value` 变体 |
| `converter` | 所有扩展变体、转换 API 与自然 JSON API |
| `all` | 当前支持的全部能力（即 `converter`） |

## 使用示例

### 单值操作

```rust
use qubit_value::{Value, ValueError};
use qubit_datatype::{DataConversionError, DataListConversionError, DataType};
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
assert!(any.is_unset());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
any.set("hello");
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
// 默认字符串转换使用毫秒单位
let s: String = v.to()?;
assert_eq!(s, "30000ms");
let v2 = Value::String("30s".to_string());
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
use qubit_datatype::DataType;

// 从 Vec<T> 泛型构造
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.count(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// 可以直接传数组、切片、vector 和借用的 vector
let array_ports = MultiValues::new([8080i32, 8081, 8082]);
let more_ports = [9000i32, 9001];
let borrowed = MultiValues::new(more_ports.as_slice());
let owned = vec![7000i32, 7001];
let borrowed_vec = MultiValues::new(&owned);

// 字符串列表可以直接从 &str 集合构造
let servers = MultiValues::new(["api", "worker", "cache"]);
assert_eq!(servers.get_strings()?, &["api", "worker", "cache"]);

// 泛型获取与类型推断（克隆 Vec）
let nums: Vec<i32> = ports.get()?;

// 获取首元素
let first: i32 = ports.get_first()?;
assert_eq!(first, 8080);

// 泛型添加：单个 / Vec / 切片
ports.add(8083)?;
ports.add(vec![8084, 8085])?;
ports.add(&[8086, 8087][..])?;
ports.add([8088, 8089])?;

// 泛型设置：替换整个列表
ports.set(vec![9001, 9002]);
ports.set([9100, 9101]);
ports.set(&owned);
assert_eq!(ports.get_int32s()?, &[7000, 7001]);

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

### 带默认值的读取与转换

带默认值的 API 只会在容器未设置时使用 fallback。已经设置的空集合仍保持为空；
类型不匹配和转换失败会正常返回错误，不会被默认值掩盖。

```rust
use qubit_datatype::DataType;
use qubit_value::{MultiValues, Value};

// 严格类型读取，并在空值时使用默认值
let value = Value::Empty(DataType::String);
let host: String = value.get_or("localhost")?;
assert_eq!(host, "localhost");

let value = Value::String("8080".to_string());
let port: u16 = value.to_or(9000u16)?;
assert_eq!(port, 8080);

// 多值严格读取，并使用集合默认值
let values = MultiValues::Empty(DataType::String);
let paths: Vec<String> = values.get_or(["cache", "tmp"])?;
assert_eq!(paths, vec!["cache".to_string(), "tmp".to_string()]);

// 首元素转换，并使用标量默认值
let values = MultiValues::Empty(DataType::UInt16);
let port: u16 = values.to_or(8080u16)?;
assert_eq!(port, 8080);

// 列表转换，并使用数组或切片默认值
let values = MultiValues::Empty(DataType::String);
let tags: Vec<String> = values.to_list_or(["blue", "green"])?;
assert_eq!(tags, vec!["blue".to_string(), "green".to_string()]);
```

### 集合参数写法

集合类 API 接受调用处最常见的便捷写法。这适用于 `MultiValues::new`、
`MultiValues::set`、`MultiValues::add`，也适用于 `get_or`、`to_list_or`
等带列表默认值的读取接口。

```rust
use qubit_datatype::DataType;
use qubit_value::MultiValues;

let array_values = MultiValues::new([1i32, 2, 3]);
let slice_source = [4i32, 5, 6];
let slice_values = MultiValues::new(slice_source.as_slice());
let vec_source = vec![7i32, 8, 9];
let vec_values = MultiValues::new(vec_source.clone());
let borrowed_vec_values = MultiValues::new(&vec_source);

let mut values = MultiValues::Empty(DataType::Int32);
values.set([10, 11, 12]);
values.add(slice_source.as_slice())?;
values.add(&vec_source)?;

let strings = MultiValues::new(["api", "worker"]);
let fallback: Vec<String> = MultiValues::Empty(DataType::String)
    .get_or(["cache", "tmp"])?;
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
nv.set(45i32);
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
- **多值**: `MultiValues::new<S>(values) -> MultiValues`

`MultiValues::new` 支持 `Vec<T>`、`&Vec<T>`、`&[T]`、`[T; N]` 和
`&[T; N]`。字符串值还支持 `Vec<&str>`、`&Vec<&str>`、`&[&str]`、
`[&str; N]` 和 `&[&str; N]`，内部会生成 `Vec<String>`。

`new` 支持的 `T`：`bool`、`char`、`i8`、`i16`、`i32`、`i64`、`i128`、
`u8`、`u16`、`u32`、`u64`、`u128`、`f32`、`f64`、`String`、`&str`、
`NaiveDate`、`NaiveTime`、`NaiveDateTime`、`DateTime<Utc>`、`BigInt`、
`BigDecimal`、`isize`、`usize`、`Duration`、`Url`、
`HashMap<String, String>`、`serde_json::Value`。

#### 获取
- **单值**: `Value::get<T>(&self) -> ValueResult<T>`
- **带默认值的单值读取**: `Value::get_or<T>(&self, default) -> ValueResult<T>`
- **多值**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **带默认值的多值读取**: `MultiValues::get_or<T>(&self, default) -> ValueResult<Vec<T>>`
- **首元素**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`
- **带默认值的首元素读取**: `MultiValues::get_first_or<T>(&self, default) -> ValueResult<T>`

`get<T>()` 执行**严格类型匹配**——存储的变体必须与 `T` 完全一致。
跨类型转换请使用 `to<T>()`。

#### 设置
- **单值**: `Value::set<T: Into<Value>>(&mut self, value) -> ()`
- **多值**:
  - `MultiValues::set<S: Into<MultiValues>>(&mut self, values) -> ()`
    替换整个集合，并可改变元素类型
  - `MultiValues::add<S: Into<MultiValues>>(&mut self, values) -> ValueResult<()>`
    只在元素类型一致时追加
  - 两者支持相应 `Into<MultiValues>` 实现覆盖的标量、`Vec<T>`、`&Vec<T>`、
    `&[T]`、`[T; N]` 和 `&[T; N]`
  - 字符串集合还支持 `Vec<&str>`、`&Vec<&str>`、`&[&str]`、`[&str; N]`
    和 `&[&str; N]`

#### 类型转换
- **`Value::to<T>(&self) -> ValueResult<T>`** — 按共享转换规则将当前值转换为
  `T`，支持跨类型转换，必要时进行范围检查。
- **`Value::to_or<T>(&self, default) -> ValueResult<T>`** — 转换为 `T`，
  如果值未设置则返回默认值。
- **`Value::to_or_with<T>(&self, default, options) -> ValueResult<T>`** —
  使用显式转换选项，并保持相同的默认值语义。
- **`MultiValues::to<T>(&self) -> ValueResult<T>`** — 转换第一个存储值。
- **`MultiValues::to_or<T>(&self, default) -> ValueResult<T>`** —
  转换第一个存储值，如果没有值则返回默认值。
- **`MultiValues::to_or_with<T>(&self, default, options) -> ValueResult<T>`** —
  使用显式转换选项，并保持相同的默认值语义。
- **`MultiValues::to_list<T>(&self) -> ValueResult<Vec<T>>`** —
  转换所有存储值。
- **`MultiValues::to_list_with<T>(&self, options) -> ValueResult<Vec<T>>`** —
  使用显式转换选项转换所有存储值。
- **`MultiValues::to_list_or<T>(&self, default) -> ValueResult<Vec<T>>`** —
  转换所有存储值，仅在容器未设置时返回默认值；具体的空 vector 仍保持为空。
- **`MultiValues::to_list_or_with<T>(&self, default, options) -> ValueResult<Vec<T>>`** —
  使用显式转换选项，并保持相同的列表默认值语义。

**各目标类型支持的源变体：**

| 目标 `T` | 支持的源变体 |
|---|---|
| `bool` | `Bool`；整数变体（0=false，非零=true）；`String` 值 `1`、`0`、`true`、`false`（`true`/`false` 忽略大小写） |
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
| `Duration` | `Duration`；整数变体和 `BigInteger`（使用配置的时长单位）；`String`（可带 `ns`/`us`/`ms`/`s`/`m`/`h`/`d` 后缀；无后缀时使用配置的时长单位） |
| `Url` | `Url`；`String` |
| `HashMap<String, String>` | `StringMap` |
| `serde_json::Value` | `Json`；`String`（解析为 JSON）；`StringMap` |

### 类型化与具名 API

#### 单值
- **获取器**: `get_xxx()` 方法——`get_bool()`、`get_int32()`、`get_string()`、
  `get_duration()`、`get_url()`、`get_string_map()`、`get_json()` 等
- **修改**: 使用泛型 `set()`。类型化 setter 已删除，因为它们没有提供泛型 API
  之外的行为。

#### 多值
- **获取器**: `get_xxxs()` 方法——`get_int32s()`、`get_strings()`、
  `get_durations()`、`get_urls()`、`get_string_maps()`、`get_jsons()` 等
- **修改**: 使用泛型 `set()` 和 `add()`，支持标量、拥有所有权的集合、数组、
  切片与借用 vector 输入。

### JSON 工具方法
- `Value::from_json_value(serde_json::Value) -> Value`
- `Value::from_serializable<T: Serialize>(value: &T) -> ValueResult<Value>`
- `Value::deserialize_json<T: DeserializeOwned>(&self) -> ValueResult<T>`
- `Value::to_json_value(&self) -> ValueResult<serde_json::Value>`
- `MultiValues::to_json_value(&self) -> ValueResult<serde_json::Value>`

带类型标签的 Serde 与自然 JSON 是两个独立契约。前者保留变体名称；后者把未设置值
映射为 `null`，具体空集合映射为 `[]`，单元素集合映射为标量或对象，多元素集合映射为
数组。自然 JSON 用字符串表示 128 位整数和大数。内存中可以保存非有限浮点数，但两个
JSON 边界都会拒绝 `NaN`、正无穷和负无穷，因为 JSON 没有这些数值字面量。

### 工具方法

#### 单值
- `data_type()` — 获取数据类型
- `is_unset()` — 检查是否没有存储具体值
- `is_numeric()` — 判断具体值是否为数值类型
- `unset()` / `clear()` — 移除值并保留声明类型
- `set_type()` — 更改类型

#### 多值
- `count()` — 获取元素数量
- `is_unset()` — 区分未设置状态与具体空 vector
- `is_numeric()` — 判断具体集合是否为数值类型
- `unset()` — 移除具体 vector 并保留声明类型
- `clear()` — 清空具体 vector 并保持具体状态；未设置值仍保持未设置
- `set_type()` — 更改类型
- `merge()` — 与另一个多值合并（类型需一致）
- `to_value()` — 转换为单值（取首元素）

## 错误类型

```rust
use qubit_value::{ValueError, ValueResult};
use qubit_datatype::DataType;

// 主要错误变体
ValueError::NoValue                           // 访问了空值
ValueError::TypeMismatch { expected, actual } // get<T>() 类型不匹配
ValueError::DataConversion(DataConversionError) // 结构化的 to<T>() 错误
ValueError::DataListConversion(DataListConversionError) // 含原始索引的列表错误
```

所有可能失败的操作均返回 `ValueResult<T> = Result<T, ValueError>`。
转换错误会保留共享转换层的结构化 source；列表错误还会保留原始
`source_index`。`to()` 默认使用精确数值转换；确实需要截断或舍入时，应通过
`to_with()` 指定 `NumericConversionPolicy::Lossy`。除非在
`StringConversionOptions` 中显式开启，否则文本不会自动 trim。

## 支持的数据类型

### 基本标量类型
- **有符号整数**: `i8`, `i16`, `i32`, `i64`, `i128`
- **无符号整数**: `u8`, `u16`, `u32`, `u64`, `u128`
- **平台整数**: `isize`, `usize`（`IntSize`/`UIntSize`）；其范围依赖目标架构，
  不适合作为可移植的持久化类型
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
- **`Duration`**: `std::time::Duration`；字符串转换使用配置的时长单位，
  默认是毫秒，例如 `1500ms`。解析时支持 `ns`、`us`、`ms`、`s`、`m`、
  `h` 和 `d` 后缀；无后缀字符串使用配置的时长单位解析。
- **`Url`**: `url::Url`；字符串表示为 URL 文本
- **`HashMap<String, String>`**: 字符串映射；字符串表示为 JSON
- **`serde_json::Value`**: 用于复杂/自定义类型的 JSON 逃生舱

## 序列化契约

启用的类型均实现 `Serialize`/`Deserialize`：
- `Value`、`MultiValues`、`NamedValue`、`NamedMultiValues`

带类型标签的序列化保留变体。`Int128` 与 `UInt128` 的标签 payload 使用十进制
字符串，使其能合法、无损地通过 JSON 和 Serde 的缓冲枚举表示。浮点 payload 必须
是有限值。

## 性能说明

- **引用返回**: `get_string()` 返回 `&str` 避免克隆
- **借用支持**: `Value::new()` 和 `set()` 接受 `&str`（自动转换为 `String`）
- **灵活输入**: `MultiValues::new/set/add` 接受直接数组、切片、vector
  和借用的 vector，支持所有已实现的元素类型
- **借用默认值**: 带默认值的读取接口可以直接使用借用字符串字面量和借用集合，
  调用方无需提前分配 owned fallback

## 依赖项

```toml
[dependencies]
qubit-datatype = { version = "0.5", default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
url = { version = "2.5", features = ["serde"] }
num-bigint = { version = "0.4", features = ["serde"] }
bigdecimal = { version = "0.4", features = ["serde"] }
```

## 测试

本项目保持全面的测试覆盖，对所有功能进行详细验证。发布或提交前请运行
`./ci-check.sh`。

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
