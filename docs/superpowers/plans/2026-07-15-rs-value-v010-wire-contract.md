# rs-value 0.10 Wire V1 and API Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 发布 `qubit-value 0.10.0`，修复 `MultiValues` 默认严格读取的类型绕过，为可扩展公开 enum 建立 non-exhaustive 边界，并以公开 `ValueWireV1` 固化统一、严格、无旧格式兼容的版本化 wire。

**Architecture:** `Value`、`MultiValues` 与 `ValueContainer` 保留直接 `Serialize/Deserialize`，但实现改为委托给 `src/value_wire.rs` 中的 `ValueWireV1`。序列化使用借用 payload，反序列化使用独立于运行时 `value_type_table.rs` 的拥有型 wire 表；两个表之间由穷尽 `match` 形成编译期协议更新门槛。`rs-config` 为未来 `ValueError` 变体保留带 key 和 source 的兜底，`rs-metadata` 继续 derive 外层协议，只更新内嵌 value 的 golden wire。

**Tech Stack:** Rust 1.94、edition 2024、Serde/serde_json、thiserror、Cargo feature matrix、cargo-llvm-cov、项目 `.rs-ci` 脚本。

## Global Constraints

- `qubit-value` 目标版本固定为 `0.10.0`；`rs-config` 与 `rs-metadata` 自身版本不变。
- 允许破坏性变更；V1 不读取 0.9 外部标签格式，也不增加兼容 feature 或迁移分支。
- V1 信封固定为 `version` 数字 `1` 与 `value`；所有未知/缺失字段、错误版本、未知 shape、未知类型和错误入口 shape 均拒绝。
- `Value` 只接受 `scalar`，`MultiValues` 只接受 `collection`，`ValueContainer` 接受两者。
- wire 标签固定使用 `DataType::as_str()` 的 25 个小写名称；`Unset` payload 同样使用这些名称。
- `int128`/`uint128`、大数、`Duration` 与有限浮点规则保持现有严格语义。
- `Value::Json(null)`、`Unset(Json)`、具体空集合和单元素集合必须保持可区分。
- `Value`、`MultiValues`、`ValueError` 增加 `#[non_exhaustive]`；`ValueContainer` 不增加。
- 第四项泛型读取/转换 trait 门面严格不在本计划范围内。
- 不增加运行时依赖；不改变自然 JSON 投影、转换、比较或 schema 语义。
- 不直接运行 `cargo fmt`。每个修改过的仓库只能用 `./align-ci.sh` 格式化，再用 `./ci-check.sh` 验证。
- 当前 `rs-value` 已存在的格式差异由 `./align-ci.sh` 按项目 rustfmt 配置统一；不得手工回滚或覆盖用户已有修改。
- 未获用户明确授权，不运行 `git add`、`git commit` 或 `git push`；每个任务以只读 diff 检查替代提交步骤。

---

## File Map

### rs-value

- Create: `src/value_wire.rs` — V1 DTO、独立 wire 类型表、拥有/借用 payload、转换和手写 Serde。
- Create: `doc/user_guide.md`, `doc/user_guide.zh_CN.md` — V1 wire 与自然 JSON 用户指南。
- Modify: `src/value/value.rs`, `src/multi_values/multi_values.rs`, `src/value_container.rs`, `src/value_error.rs` — 手写 Serde 与 non-exhaustive 边界。
- Modify: `src/value_type_table.rs` 及其宏消费者 — 删除仅服务旧派生 Serde 的属性列。
- Modify: `src/multi_values/multi_values_core.rs` — 修复 `get_or/get_first_or`。
- Modify: `src/lib.rs` — 注册并公开 `ValueWireV1`，更新 crate-level wire 文档。
- Modify: `tests/multi_values/multi_values_tests.rs`, `tests/public_api_boundary_tests.rs`, `tests/tagged_serde_tests.rs`, `tests/value_container_tests.rs`, `tests/json_tests.rs` — 回归、golden 与拒绝矩阵。
- Modify: `README.md`, `README.zh_CN.md`, `Cargo.toml`, `Cargo.lock` — 0.10 文档与版本。

### rs-config

- Modify: `src/config_error.rs`, `tests/config_error_tests.rs` — 未来 `ValueError` 兜底并保留 key/source。
- Modify: `Cargo.toml`, `Cargo.lock` — `qubit-value` 依赖要求改为 `0.10`。

### rs-metadata

- Modify: `tests/filter/wire/condition_wire_tests.rs`, `tests/filter/wire/filter_expr_wire_tests.rs`, `tests/filter/wire/metadata_filter_serde_tests.rs` — 内嵌 Value V1 golden。
- Modify: `Cargo.toml`, `Cargo.lock` — `qubit-value` 依赖要求改为 `0.10`。

---

### Task 1: Fix strict defaulted MultiValues reads with regression coverage

**Files:**
- Modify: `tests/multi_values/multi_values_tests.rs`
- Modify: `src/multi_values/multi_values_core.rs`

**Interfaces:**
- Consumes: `MultiValues::get<T>()`, `MultiValues::get_first<T>()`, `MultiValues::is_unset()`.
- Produces: `get_or/get_first_or` 只把匹配声明类型的 `Unset` 映射为默认值。

- [ ] **Step 1: Add the regression test**

Append beside `test_multi_value_defaulted_reads_use_default_only_for_unset`:

```rust
#[test]
fn test_multi_value_defaulted_strict_reads_reject_mismatched_unset_type() {
    let unset_int = MultiValues::Unset(DataType::Int32);

    assert!(matches!(
        unset_int.get_or::<String>(["fallback"]),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
    assert!(matches!(
        unset_int.get_first_or::<String>("fallback"),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
}
```

- [ ] **Step 2: Run the new test and verify RED**

```bash
cargo +1.94.0 test --all-features --test integration_tests test_multi_value_defaulted_strict_reads_reject_mismatched_unset_type
```

Expected: FAIL because both current methods return fallback values for `Unset(Int32)` before checking the requested `String` type.

- [ ] **Step 3: Replace the two short-circuits with strict-result mapping**

```rust
pub fn get_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
where
    for<'a> Vec<T>: TryFrom<&'a Self, Error = ValueError>,
{
    match self.get() {
        Err(ValueError::NoValue) => Ok(default.into_value_default()),
        result => result,
    }
}

pub fn get_first_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
where
    for<'a> T: TryFrom<&'a Self, Error = ValueError>,
{
    match self.get_first() {
        Err(ValueError::NoValue) if self.is_unset() => Ok(default.into_value_default()),
        result => result,
    }
}
```

- [ ] **Step 4: Verify the complete default-state matrix**

```bash
cargo +1.94.0 test --all-features --test integration_tests multi_value_defaulted
git diff --check
git diff -- src/multi_values/multi_values_core.rs tests/multi_values/multi_values_tests.rs
```

Expected: PASS; no getter/converter bounds or unrelated APIs changed.

---

### Task 2: Enforce non-exhaustive public enums and make rs-config forward-compatible

**Files:**
- Modify: `rs-value/tests/public_api_boundary_tests.rs`
- Modify: `rs-value/src/value/value.rs`
- Modify: `rs-value/src/multi_values/multi_values.rs`
- Modify: `rs-value/src/value_error.rs`
- Modify: `rs-config/src/config_error.rs`
- Modify: `rs-config/tests/config_error_tests.rs`

**Interfaces:**
- Produces: `Value`, `MultiValues`, `ValueError` 对外不可穷尽匹配。
- Produces: `ConfigError::ValueError { key: String, source: ValueError }`。
- Preserves: 当前四个 `ValueError` 变体继续使用精细映射。

- [ ] **Step 1: Add an external-consumer compiler**

Add to `public_api_boundary_tests.rs`:

