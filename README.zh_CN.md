# Prism3 Value

[![CircleCI](https://circleci.com/gh/3-prism/prism3-rust-value.svg?style=shield)](https://circleci.com/gh/3-prism/prism3-rust-value)
[![Coverage Status](https://coveralls.io/repos/github/3-prism/prism3-rust-value/badge.svg?branch=main)](https://coveralls.io/github/3-prism/prism3-rust-value?branch=main)
[![Crates.io](https://img.shields.io/crates/v/prism3-value.svg?color=blue)](https://crates.io/crates/prism3-value)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![English Documentation](https://img.shields.io/badge/docs-English-blue.svg)](README.md)

基于 `prism3_core::lang::DataType` 的类型安全值容器框架，提供单值、多值与命名值的统一抽象，支持泛型构造/获取/设置与类型转换，并完整支持 `serde` 序列化。

[English Documentation](README.md)

## 概述

Prism3 Value 提供了以类型安全方式处理动态类型值的综合解决方案。它在静态类型和运行时灵活性之间架起桥梁，为值的存储、检索和转换提供强大的抽象，同时保持 Rust 的安全保证。

## 特性

### 🎯 **核心设计**
- **枚举抽象**: 使用 `Value`/`MultiValues` 两个枚举覆盖原 Java 版接口+实现的层级
- **类型安全**: 枚举变体携带静态类型；通过 `Result<T, ValueError>` 表达失败
- **零成本**: 基本类型直接存储；大量 API 返回引用避免拷贝
- **命名值**: `NamedValue`/`NamedMultiValues` 提供名称绑定，便于配置/标识场景
- **serde 支持**: 所有核心类型均 `Serialize`/`Deserialize`
- **大数支持**: 支持 `BigInt` 和 `BigDecimal` 类型，满足高精度计算需求

### 📦 **核心类型**
- **`Value`**: 单值容器，包含 `Empty(DataType)` 与基本/字符串/字节/日期时间/大数等 20+ 变体
- **`MultiValues`**: 多值容器，对应 `Vec<T>` 的枚举变体，含 `Empty(DataType)`
- **`NamedValue`**: 绑定名称的 `Value`，提供 `Deref/DerefMut` 直达内部值
- **`NamedMultiValues`**: 绑定名称的 `MultiValues`，提供 `Deref/DerefMut`，并可 `to_named_value()`
- **`ValueError` 与 `ValueResult<T>`**: 标准错误与结果别名

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
prism3-value = "0.1.0"
```

## 使用示例

### 单值操作

```rust
use prism3_value::{Value, ValueError};
use prism3_core::lang::DataType;
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

// 跨类型转换
assert_eq!(v.as_int64()?, 8080i64);
assert_eq!(v.as_string()?, "8080".to_string());

// 大数类型与类型推断
let big_int = Value::new(BigInt::from(12345678901234567890i64));
let num: &BigInt = big_int.get()?;  // 类型推断
assert_eq!(num, &BigInt::from(12345678901234567890i64));

let big_decimal = Value::new(BigDecimal::from_str("123.456")?);
let decimal: &BigDecimal = big_decimal.get()?;  // 类型推断
assert_eq!(decimal, &BigDecimal::from_str("123.456")?);

// 空值与类型管理
let mut any = Value::Int32(42);
any.clear();
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::Int32);
any.set_type(DataType::String);
assert!(any.is_empty());
assert_eq!(any.data_type(), DataType::String);

// 泛型设置与类型推断（分派到具名 setter）
any.set("hello")?; // &str -> String，从参数推断类型
assert_eq!(any.get_string()?, "hello");
```

### 多值操作

```rust
use prism3_value::{MultiValues, ValueError};
use prism3_core::lang::DataType;
use num_bigint::BigInt;
use bigdecimal::BigDecimal;

// 泛型构造
let mut ports = MultiValues::new(vec![8080i32, 8081, 8082]);
assert_eq!(ports.count(), 3);
assert_eq!(ports.get_int32s()?, &[8080, 8081, 8082]);

// 泛型获取与类型推断（克隆 Vec）
let nums: Vec<i32> = ports.get()?;  // 从变量类型推断
assert_eq!(nums, vec![8080, 8081, 8082]);

// 在迭代中的类型推断
for port in ports.get::<i32>()? {  // 返回 Vec<i32>
    println!("端口: {}", port);
}

// 获取首元素与类型推断
let first: i32 = ports.get_first()?;  // 从变量类型推断
assert_eq!(first, 8080);

// 泛型添加与类型推断：单个/Vec/切片
ports.add(8083)?;                    // 从参数推断类型
ports.add(vec![8084, 8085])?;        // 从 Vec<i32> 推断
ports.add(&[8086, 8087][..])?;       // 从切片推断
assert_eq!(ports.get_int32s()?, &[8080,8081,8082,8083,8084,8085,8086,8087]);

// 泛型设置与类型推断：单个/Vec/切片，替换整个列表
ports.set(9000)?;                    // 从参数推断类型
ports.set(vec![9001, 9002])?;        // 从 Vec<i32> 推断
ports.set(&[9003, 9004][..])?;       // 从切片推断
assert_eq!(ports.get_int32s()?, &[9003, 9004]);

// 大数多值与类型推断
let mut big_nums = MultiValues::new(vec![
    BigInt::from(123456789),
    BigInt::from(987654321)
]);
assert_eq!(big_nums.count(), 2);
big_nums.add(BigInt::from(111111111))?;  // 从参数推断类型
let nums_vec: Vec<BigInt> = big_nums.get()?;  // 类型推断
assert_eq!(nums_vec.len(), 3);

// 合并（类型需一致）
let mut a = MultiValues::Int32(vec![1,2]);
let b = MultiValues::Int32(vec![3,4]);
a.merge(&b)?;
assert_eq!(a.get_int32s()?, &[1,2,3,4]);

// 转为单值（取首元素）
let single = a.to_value();
let first_val: i32 = single.get()?;  // 类型推断
assert_eq!(first_val, 1);
```

### 命名值操作

```rust
use prism3_value::{NamedValue, NamedMultiValues, Value, MultiValues};
use num_bigint::BigInt;

// 命名单值与类型推断
let mut nv = NamedValue::new("timeout", Value::new(30i32));
assert_eq!(nv.name(), "timeout");
let timeout: i32 = nv.get()?;  // 从变量类型推断
assert_eq!(timeout, 30);

// 泛型设置与类型推断
nv.set_name("read_timeout");
nv.set(45i32)?;  // 从参数推断类型
assert_eq!(nv.get_int32()?, 45);

// 命名多值与类型推断
let mut nmv = NamedMultiValues::new("ports", MultiValues::new(vec![8080i32, 8081]));
assert_eq!(nmv.name(), "ports");
nmv.add(8082)?;  // 从现有值推断类型
let first_port: i32 = nmv.get_first()?;  // 类型推断
assert_eq!(first_port, 8080);

// 命名多值 → 命名单值（取首元素）
let first_named = nmv.to_named_value();
assert_eq!(first_named.name(), "ports");
let val: i32 = first_named.get()?;  // 类型推断
assert_eq!(val, 8080);

// 大数命名值与类型推断
let big_named = NamedValue::new("big_number", Value::new(BigInt::from(12345678901234567890i64)));
assert_eq!(big_named.name(), "big_number");
let big_num: &BigInt = big_named.get()?;  // 类型推断
assert_eq!(big_num, &BigInt::from(12345678901234567890i64));
```

## API 参考

### 泛型 API

#### 构造
- **单值**: `Value::new<T>(t) -> Value`
- **多值**: `MultiValues::new<T>(Vec<T>) -> MultiValues`

#### 获取
- **单值**: `Value::get<T>(&self) -> ValueResult<T>`
- **多值**: `MultiValues::get<T>(&self) -> ValueResult<Vec<T>>`
- **首元素**: `MultiValues::get_first<T>(&self) -> ValueResult<T>`

#### 设置
- **单值**: `Value::set<T>(&mut self, t) -> ValueResult<()>`
- **多值**:
  - `MultiValues::set<'a, T, S>(&mut self, values: S) -> ValueResult<()>`，其中 `S` 可为 `T`、`Vec<T>` 或 `&[T]`
  - `MultiValues::add<'a, T, S>(&mut self, values: S)` 支持 `T`、`Vec<T>` 或 `&[T]`

### 具名 API

#### 单值
- **获取器**: `get_xxx()` 方法，如 `get_int32()`、`get_string()` 等
- **设置器**: `set_xxx()` 方法，如 `set_int32()`、`set_string()` 等

#### 多值
- **获取器**: `get_xxxs()` 方法，如 `get_int32s()`、`get_strings()` 等
- **设置器**: `set_xxxs()` 方法，如 `set_int32s()`、`set_strings()` 等
- **添加器**: `add_xxx()` 方法，如 `add_int32()`、`add_string()` 等
- **切片操作**: `*_slice` 方法，如 `set_int32s_slice()`、`add_strings_slice()` 等

### 类型转换

- **跨类型转换**: `Value::as_bool()`、`as_int32()`、`as_int64()`、`as_float64()`、`as_string()` 等
- **多值转单值**: `MultiValues::to_value()` 获取第一个值

### 工具方法

#### 单值
- `data_type()` - 获取数据类型
- `is_empty()` - 检查是否为空
- `clear()` - 清空值
- `set_type()` - 更改类型

#### 多值
- `count()` - 获取元素数量
- `is_empty()` - 检查是否为空
- `clear()` - 清空所有值
- `set_type()` - 更改类型
- `merge()` - 与另一个多值合并
- `to_value()` - 转换为单值

## 错误类型

```rust
use prism3_value::{ValueError, ValueResult};
use prism3_core::lang::DataType;

// 主要错误变体
let no_value = ValueError::NoValue;
let type_mismatch = ValueError::TypeMismatch {
    expected: DataType::Int32,
    actual: DataType::String,
};
let conversion_failed = ValueError::ConversionFailed {
    from: DataType::String,
    to: DataType::Int32,
};
let conversion_error = ValueError::ConversionError("无法解析数字".to_string());
let index_error = ValueError::IndexOutOfBounds {
    index: 5,
    len: 3,
};

fn demo() -> ValueResult<()> { Ok(()) }
```

所有可能失败的操作均返回 `ValueResult<T> = Result<T, ValueError>`。

## 支持的数据类型

### 基本类型
- **整数**: `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, `u128`
- **浮点数**: `f32`, `f64`
- **其他**: `bool`, `char`, `String`

### 字符串与字节
- **字符串**: `String`（直接存储）
- **字节**: `Vec<u8>`（直接存储）

### 日期/时间类型
- **Chrono 集成**: `NaiveDate`, `NaiveTime`, `NaiveDateTime`, `DateTime<Utc>`

### 大数类型
- **任意精度**: `BigInt`, `BigDecimal`（来自 `num-bigint` 和 `bigdecimal` 库）

## 性能优化

### 引用返回
字符串与字节数组的具名 getter 返回引用：
- `get_string() -> &str`
- `get_byte_array() -> &[u8]`

### 借用支持
`Value::new()` 和 `::set()` 支持借用类型：
- `&str` 自动转换为 `String`
- `&[u8]` 自动转换为 `Vec<u8>`

### 智能分发
`MultiValues::set/add` 支持三形态入参：`T`、`Vec<T>`、`&[T]`，内部自动分派最优路径。

## 序列化支持

### 完整支持
所有类型均实现 `Serialize`/`Deserialize`：
- `Value`
- `MultiValues`
- `NamedValue`
- `NamedMultiValues`

### 类型保持
- 序列化时保持完整的类型信息
- 反序列化时进行类型验证

## 与 Java 版本的差异

### 架构差异
- **继承层次 → 枚举表示**: 使用 Rust 枚举替代 Java 的继承体系
- **类型检查**: 从运行时类型检查转向编译期检查+少量运行时匹配
- **错误处理**: 统一错误返回而非异常；跨类型转换通过 `as_xxx()` 显式进行

### 性能优势
- **零成本抽象**: 枚举变体直接存储，无额外运行时开销
- **内存效率**: 基本类型直接存储，避免装箱
- **引用优化**: 大量 API 返回引用避免不必要的拷贝

## 扩展方向

### 已实现功能
- ✅ **大数支持**: `BigInt`/`BigDecimal` 支持（通过第三方库）
- ✅ **完整类型覆盖**: 支持所有基本类型和常用复合类型
- ✅ **序列化支持**: 完整的 `serde` 序列化/反序列化

### 未来扩展
- 🔄 **更多转换**: 扩展 `as_xxx()` 转换覆盖范围
- 🔄 **自定义类型**: 枚举类型/自定义类型的过程宏生成
- 🔄 **性能优化**: 进一步优化内存使用和访问性能

## 依赖项

```toml
[dependencies]
prism3-core = { path = "external/prism3-rust-commons/prism3-core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
num-traits = "0.2"
num-bigint = { version = "0.4", features = ["serde"] }
bigdecimal = { version = "0.4", features = ["serde"] }
```

## 测试与代码覆盖率

本项目保持全面的测试覆盖，对所有功能进行详细验证。有关覆盖率报告和测试说明，请参阅 [COVERAGE.zh_CN.md](COVERAGE.zh_CN.md) 文档。

## 许可证

Copyright (c) 2025 3-Prism Co. Ltd. All rights reserved.

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

**胡海星** - *棱芯科技有限公司*

---

有关 Prism3 生态系统的更多信息，请访问我们的 [GitHub 主页](https://github.com/3-prism)。