```rust
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_PROJECT_ID: AtomicUsize = AtomicUsize::new(0);

fn compile_all_features_consumer(source: &str) -> Output {
    let project_id = NEXT_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    let project_root = std::env::temp_dir().join(format!(
        "qubit-value-public-api-contract-{}-{project_id}",
        std::process::id(),
    ));
    let source_root = project_root.join("src");
    fs::create_dir_all(&source_root)
        .expect("temporary consumer source directory should be created");

    let dependency_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = format!(
        "[package]\n\
         name = \"qubit-value-public-api-consumer\"\n\
         version = \"0.0.0\"\n\
         edition = \"2024\"\n\n\
         [dependencies]\n\
         qubit-value = {{ path = \"{}\", default-features = false, features = [\"all\"] }}\n\n\
         [workspace]\n",
        dependency_path.display(),
    );
    fs::write(project_root.join("Cargo.toml"), manifest)
        .expect("temporary consumer manifest should be written");
    fs::write(source_root.join("main.rs"), source)
        .expect("temporary consumer source should be written");

    let output = Command::new("cargo")
        .args(["+1.94.0", "check", "--offline", "--quiet", "--target-dir"])
        .arg(project_root.join("target"))
        .current_dir(&project_root)
        .output()
        .expect("temporary consumer should invoke Cargo");
    fs::remove_dir_all(&project_root)
        .expect("temporary consumer directory should be removed");
    output
}

fn cargo_diagnostics(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

fn assert_non_exhaustive_match_failure(output: &Output) {
    let diagnostics = cargo_diagnostics(output);
    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("non-exhaustive"),
        "{diagnostics}",
    );
}
```

- [ ] **Step 2: Add three exhaustive-match consumer fixtures**

Use these complete fixture sources:

```rust
#[test]
fn test_external_consumer_cannot_exhaustively_match_value() {
    let output = compile_all_features_consumer(
        r#"
use qubit_value::Value;
fn classify(value: Value) -> usize {
    match value {
        Value::Unset(_) => 0, Value::Bool(_) => 1, Value::Char(_) => 2,
        Value::Int8(_) => 3, Value::Int16(_) => 4, Value::Int32(_) => 5,
        Value::Int64(_) => 6, Value::Int128(_) => 7, Value::UInt8(_) => 8,
        Value::UInt16(_) => 9, Value::UInt32(_) => 10, Value::UInt64(_) => 11,
        Value::UInt128(_) => 12, Value::Float32(_) => 13, Value::Float64(_) => 14,
        Value::BigInteger(_) => 15, Value::BigDecimal(_) => 16,
        Value::String(_) => 17, Value::Date(_) => 18, Value::Time(_) => 19,
        Value::DateTime(_) => 20, Value::Instant(_) => 21,
        Value::Duration(_) => 22, Value::Url(_) => 23,
        Value::StringMap(_) => 24, Value::Json(_) => 25,
    }
}
fn main() { let _ = classify; }
"#,
    );
    assert_non_exhaustive_match_failure(&output);
}

#[test]
fn test_external_consumer_cannot_exhaustively_match_multi_values() {
    let output = compile_all_features_consumer(
        r#"
use qubit_value::MultiValues;
fn classify(value: MultiValues) -> usize {
    match value {
        MultiValues::Unset(_) => 0, MultiValues::Bool(_) => 1,
        MultiValues::Char(_) => 2, MultiValues::Int8(_) => 3,
        MultiValues::Int16(_) => 4, MultiValues::Int32(_) => 5,
        MultiValues::Int64(_) => 6, MultiValues::Int128(_) => 7,
        MultiValues::UInt8(_) => 8, MultiValues::UInt16(_) => 9,
        MultiValues::UInt32(_) => 10, MultiValues::UInt64(_) => 11,
        MultiValues::UInt128(_) => 12, MultiValues::Float32(_) => 13,
        MultiValues::Float64(_) => 14, MultiValues::BigInteger(_) => 15,
        MultiValues::BigDecimal(_) => 16, MultiValues::String(_) => 17,
        MultiValues::Date(_) => 18, MultiValues::Time(_) => 19,
        MultiValues::DateTime(_) => 20, MultiValues::Instant(_) => 21,
        MultiValues::Duration(_) => 22, MultiValues::Url(_) => 23,
        MultiValues::StringMap(_) => 24, MultiValues::Json(_) => 25,
    }
}
fn main() { let _ = classify; }
"#,
    );
    assert_non_exhaustive_match_failure(&output);
}

#[test]
fn test_external_consumer_cannot_exhaustively_match_value_error() {
    let output = compile_all_features_consumer(
        r#"
use qubit_value::ValueError;
fn classify(error: ValueError) -> usize {
    match error {
        ValueError::NoValue => 0,
        ValueError::TypeMismatch { .. } => 1,
        ValueError::DataConversion(_) => 2,
        ValueError::DataListConversion(_) => 3,
    }
}
fn main() { let _ = classify; }
"#,
    );
    assert_non_exhaustive_match_failure(&output);
}
```

- [ ] **Step 3: Verify RED, then add the attributes**

```bash
cargo +1.94.0 test --all-features --test integration_tests external_consumer_cannot_exhaustively
```

Expected before the attributes: FAIL because the temporary consumers compile.

Use:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
```

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MultiValues {
```

```rust
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueError {
```

Re-run the same command. Expected: PASS with all three external consumers receiving non-exhaustive diagnostics.

- [ ] **Step 4: Add the rs-config fallback**

Add after `ConversionError`:

```rust
/// Value-layer error not covered by a specialized configuration mapping.
#[error("Value error at '{key}': {source}")]
ValueError {
    /// Configuration key/path where the value access failed.
    key: String,
    /// Original value-layer error.
    #[source]
    source: ValueError,
},
```

Use this complete mapper:

```rust
fn from_value_error(key: &str, error: ValueError) -> Self {
    match error {
        ValueError::NoValue => Self::PropertyHasNoValue(key.to_string()),
        ValueError::TypeMismatch { expected, actual } => Self::TypeMismatch {
            key: key.to_string(),
            expected,
            actual,
        },
        ValueError::DataConversion(source) => Self::from_data_conversion_error(key, source),
        ValueError::DataListConversion(error) => Self::ConversionError {
            key: key.to_string(),
            source_index: Some(error.source_index),
            source: error.source,
        },
        source => Self::ValueError {
            key: key.to_string(),
            source,
        },
    }
}
```

- [ ] **Step 5: Add and run the fallback error contract**

Add `use qubit_value::ValueError;` and:

```rust
#[test]
fn test_config_value_error_fallback_retains_key_and_source() {
    let error = ConfigError::ValueError {
        key: "server.port".to_string(),
        source: ValueError::TypeMismatch {
            expected: DataType::UInt16,
            actual: DataType::String,
        },
    };

    assert!(error.to_string().contains("server.port"));
    assert!(std::error::Error::source(&error).is_some());
    assert!(matches!(
        error,
        ConfigError::ValueError {
            key,
            source: ValueError::TypeMismatch {
                expected: DataType::UInt16,
                actual: DataType::String,
            },
        } if key == "server.port"
    ));
}
```

```bash
cd ../rs-config
cargo +1.94.0 test --all-features --test config_error_tests
cargo +1.94.0 test --all-features --test config_tests value_error
git diff --check
```

Expected: PASS; current detailed mappings remain unchanged and the new public variant retains its source.

---

### Task 3: Introduce ValueWireV1 and delegate runtime Serde

**Files:**
- Create: `rs-value/src/value_wire.rs`
- Modify: `rs-value/src/value_type_table.rs`
- Modify: `rs-value/src/value/value.rs`, `rs-value/src/value/value_constructor.rs`, `rs-value/src/value/value_getter.rs`, `rs-value/src/value/value_converters.rs`
- Modify: `rs-value/src/multi_values/multi_values.rs`, `rs-value/src/multi_values/multi_values_core.rs`, `rs-value/src/multi_values/multi_values_constructor.rs`, `rs-value/src/multi_values/multi_values_getter.rs`, `rs-value/src/multi_values/multi_values_converters.rs`
- Modify: `rs-value/src/value_container.rs`, `rs-value/src/json.rs`, `rs-value/src/lib.rs`
- Modify: `rs-value/tests/tagged_serde_tests.rs`, `rs-value/tests/value_container_tests.rs`

**Interfaces:**
- Produces: `pub struct ValueWireV1` with `VERSION`, `new`, `container`, `into_container`.
- Produces: owned `From<Value>`, `From<MultiValues>`, `From<ValueContainer>` and `From<ValueWireV1> for ValueContainer`.
- Produces: direct V1 Serde for `Value`, `MultiValues`, `ValueContainer`.
- Preserves: `src/wire.rs` remains the primitive canonical adapter module.

- [ ] **Step 1: Replace the old fixture model with V1 helpers and all-type fixtures**

In `tagged_serde_tests.rs`, import:

```rust
use qubit_value::{
    MultiValues, NamedMultiValues, NamedValue, Value, ValueContainer, ValueWireV1,
};
```

Replace `value_fixtures` and the two old golden tests with:

```rust
#[derive(Debug)]
struct ValueFixture {
    data_type: DataType,
    value: Value,
    tag: &'static str,
    payload: JsonValue,
}

fn tagged_payload(tag: &str, payload: JsonValue) -> JsonValue {
    JsonValue::Object(Map::from_iter([(tag.to_string(), payload)]))
}

fn wire_value(shape: &str, tag: &str, payload: JsonValue) -> JsonValue {
    json!({
        "version": 1,
        "value": JsonValue::Object(Map::from_iter([(
            shape.to_string(),
            tagged_payload(tag, payload),
        )])),
    })
}

fn scalar_wire(tag: &str, payload: JsonValue) -> JsonValue {
    wire_value("scalar", tag, payload)
}

fn collection_wire(tag: &str, payload: JsonValue) -> JsonValue {
    wire_value("collection", tag, payload)
}

fn value_fixtures() -> Vec<ValueFixture> {
    vec![
        ValueFixture { data_type: DataType::Bool, value: Value::Bool(true), tag: "bool", payload: json!(true) },
        ValueFixture { data_type: DataType::Char, value: Value::Char('界'), tag: "char", payload: json!("界") },
        ValueFixture { data_type: DataType::Int8, value: Value::Int8(-8), tag: "int8", payload: json!(-8) },
        ValueFixture { data_type: DataType::Int16, value: Value::Int16(-16), tag: "int16", payload: json!(-16) },
        ValueFixture { data_type: DataType::Int32, value: Value::Int32(-32), tag: "int32", payload: json!(-32) },
        ValueFixture { data_type: DataType::Int64, value: Value::Int64(-64), tag: "int64", payload: json!(-64) },
        ValueFixture { data_type: DataType::Int128, value: Value::Int128(i128::MIN), tag: "int128", payload: json!(i128::MIN.to_string()) },
        ValueFixture { data_type: DataType::UInt8, value: Value::UInt8(8), tag: "uint8", payload: json!(8) },
        ValueFixture { data_type: DataType::UInt16, value: Value::UInt16(16), tag: "uint16", payload: json!(16) },
        ValueFixture { data_type: DataType::UInt32, value: Value::UInt32(32), tag: "uint32", payload: json!(32) },
        ValueFixture { data_type: DataType::UInt64, value: Value::UInt64(64), tag: "uint64", payload: json!(64) },
        ValueFixture { data_type: DataType::UInt128, value: Value::UInt128(u128::MAX), tag: "uint128", payload: json!(u128::MAX.to_string()) },
        ValueFixture { data_type: DataType::Float32, value: Value::Float32(1.25), tag: "float32", payload: json!(1.25) },
        ValueFixture { data_type: DataType::Float64, value: Value::Float64(2.5), tag: "float64", payload: json!(2.5) },
        ValueFixture { data_type: DataType::BigInteger, value: Value::BigInteger(BigInt::from(123)), tag: "biginteger", payload: json!("123") },
        ValueFixture {
            data_type: DataType::BigDecimal,
            value: Value::BigDecimal(BigDecimal::from_str("123.4500").expect("valid decimal")),
            tag: "bigdecimal",
            payload: json!("123.4500"),
        },
        ValueFixture { data_type: DataType::String, value: Value::String("text".to_string()), tag: "string", payload: json!("text") },
        ValueFixture {
            data_type: DataType::Date,
            value: Value::Date(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()),
            tag: "date",
            payload: json!("2026-07-14"),
        },
        ValueFixture {
            data_type: DataType::Time,
            value: Value::Time(NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
            tag: "time",
            payload: json!("01:02:03"),
        },
        ValueFixture {
            data_type: DataType::DateTime,
            value: Value::DateTime(
                NaiveDateTime::parse_from_str("2026-07-14 01:02:03", "%Y-%m-%d %H:%M:%S").unwrap(),
            ),
            tag: "datetime",
            payload: json!("2026-07-14T01:02:03"),
        },
        ValueFixture {
            data_type: DataType::Instant,
            value: Value::Instant(Utc.with_ymd_and_hms(2026, 7, 14, 1, 2, 3).unwrap()),
            tag: "instant",
            payload: json!("2026-07-14T01:02:03Z"),
        },
        ValueFixture {
            data_type: DataType::Duration,
            value: Value::Duration(Duration::new(1, 2)),
            tag: "duration",
            payload: json!({"secs": 1, "nanos": 2}),
        },
        ValueFixture {
            data_type: DataType::Url,
            value: Value::Url(Url::parse("https://example.com/path").unwrap()),
            tag: "url",
            payload: json!("https://example.com/path"),
        },
        ValueFixture {
            data_type: DataType::StringMap,
            value: Value::StringMap(HashMap::from([("key".to_string(), "value".to_string())])),
            tag: "stringmap",
            payload: json!({"key": "value"}),
        },
        ValueFixture {
            data_type: DataType::Json,
            value: Value::Json(json!({"nested": true})),
            tag: "json",
            payload: json!({"nested": true}),
        },
    ]
}

#[test]
fn value_wire_v1_fixtures_cover_every_data_type() {
    let mut actual = value_fixtures()
        .into_iter()
        .map(|fixture| fixture.data_type)
        .collect::<Vec<_>>();
    let mut expected = DataType::ALL.to_vec();
    actual.sort_by_key(|data_type| data_type.as_str());
    expected.sort_by_key(|data_type| data_type.as_str());
    assert_eq!(actual, expected);
}

#[test]
fn value_wire_v1_scalar_golden_round_trips_all_types() {
    for fixture in value_fixtures() {
        let expected = scalar_wire(fixture.tag, fixture.payload);
        assert_eq!(serde_json::to_value(&fixture.value).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<Value>(expected.clone()).unwrap(),
            fixture.value,
        );
        let dto = ValueWireV1::from(fixture.value.clone());
        assert_eq!(serde_json::to_value(&dto).unwrap(), expected);
        let container = ValueContainer::Scalar(fixture.value.clone());
        assert_eq!(serde_json::to_value(&container).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<ValueContainer>(expected.clone()).unwrap(),
            container,
        );
        let restored = serde_json::from_value::<ValueWireV1>(expected).unwrap();
        assert_eq!(
            ValueContainer::from(restored),
            ValueContainer::Scalar(fixture.value),
        );
    }
}

#[test]
fn value_wire_v1_collection_golden_round_trips_all_types() {
    for fixture in value_fixtures() {
        let values = MultiValues::from(fixture.value);
        let expected = collection_wire(fixture.tag, json!([fixture.payload]));
        assert_eq!(serde_json::to_value(&values).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<MultiValues>(expected.clone()).unwrap(),
            values,
        );
        let dto = ValueWireV1::from(values.clone());
        assert_eq!(serde_json::to_value(&dto).unwrap(), expected);
        let container = ValueContainer::Collection(values.clone());
        assert_eq!(serde_json::to_value(&container).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<ValueContainer>(expected.clone()).unwrap(),
            container,
        );
        let restored = serde_json::from_value::<ValueWireV1>(expected).unwrap();
        assert_eq!(
            ValueContainer::from(restored),
            ValueContainer::Collection(values),
        );
    }
}
```

- [ ] **Step 2: Add state, conversion and named-wrapper golden tests**

```rust
#[test]
fn value_wire_v1_preserves_unset_empty_singleton_and_json_null() {
    let cases = [
        (
            ValueContainer::Scalar(Value::Unset(DataType::Int32)),
            scalar_wire("unset", json!("int32")),
        ),
        (
            ValueContainer::Collection(MultiValues::Unset(DataType::Int32)),
            collection_wire("unset", json!("int32")),
        ),
        (
            ValueContainer::Collection(MultiValues::Int32(Vec::new())),
            collection_wire("int32", json!([])),
        ),
        (
            ValueContainer::Collection(MultiValues::Int32(vec![42])),
            collection_wire("int32", json!([42])),
        ),
        (
            ValueContainer::Scalar(Value::Json(JsonValue::Null)),
            scalar_wire("json", JsonValue::Null),
        ),
        (
            ValueContainer::Scalar(Value::Unset(DataType::Json)),
            scalar_wire("unset", json!("json")),
        ),
    ];
    for (container, expected) in cases {
        assert_eq!(serde_json::to_value(&container).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<ValueContainer>(expected).unwrap(),
            container,
        );
    }
}

#[test]
fn value_wire_v1_owned_conversions_preserve_shape() {
    let scalar = ValueWireV1::from(Value::Int32(42));
    assert_eq!(
        scalar.container(),
        &ValueContainer::Scalar(Value::Int32(42)),
    );
    let collection = ValueWireV1::from(MultiValues::Int32(vec![42]));
    assert_eq!(
        collection.container(),
        &ValueContainer::Collection(MultiValues::Int32(vec![42])),
    );
    assert_eq!(
        ValueContainer::from(scalar),
        ValueContainer::Scalar(Value::Int32(42)),
    );
    assert_eq!(
        collection.into_container(),
        ValueContainer::Collection(MultiValues::Int32(vec![42])),
    );
}

#[test]
fn named_values_keep_outer_fields_and_embed_value_wire_v1() {
    let named = NamedValue::new("port", Value::Int32(8080));
    let expected = json!({
        "name": "port",
        "value": scalar_wire("int32", json!(8080)),
    });
    assert_eq!(serde_json::to_value(&named).unwrap(), expected);
    assert_eq!(serde_json::from_value::<NamedValue>(expected).unwrap(), named);

    let named = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081]));
    let expected = json!({
        "name": "ports",
        "value": collection_wire("int32", json!([8080, 8081])),
    });
    assert_eq!(serde_json::to_value(&named).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<NamedMultiValues>(expected).unwrap(),
        named,
    );
}
```

Update `test_value_container_tagged_wire_preserves_shape` to expect:

```rust
json!({"version": 1, "value": {"scalar": {"int32": 42}}})
json!({"version": 1, "value": {"collection": {"int32": [42]}}})
```

- [ ] **Step 3: Run the V1 tests and verify RED**

```bash
cargo +1.94.0 test --all-features --test integration_tests value_wire_v1
```

Expected: compilation FAIL because `ValueWireV1` does not exist and runtime Serde still emits 0.9 tags.

- [ ] **Step 4: Remove obsolete runtime-table Serde metadata**

Replace the callback body in `value_type_table.rs` with this complete
runtime-only table:

```rust
$macro! {
    $($arg),*;
    ([], Bool, bool, ::qubit_datatype::DataType::Bool, copy, json_bool, "Boolean value", "Boolean value list"),
    ([], Char, char, ::qubit_datatype::DataType::Char, copy, json_string, "Character value", "Character value list"),
    ([], Int8, i8, ::qubit_datatype::DataType::Int8, copy, json_number, "8-bit signed integer", "8-bit signed integer list"),
    ([], Int16, i16, ::qubit_datatype::DataType::Int16, copy, json_number, "16-bit signed integer", "16-bit signed integer list"),
    ([], Int32, i32, ::qubit_datatype::DataType::Int32, copy, json_number, "32-bit signed integer", "32-bit signed integer list"),
    ([], Int64, i64, ::qubit_datatype::DataType::Int64, copy, json_number, "64-bit signed integer", "64-bit signed integer list"),
    ([], Int128, i128, ::qubit_datatype::DataType::Int128, copy, json_string, "128-bit signed integer", "128-bit signed integer list"),
    ([], UInt8, u8, ::qubit_datatype::DataType::UInt8, copy, json_number, "8-bit unsigned integer", "8-bit unsigned integer list"),
    ([], UInt16, u16, ::qubit_datatype::DataType::UInt16, copy, json_number, "16-bit unsigned integer", "16-bit unsigned integer list"),
    ([], UInt32, u32, ::qubit_datatype::DataType::UInt32, copy, json_number, "32-bit unsigned integer", "32-bit unsigned integer list"),
    ([], UInt64, u64, ::qubit_datatype::DataType::UInt64, copy, json_number, "64-bit unsigned integer", "64-bit unsigned integer list"),
    ([], UInt128, u128, ::qubit_datatype::DataType::UInt128, copy, json_string, "128-bit unsigned integer", "128-bit unsigned integer list"),
    ([], Float32, f32, ::qubit_datatype::DataType::Float32, copy, json_float, "32-bit floating-point number", "32-bit floating-point number list"),
    ([], Float64, f64, ::qubit_datatype::DataType::Float64, copy, json_float, "64-bit floating-point number", "64-bit floating-point number list"),
    ([cfg(feature = "big-number")], BigInteger, ::num_bigint::BigInt, ::qubit_datatype::DataType::BigInteger, clone, json_string, "Arbitrary-precision integer", "Arbitrary-precision integer list"),
    ([cfg(feature = "big-number")], BigDecimal, ::bigdecimal::BigDecimal, ::qubit_datatype::DataType::BigDecimal, clone, json_string, "Arbitrary-precision decimal", "Arbitrary-precision decimal list"),
    ([], String, String, ::qubit_datatype::DataType::String, clone, json_string, "String value", "String value list"),
    ([cfg(feature = "chrono")], Date, ::chrono::NaiveDate, ::qubit_datatype::DataType::Date, copy, json_string, "Calendar date", "Calendar date list"),
    ([cfg(feature = "chrono")], Time, ::chrono::NaiveTime, ::qubit_datatype::DataType::Time, copy, json_string, "Time of day", "Time-of-day list"),
    ([cfg(feature = "chrono")], DateTime, ::chrono::NaiveDateTime, ::qubit_datatype::DataType::DateTime, copy, json_string, "Date and time", "Date-and-time list"),
    ([cfg(feature = "chrono")], Instant, ::chrono::DateTime<::chrono::Utc>, ::qubit_datatype::DataType::Instant, copy, json_string, "UTC instant", "UTC instant list"),
    ([], Duration, ::std::time::Duration, ::qubit_datatype::DataType::Duration, copy, json_duration, "Duration", "Duration list"),
    ([cfg(feature = "url")], Url, ::url::Url, ::qubit_datatype::DataType::Url, clone, json_string, "URL", "URL list"),
    ([], StringMap, ::std::collections::HashMap<String, String>, ::qubit_datatype::DataType::StringMap, clone, json_object, "Map with string keys and values", "String-map list"),
    ([cfg(feature = "json")], Json, ::serde_json::Value, ::qubit_datatype::DataType::Json, clone, json_identity, "JSON value", "JSON value list"),
}
```

Update its rustdoc to state, in order: feature attributes, enum variant, Rust
storage type, `DataType`, materialization strategy, natural JSON class, and the
two public variant docs. In every macro consumer listed under this task replace:

```rust
([$($cfg:meta),*], [$($value_attr:meta),*], [$($multi_attr:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)
```

with:

```rust
([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)
```

Delete the corresponding two multiline matcher lines where used. Update the table rustdoc to describe only its eight fields. Verify:

```bash
! rg -n '\$value_attr|\$multi_attr' src
```

Expected: no matches. Do not alter method bodies, bounds, materialization or `json_class`.

- [ ] **Step 5: Remove derived Serde from runtime containers**

Use:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
```

```rust
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum MultiValues {
```

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ValueContainer {
```

Remove now-unused Serde imports from these files. Keep named wrapper derives unchanged.

- [ ] **Step 6: Create `src/value_wire.rs` with the independent type table and payloads**

Use this complete table and owned/borrowed payload definition:

```rust
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Versioned, type-preserving wire representation for runtime values.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use qubit_datatype::DataType;

use crate::{MultiValues, Value, ValueContainer};

const VALUE_WIRE_V1_VERSION: u8 = 1;

macro_rules! for_each_wire_type {
    ($macro:ident) => {
        $macro! {
            ([], [], [], Bool, bool, "bool"),
            ([], [], [], Char, char, "char"),
            ([], [], [], Int8, i8, "int8"),
            ([], [], [], Int16, i16, "int16"),
            ([], [], [], Int32, i32, "int32"),
            ([], [], [], Int64, i64, "int64"),
            ([], [serde(with = "crate::wire::int128")], [serde(with = "crate::wire::int128_vec")], Int128, i128, "int128"),
            ([], [], [], UInt8, u8, "uint8"),
            ([], [], [], UInt16, u16, "uint16"),
            ([], [], [], UInt32, u32, "uint32"),
            ([], [], [], UInt64, u64, "uint64"),
            ([], [serde(with = "crate::wire::uint128")], [serde(with = "crate::wire::uint128_vec")], UInt128, u128, "uint128"),
            ([], [serde(with = "crate::wire::float32")], [serde(with = "crate::wire::float32_vec")], Float32, f32, "float32"),
            ([], [serde(with = "crate::wire::float64")], [serde(with = "crate::wire::float64_vec")], Float64, f64, "float64"),
            ([cfg(feature = "big-number")], [serde(with = "crate::wire::big_integer")], [serde(with = "crate::wire::big_integer_vec")], BigInteger, num_bigint::BigInt, "biginteger"),
            ([cfg(feature = "big-number")], [serde(with = "crate::wire::big_decimal")], [serde(with = "crate::wire::big_decimal_vec")], BigDecimal, bigdecimal::BigDecimal, "bigdecimal"),
            ([], [], [], String, String, "string"),
            ([cfg(feature = "chrono")], [], [], Date, chrono::NaiveDate, "date"),
            ([cfg(feature = "chrono")], [], [], Time, chrono::NaiveTime, "time"),
            ([cfg(feature = "chrono")], [], [], DateTime, chrono::NaiveDateTime, "datetime"),
            ([cfg(feature = "chrono")], [], [], Instant, chrono::DateTime<chrono::Utc>, "instant"),
            ([], [serde(with = "crate::wire::duration")], [serde(with = "crate::wire::duration_vec")], Duration, std::time::Duration, "duration"),
            ([cfg(feature = "url")], [], [], Url, url::Url, "url"),
            ([], [], [], StringMap, std::collections::HashMap<String, String>, "stringmap"),
            ([cfg(feature = "json")], [], [], Json, serde_json::Value, "json"),
        }
    };
}

macro_rules! define_wire_payloads {
    (
        $(
            (
                [$($cfg:meta),*],
                [$($scalar_attr:meta),*],
                [$($collection_attr:meta),*],
                $variant:ident,
                $type:ty,
                $tag:literal
            )
        ),+ $(,)?
    ) => {
        #[derive(Serialize)]
        enum ScalarWireRef<'a> {
            #[serde(rename = "unset")]
            Unset(&'a DataType),
            $(
                $(#[$cfg])*
                $(#[$scalar_attr])*
                #[serde(rename = $tag)]
                $variant(&'a $type),
            )+
        }

        #[derive(Deserialize)]
        enum ScalarWireOwned {
            #[serde(rename = "unset")]
            Unset(DataType),
            $(
                $(#[$cfg])*
                $(#[$scalar_attr])*
                #[serde(rename = $tag)]
                $variant($type),
            )+
        }

        #[derive(Serialize)]
        enum CollectionWireRef<'a> {
            #[serde(rename = "unset")]
            Unset(&'a DataType),
            $(
                $(#[$cfg])*
                $(#[$collection_attr])*
                #[serde(rename = $tag)]
                $variant(&'a Vec<$type>),
            )+
        }

        #[derive(Deserialize)]
        enum CollectionWireOwned {
            #[serde(rename = "unset")]
            Unset(DataType),
            $(
                $(#[$cfg])*
                $(#[$collection_attr])*
                #[serde(rename = $tag)]
                $variant(Vec<$type>),
            )+
        }
    };
}

for_each_wire_type!(define_wire_payloads);
```

- [ ] **Step 7: Complete `src/value_wire.rs` conversions, envelope and Serde**

Append:

```rust
macro_rules! define_wire_conversions {
    (
        $(
            (
                [$($cfg:meta),*],
                [$($scalar_attr:meta),*],
                [$($collection_attr:meta),*],
                $variant:ident,
                $type:ty,
                $tag:literal
            )
        ),+ $(,)?
    ) => {
        impl<'a> From<&'a Value> for ScalarWireRef<'a> {
            fn from(value: &'a Value) -> Self {
                match value {
                    Value::Unset(data_type) => Self::Unset(data_type),
                    $(
                        $(#[$cfg])*
                        Value::$variant(value) => Self::$variant(value),
                    )+
                }
            }
        }

        impl From<ScalarWireOwned> for Value {
            fn from(value: ScalarWireOwned) -> Self {
                match value {
                    ScalarWireOwned::Unset(data_type) => Self::Unset(data_type),
                    $(
                        $(#[$cfg])*
                        ScalarWireOwned::$variant(value) => Self::$variant(value),
                    )+
                }
            }
        }

        impl<'a> From<&'a MultiValues> for CollectionWireRef<'a> {
            fn from(values: &'a MultiValues) -> Self {
                match values {
                    MultiValues::Unset(data_type) => Self::Unset(data_type),
                    $(
                        $(#[$cfg])*
                        MultiValues::$variant(values) => Self::$variant(values),
                    )+
                }
            }
        }

        impl From<CollectionWireOwned> for MultiValues {
            fn from(values: CollectionWireOwned) -> Self {
                match values {
                    CollectionWireOwned::Unset(data_type) => Self::Unset(data_type),
                    $(
                        $(#[$cfg])*
                        CollectionWireOwned::$variant(values) => Self::$variant(values),
                    )+
                }
            }
        }
    };
}

for_each_wire_type!(define_wire_conversions);

#[derive(Serialize)]
enum WireShapeRef<'a> {
    #[serde(rename = "scalar")]
    Scalar(ScalarWireRef<'a>),
    #[serde(rename = "collection")]
    Collection(CollectionWireRef<'a>),
}

#[derive(Deserialize)]
enum WireShapeOwned {
    #[serde(rename = "scalar")]
    Scalar(ScalarWireOwned),
    #[serde(rename = "collection")]
    Collection(CollectionWireOwned),
}

impl From<WireShapeOwned> for ValueContainer {
    fn from(value: WireShapeOwned) -> Self {
        match value {
            WireShapeOwned::Scalar(value) => Self::Scalar(value.into()),
            WireShapeOwned::Collection(values) => Self::Collection(values.into()),
        }
    }
}

impl<'a> From<&'a ValueContainer> for WireShapeRef<'a> {
    fn from(value: &'a ValueContainer) -> Self {
        match value {
            ValueContainer::Scalar(value) => Self::Scalar(value.into()),
            ValueContainer::Collection(values) => Self::Collection(values.into()),
        }
    }
}

#[derive(Serialize)]
struct WireEnvelopeRef<'a> {
    version: u8,
    value: WireShapeRef<'a>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnvelopeOwned {
    version: u8,
    value: WireShapeOwned,
}

fn serialize_wire<S>(value: WireShapeRef<'_>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    WireEnvelopeRef {
        version: VALUE_WIRE_V1_VERSION,
        value,
    }
    .serialize(serializer)
}

fn deserialize_wire<'de, D>(deserializer: D) -> Result<ValueContainer, D::Error>
where
    D: Deserializer<'de>,
{
    let envelope = WireEnvelopeOwned::deserialize(deserializer)?;
    if envelope.version != VALUE_WIRE_V1_VERSION {
        return Err(D::Error::custom(format_args!(
            "unsupported qubit-value wire version {}",
            envelope.version,
        )));
    }
    Ok(envelope.value.into())
}

/// Stable version-one wire DTO for a scalar or homogeneous collection.
#[derive(Debug, Clone, PartialEq)]
pub struct ValueWireV1 {
    value: ValueContainer,
}

impl ValueWireV1 {
    /// Numeric version emitted and accepted by this DTO.
    pub const VERSION: u8 = VALUE_WIRE_V1_VERSION;

    /// Creates a V1 DTO from an explicit scalar-or-collection container.
    #[inline]
    pub const fn new(value: ValueContainer) -> Self {
        Self { value }
    }

    /// Returns the runtime container represented by this DTO.
    #[inline]
    pub const fn container(&self) -> &ValueContainer {
        &self.value
    }

    /// Consumes the DTO and returns its runtime container.
    #[inline]
    pub fn into_container(self) -> ValueContainer {
        self.value
    }
}

impl From<Value> for ValueWireV1 {
    #[inline]
    fn from(value: Value) -> Self {
        Self::new(ValueContainer::Scalar(value))
    }
}

impl From<MultiValues> for ValueWireV1 {
    #[inline]
    fn from(values: MultiValues) -> Self {
        Self::new(ValueContainer::Collection(values))
    }
}

impl From<ValueContainer> for ValueWireV1 {
    #[inline]
    fn from(value: ValueContainer) -> Self {
        Self::new(value)
    }
}

impl From<ValueWireV1> for ValueContainer {
    #[inline]
    fn from(value: ValueWireV1) -> Self {
        value.into_container()
    }
}

impl Serialize for ValueWireV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire((&self.value).into(), serializer)
    }
}

impl<'de> Deserialize<'de> for ValueWireV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer).map(Self::new)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Scalar(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_wire(deserializer)? {
            ValueContainer::Scalar(value) => Ok(value),
            ValueContainer::Collection(_) => {
                Err(D::Error::custom("expected scalar value wire shape"))
            }
        }
    }
}

impl Serialize for MultiValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(WireShapeRef::Collection(self.into()), serializer)
    }
}

impl<'de> Deserialize<'de> for MultiValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserialize_wire(deserializer)? {
            ValueContainer::Collection(values) => Ok(values),
            ValueContainer::Scalar(_) => {
                Err(D::Error::custom("expected collection value wire shape"))
            }
        }
    }
}

impl Serialize for ValueContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_wire(self.into(), serializer)
    }
}

impl<'de> Deserialize<'de> for ValueContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_wire(deserializer)
    }
}
```

Do not call `for_each_value_type!` from this module. The duplicate table and exhaustive runtime matches are the protocol guard.

- [ ] **Step 8: Register/export and run basic V1 tests**

In `src/lib.rs`:

```rust
mod value_wire;
mod wire;
```

and:

```rust
pub use value_wire::ValueWireV1;
```

Run:

```bash
cargo +1.94.0 test --all-features --test integration_tests value_wire_v1
cargo +1.94.0 test --all-features --test integration_tests named_values_keep_outer_fields
cargo +1.94.0 test --all-features --test integration_tests test_value_container_tagged_wire_preserves_shape
cargo +1.94.0 test --test core_feature_tests
cargo +1.94.0 test --test feature_contract_tests
! rg -n 'for_each_value_type' src/value_wire.rs
! rg -n '\$value_attr|\$multi_attr' src
git diff --check
```

Expected: all tests PASS and both searches return no matches.

---

### Task 4: Complete strict V1 rejection and primitive-boundary coverage

**Files:**
- Modify: `rs-value/tests/tagged_serde_tests.rs`
- Modify: `rs-value/tests/json_tests.rs`

**Interfaces:**
- Consumes: `scalar_wire`, `collection_wire`, `ValueWireV1` from Task 3.
- Produces: explicit rejection of malformed envelopes, wrong shapes, unknown types, legacy formats and noncanonical payloads.

- [ ] **Step 1: Add envelope, shape and legacy rejection tests**

```rust
#[test]
fn value_wire_v1_rejects_invalid_envelopes_and_unknown_tags() {
    let valid_value = json!({"scalar": {"int32": 42}});
    for invalid in [
        json!({"value": valid_value}),
        json!({"version": "1", "value": valid_value}),
        json!({"version": 2, "value": valid_value}),
        json!({"version": 1}),
        json!({"version": 1, "value": valid_value, "extra": true}),
        json!({"version": 1, "value": {"unknown": {"int32": 42}}}),
        json!({"version": 1, "value": {"scalar": {"unknown": 42}}}),
        json!({"version": 1, "value": {"scalar": {"int32": 42, "bool": true}}}),
    ] {
        assert!(
            serde_json::from_value::<ValueWireV1>(invalid.clone()).is_err(),
            "unexpectedly accepted {invalid}",
        );
    }
}

#[test]
fn value_wire_v1_rejects_runtime_entry_shape_mismatches() {
    let scalar = scalar_wire("int32", json!(42));
    let collection = collection_wire("int32", json!([42]));
    assert!(serde_json::from_value::<Value>(collection.clone()).is_err());
    assert!(serde_json::from_value::<MultiValues>(scalar.clone()).is_err());
    assert!(serde_json::from_value::<ValueContainer>(scalar).is_ok());
    assert!(serde_json::from_value::<ValueContainer>(collection).is_ok());
}

#[test]
fn value_wire_v1_rejects_all_legacy_external_tag_shapes() {
    for legacy in [
        json!({"Int32": 42}),
        json!({"Unset": "int32"}),
        json!({"Scalar": {"Int32": 42}}),
        json!({"Collection": {"Int32": [42]}}),
    ] {
        assert!(serde_json::from_value::<ValueWireV1>(legacy.clone()).is_err());
        assert!(serde_json::from_value::<Value>(legacy.clone()).is_err());
        assert!(serde_json::from_value::<MultiValues>(legacy).is_err());
    }
}
```

- [ ] **Step 2: Rewrite canonical primitive rejection tests through V1**

```rust
#[test]
fn value_wire_v1_wide_integer_payloads_require_canonical_decimal_strings() {
    for invalid in [
        scalar_wire("int128", json!(128)),
        scalar_wire("int128", json!("12x")),
        scalar_wire("int128", json!("+1")),
        scalar_wire("int128", json!("01")),
        scalar_wire("uint128", json!("-1")),
        scalar_wire("uint128", json!("01")),
    ] {
        assert!(serde_json::from_value::<Value>(invalid).is_err());
    }
    for invalid in [
        collection_wire("uint128", json!(["1", 2])),
        collection_wire("uint128", json!(["1", "02"])),
    ] {
        assert!(serde_json::from_value::<MultiValues>(invalid).is_err());
    }
}

#[test]
fn value_wire_v1_big_number_payloads_require_canonical_decimal_strings() {
    for invalid in [
        scalar_wire("biginteger", json!([1, [123]])),
        scalar_wire("biginteger", json!("12x")),
        scalar_wire("biginteger", json!("+1")),
        scalar_wire("biginteger", json!("001")),
        scalar_wire("bigdecimal", json!(12.5)),
        scalar_wire("bigdecimal", json!("001.0")),
    ] {
        assert!(serde_json::from_value::<Value>(invalid).is_err());
    }
    assert!(
        serde_json::from_value::<MultiValues>(collection_wire(
            "biginteger",
            json!(["1", "02"]),
        ))
        .is_err(),
    );
}

#[test]
fn value_wire_v1_duration_payload_is_strict() {
    assert!(
        serde_json::from_value::<Value>(scalar_wire(
            "duration",
            json!({"secs": 1, "nanos": 1_000_000_000}),
        ))
        .is_err(),
    );
    assert!(
        serde_json::from_value::<Value>(scalar_wire(
            "duration",
            json!({"secs": 1, "nanos": 2, "extra": 3}),
        ))
        .is_err(),
    );
    assert!(
        serde_json::from_value::<MultiValues>(collection_wire(
            "duration",
            json!([{"secs": 1, "nanos": 2, "extra": 3}]),
        ))
        .is_err(),
    );
}
```

- [ ] **Step 3: Wrap custom non-finite deserializers in a V1 envelope**

Add `MapDeserializer` to the `serde::de::value` imports in `json_tests.rs`. Add:

```rust
enum Either<L, R> {
    Left(L),
    Right(R),
}

enum EitherDeserializer<L, R> {
    Left(L),
    Right(R),
}

impl<'de, L, R> IntoDeserializer<'de, DeError> for Either<L, R>
where
    L: IntoDeserializer<'de, DeError>,
    R: IntoDeserializer<'de, DeError>,
{
    type Deserializer = EitherDeserializer<L::Deserializer, R::Deserializer>;

    fn into_deserializer(self) -> Self::Deserializer {
        match self {
            Self::Left(value) => EitherDeserializer::Left(value.into_deserializer()),
            Self::Right(value) => EitherDeserializer::Right(value.into_deserializer()),
        }
    }
}

impl<'de, L, R> serde::Deserializer<'de> for EitherDeserializer<L, R>
where
    L: serde::Deserializer<'de, Error = DeError>,
    R: serde::Deserializer<'de, Error = DeError>,
{
    type Error = DeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Left(value) => value.deserialize_any(visitor),
            Self::Right(value) => value.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

fn wire_payload<'de, V>(
    shape: &'static str,
    variant: &'static str,
    value: V,
) -> impl serde::Deserializer<'de, Error = DeError>
where
    V: IntoDeserializer<'de, DeError>,
{
    let payload = tagged_payload(variant, value);
    let shape = tagged_payload(shape, payload);
    MapDeserializer::new(
        vec![
            ("version", Either::Left(1_u8)),
            ("value", Either::Right(shape)),
        ]
        .into_iter(),
    )
}
```

Replace the old tagged float tests with:

```rust
#[test]
fn test_value_wire_v1_deserialization_rejects_non_finite_payloads() {
    for error in [
        Value::deserialize(wire_payload("scalar", "float32", f32::NAN)).unwrap_err(),
        Value::deserialize(wire_payload("scalar", "float64", f64::INFINITY)).unwrap_err(),
    ] {
        assert!(
            error.to_string().contains("non-finite floating-point value"),
            "{error}",
        );
    }

    let error = MultiValues::deserialize(wire_payload(
        "collection",
        "float32",
        DeserializerSequence(vec![1.0_f32, f32::NAN]),
    ))
    .unwrap_err();
    assert!(error.to_string().contains("non-finite floating-point value"));

    let error = MultiValues::deserialize(wire_payload(
        "collection",
        "float64",
        DeserializerSequence(vec![1.0_f64, f64::NEG_INFINITY]),
    ))
    .unwrap_err();
    assert!(error.to_string().contains("non-finite floating-point value"));
}

#[test]
fn test_value_wire_v1_float_deserialization_propagates_malformed_payloads() {
    assert!(
        Value::deserialize(wire_payload("scalar", "float32", "not-a-float")).is_err(),
    );
    assert!(
        MultiValues::deserialize(wire_payload(
            "collection",
            "float64",
            DeserializerSequence(vec!["not-a-float"]),
        ))
        .is_err(),
    );
}
```

- [ ] **Step 4: Run the complete boundary suite**

```bash
cargo +1.94.0 test --all-features --test integration_tests value_wire_v1
cargo +1.94.0 test --all-features --test integration_tests non_finite
cargo +1.94.0 test --all-features --test integration_tests natural_json
rg -n '"(Int32|Int64|Int128|UInt128|Float64|String|Unset|Scalar|Collection)"' tests/tagged_serde_tests.rs tests/value_container_tests.rs
```

Expected: tests PASS; the final search finds only the four deliberate legacy-rejection fixtures in `value_wire_v1_rejects_all_legacy_external_tag_shapes` and no old golden expectation. Natural JSON assertions remain unchanged.

---

### Task 5: Publish the 0.10 contract in package metadata and docs

**Files:**
- Modify: `rs-value/Cargo.toml`, `rs-value/Cargo.lock`
- Modify: `rs-value/src/lib.rs`
- Modify: `rs-value/README.md`, `rs-value/README.zh_CN.md`
- Create: `rs-value/doc/user_guide.md`, `rs-value/doc/user_guide.zh_CN.md`

**Interfaces:**
- Produces: package version `0.10.0` and dependency examples at `0.10`.
- Produces: explicit V1 versus natural-JSON documentation.

- [ ] **Step 1: Bump package and README dependency versions**

```toml
version = "0.10.0"
```

```toml
qubit-value = { version = "0.10", features = ["all"] }
```

- [ ] **Step 2: Replace the English serialization-contract section**

~~~~markdown
## Serialization Contracts

Enabled types implement `Serialize`/`Deserialize`:

- `Value`, `MultiValues`, `ValueContainer`, `NamedValue`, `NamedMultiValues`
- `ValueWireV1`, the public version-one wire DTO

Type-preserving Serde uses one strict versioned envelope:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

Collections use `collection` instead of `scalar`; an unset payload uses
`{"unset":"int32"}`. `Value` accepts only scalar, `MultiValues` accepts only
collection, and `ValueContainer` accepts both shapes. `Value` and
`ValueContainer::Scalar` have identical wire, as do `MultiValues` and
`ValueContainer::Collection`. Named wrappers keep their outer `name`/`value`
fields and place this envelope in `value`.

Version 0.10 intentionally rejects former externally tagged forms such as
`{"Int32":42}`, `{"Unset":"int32"}`, and `{"Scalar":{"Int32":42}}`. It also
rejects missing or unknown fields, versions other than numeric `1`, unknown
shapes and types, and mismatched runtime entry shapes.

`Int128`, `UInt128`, `BigInteger`, and `BigDecimal` payloads use canonical
decimal strings. `Duration` uses `{"secs":u64,"nanos":u32}` and requires nanos
below one second. Float payloads must be finite.

This type-preserving V1 wire is separate from `to_json_value()`, which emits
natural JSON without runtime type tags and projects unset values to `null`.
~~~~

- [ ] **Step 3: Replace the Chinese serialization-contract section**

~~~~markdown
## 序列化契约

启用的类型均实现 `Serialize`/`Deserialize`：

- `Value`、`MultiValues`、`ValueContainer`、`NamedValue`、`NamedMultiValues`
- 公开的第一版 wire DTO `ValueWireV1`

保留类型信息的 Serde 统一使用严格的版本化信封：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
```

集合使用 `collection` 而不是 `scalar`；未设置值使用 `{"unset":"int32"}`。
`Value` 只接受 scalar，`MultiValues` 只接受 collection，`ValueContainer`
接受两种形态。Named wrapper 保留外层 `name`/`value` 字段，并将该信封放入
`value`。

0.10 会明确拒绝旧的外部标签格式，例如 `{"Int32":42}`、
`{"Unset":"int32"}` 和 `{"Scalar":{"Int32":42}}`。缺失或未知字段、不是
数字 `1` 的版本、未知 shape/类型，以及与运行时入口不匹配的 shape 同样会拒绝。

`Int128`、`UInt128`、`BigInteger` 与 `BigDecimal` 使用 canonical 十进制
字符串。`Duration` 使用 `{"secs":u64,"nanos":u32}`，且 nanos 必须小于一秒。
浮点 payload 必须是有限值。

保留类型的 V1 wire 与 `to_json_value()` 的自然 JSON 投影是两个独立契约。
自然 JSON 不包含运行时类型标签，未设置值投影为 `null`。
~~~~

- [ ] **Step 4: Update crate-level rustdoc**

```rust
//! - Serde uses the strict, type-preserving [`ValueWireV1`] envelope. With
//!   `converter`, `to_json_value` provides a separate natural JSON projection.
//! - Version one rejects the pre-0.10 externally tagged representation.
//! - Non-finite floats may exist in memory, but V1 Serde and natural JSON
//!   reject them because JSON has no `NaN` or infinity number literals.
```

- [ ] **Step 5: Create the English user guide**

~~~~markdown
# qubit-value User Guide

## Dependency

```toml
qubit-value = { version = "0.10", features = ["all"] }
```

The default feature set is empty. Enable only `chrono`, `big-number`, `url`,
`json`, or `converter` when the application does not need all families.

## Runtime shapes

`Value` stores one typed scalar. `MultiValues` stores one homogeneous typed
collection. `ValueContainer` preserves an explicit `Scalar` or `Collection`
shape; a one-item collection never becomes a scalar. `Unset(DataType)` is
different from a concrete value and from a concrete empty collection.

## Type-preserving Wire V1

Direct Serde uses `ValueWireV1`:

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"int32"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

`Value` accepts scalar only, `MultiValues` accepts collection only, and
`ValueContainer` accepts either. The envelope requires numeric version `1` and
rejects unknown fields, unknown types, wrong shapes, and all pre-0.10 payloads.
Wide integers and big numbers use canonical decimal strings. `Duration` uses
secs/nanos. Non-finite floats are rejected. `Json(null)` is concrete and
distinct from `Unset(Json)`.

Owned adapters are available through `From<Value>`, `From<MultiValues>`, and
`From<ValueContainer>` for `ValueWireV1`, and `From<ValueWireV1>` for
`ValueContainer`.

## Natural JSON

With `converter`, `to_json_value()` emits ordinary application JSON without
runtime type tags. Use Wire V1 whenever the receiver must reconstruct the
exact runtime type and shape.
~~~~

- [ ] **Step 6: Create the Chinese user guide**

~~~~markdown
# qubit-value 用户指南

## 依赖

```toml
qubit-value = { version = "0.10", features = ["all"] }
```

默认 feature 集为空。不需要全部类型族时，只启用 `chrono`、`big-number`、
`url`、`json` 或 `converter`。

## 运行时形态

`Value` 保存一个带类型标量，`MultiValues` 保存一个同类型集合。
`ValueContainer` 显式保留 `Scalar` 或 `Collection`；单元素集合不会变成标量。
`Unset(DataType)` 与具体值、具体空集合均不同。

## 保留类型的 Wire V1

直接 Serde 使用 `ValueWireV1`：

```json
{"version":1,"value":{"scalar":{"int32":42}}}
{"version":1,"value":{"scalar":{"unset":"int32"}}}
{"version":1,"value":{"collection":{"int32":[1,2]}}}
{"version":1,"value":{"collection":{"int32":[]}}}
{"version":1,"value":{"collection":{"unset":"int32"}}}
```

`Value` 只接受 scalar，`MultiValues` 只接受 collection，`ValueContainer`
接受两者。信封必须包含数字版本 `1`；未知字段、未知类型、错误 shape 和所有
0.10 之前的 payload 都会被拒绝。宽整数和大数使用 canonical 十进制字符串；
`Duration` 使用 secs/nanos；非有限浮点会被拒绝。`Json(null)` 与
`Unset(Json)` 不同。

`Value`、`MultiValues`、`ValueContainer` 可通过 `From` 转成
`ValueWireV1`；`ValueWireV1` 可转回 `ValueContainer`。

## 自然 JSON

启用 `converter` 后，`to_json_value()` 生成不含运行时类型标签的普通业务
JSON。如果接收方必须恢复精确的数据类型和形态，应使用 Wire V1。
~~~~

- [ ] **Step 7: Refresh lockfile and validate package/docs**

```bash
cargo +1.94.0 check --all-features
cargo +1.94.0 test --all-features --doc
RUSTDOCFLAGS="-D warnings" cargo +1.94.0 doc --all-features --no-deps
python3 .rs-ci/readme-version-check.py
cargo +1.94.0 package --allow-dirty --list | rg '^(doc/user_guide|README|src/value_wire.rs)'
```

Expected: PASS; `Cargo.lock` records `qubit-value 0.10.0`, and the package list contains both guides, READMEs and `src/value_wire.rs`.

---

### Task 6: Update rs-config to consume qubit-value 0.10

**Files:**
- Modify: `rs-config/Cargo.toml`, `rs-config/Cargo.lock`
- Verify: `rs-config/src/config_error.rs`, `rs-config/tests/config_error_tests.rs`

**Interfaces:**
- Consumes: `qubit-value 0.10` and non-exhaustive `ValueError`.
- Preserves: `rs-config` package version and generic read/convert façade.

- [ ] **Step 1: Update both dependency requirements**

```toml
qubit-value = { path = "../rs-value", version = "0.10", default-features = false, features = ["converter", "json"] }
```

```toml
qubit-value = { path = "../rs-value", version = "0.10", default-features = false, features = ["all"] }
```

- [ ] **Step 2: Refresh and test rs-config**

```bash
cd /home/starfish/working/qubit/rust-common/rs-config
cargo +1.94.0 check --all-features
cargo +1.94.0 test --all-features --test config_error_tests
cargo +1.94.0 test --all-features --test config_tests value_error
cargo +1.94.0 test --all-features
./cargo-feature-check.sh run-all
```

Expected: PASS; `Cargo.lock` resolves local `qubit-value 0.10.0`.

- [ ] **Step 3: Review rs-config scope**

```bash
git diff --check
git diff -- Cargo.toml Cargo.lock src/config_error.rs tests/config_error_tests.rs
git diff -U0 -- src/config.rs src/property.rs
```

Expected: the last command has no output; no generic bounds, conversion options, source parsing or Config API signatures changed.

---

### Task 7: Update rs-metadata's embedded value wire

**Files:**
- Modify: `rs-metadata/Cargo.toml`, `rs-metadata/Cargo.lock`
- Modify: `rs-metadata/tests/filter/wire/condition_wire_tests.rs`
- Modify: `rs-metadata/tests/filter/wire/filter_expr_wire_tests.rs`
- Modify: `rs-metadata/tests/filter/wire/metadata_filter_serde_tests.rs`

**Interfaces:**
- Consumes: `qubit-value 0.10` direct Serde.
- Preserves: MetadataFilter's own version `2`, operator tags, comparison and schema behavior.

- [ ] **Step 1: Update the dependency**

```toml
qubit-value = { path = "../rs-value", version = "0.10", features = ["all"] }
```

- [ ] **Step 2: Update Condition golden values**

Replace the three exact inner values with:

```rust
"value": {"version": 1, "value": {"scalar": {"float64": 1.5}}}
```

```rust
"value": {
    "version": 1,
    "value": {"scalar": {"int128": i128::MIN.to_string()}},
}
```

```rust
"value": {
    "version": 1,
    "value": {"scalar": {"uint128": u128::MAX.to_string()}},
}
```

- [ ] **Step 3: Update filter tree and MetadataFilter golden values**

Replace each old String/Int64 payload in `filter_expr_wire_tests.rs` and the active golden in `metadata_filter_serde_tests.rs` with:

```rust
"value": {
    "version": 1,
    "value": {"scalar": {"string": "active"}},
}
```

```rust
"value": {
    "version": 1,
    "value": {"scalar": {"int64": 10}},
}
```

Use the same V1 values inside the legacy-private-expression rejection fixture, so that test continues to isolate legacy `FilterExpr` shape rather than legacy `Value` parsing.

- [ ] **Step 4: Refresh and run metadata tests**

```bash
cd /home/starfish/working/qubit/rust-common/rs-metadata
cargo +1.94.0 check
cargo +1.94.0 test --test filter_tests condition_wire
cargo +1.94.0 test --test filter_tests filter_expr_wire
cargo +1.94.0 test --test filter_tests filter_serde
cargo +1.94.0 test
! rg -n '"(String|Int64|Int128|UInt128|Float64)"\s*:' tests/filter/wire
```

Expected: tests PASS, the search returns no old value-wire keys, `Cargo.lock` resolves `qubit-value 0.10.0`, and MetadataFilter remains version 2 outside Value V1.

---

### Task 8: Align, run full CI/coverage, and audit final scope

**Files:**
- Potentially normalize: Rust files touched by each repository's `./align-ci.sh`
- Verify: complete diffs in `rs-value`, `rs-config`, `rs-metadata`

**Interfaces:**
- Consumes: all prior task deliverables.
- Produces: project-formatted, CI-clean, coverage-checked changes ready for review.

- [ ] **Step 1: Align and verify rs-value**

```bash
cd /home/starfish/working/qubit/rust-common/rs-value
./align-ci.sh
./cargo-feature-check.sh run-all
./ci-check.sh
```

Expected: PASS. `ci-check` includes all-feature tests, docs, package verification, JSON coverage thresholds and audit. Do not invoke `cargo fmt`.

- [ ] **Step 2: Align and verify rs-config**

```bash
cd /home/starfish/working/qubit/rust-common/rs-config
./align-ci.sh
cargo +1.94.0 test --all-features
./cargo-feature-check.sh run-all
./ci-check.sh
```

Expected: PASS.

- [ ] **Step 3: Align and verify rs-metadata**

```bash
cd /home/starfish/working/qubit/rust-common/rs-metadata
./align-ci.sh
cargo +1.94.0 test
./ci-check.sh
```

Expected: PASS.

- [ ] **Step 4: Inspect all diffs and whitespace**

```bash
git -C /home/starfish/working/qubit/rust-common/rs-value diff --check
git -C /home/starfish/working/qubit/rust-common/rs-config diff --check
git -C /home/starfish/working/qubit/rust-common/rs-metadata diff --check
git -C /home/starfish/working/qubit/rust-common/rs-value diff --stat
git -C /home/starfish/working/qubit/rust-common/rs-config diff --stat
git -C /home/starfish/working/qubit/rust-common/rs-metadata diff --stat
```

Expected: no whitespace errors; only approved source, tests, manifests, lockfiles, docs and authorized align-ci normalization changed.

- [ ] **Step 5: Verify the deferred generic façade stayed untouched**

```bash
git -C /home/starfish/working/qubit/rust-common/rs-value diff -U0 -- src/value/value.rs src/multi_values/multi_values_core.rs src/value_container.rs | rg '^[+-].*(pub fn (get|to)|where|TryFrom|DataConvertTo)'
git -C /home/starfish/working/qubit/rust-common/rs-config diff -U0 -- src/config.rs src/property.rs
```

Expected: rs-value output contains only the approved `get_or/get_first_or` body changes and macro-table signature cleanup, not new generic bounds; rs-config output is empty.

- [ ] **Step 6: Prepare the completion report**

Report the getter fix, three non-exhaustive boundaries, exact V1 envelope and deliberate legacy rejection, downstream changes, version updates, every verification result, authorized align normalization, and absence of commits. Re-raise the deferred fourth issue as a separate generic read/convert target-trait façade proposal without implementing it here.
