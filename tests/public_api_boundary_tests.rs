// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Public API Boundary Tests
//!
//! Verifies that external callers can use the generic APIs through exported
//! types and standard conversion traits.

use std::fs;
use std::path::PathBuf;
use std::process::{
    Command,
    Output,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    Value,
};

static NEXT_PROJECT_ID: AtomicUsize = AtomicUsize::new(0);

/// Compiles a temporary external consumer with every crate feature enabled.
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

/// Formats all compiler output for an assertion diagnostic.
fn cargo_diagnostics(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

/// Verifies that an external exhaustive match fails for a non-exhaustive enum.
fn assert_non_exhaustive_match_failure(output: &Output) {
    let diagnostics = cargo_diagnostics(output);
    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("non-exhaustive"),
        "{diagnostics}",
    );
}

#[test]
fn test_value_generic_api_uses_public_bounds() {
    let value = Value::new(42i32);

    let strict: i32 = value.get().unwrap();
    let converted: i64 = value.to().unwrap();

    assert_eq!(strict, 42);
    assert_eq!(converted, 42);

    let mut text = Value::Unset(DataType::String);
    text.set("hello");

    assert_eq!(text.get_string().unwrap(), "hello");
}

#[test]
fn test_multi_values_generic_api_uses_public_bounds() {
    let values = MultiValues::new(vec![1i32, 2, 3]);

    let all: Vec<i32> = values.get().unwrap();
    let first: i32 = values.get_first().unwrap();
    let converted_first: i64 = values.to().unwrap();
    let converted_all: Vec<i64> = values.to_list().unwrap();

    assert_eq!(all, vec![1, 2, 3]);
    assert_eq!(first, 1);
    assert_eq!(converted_first, 1);
    assert_eq!(converted_all, vec![1, 2, 3]);

    let mut values = MultiValues::Unset(DataType::Int32);
    values.set(vec![4i32, 5]);
    values.add(6i32).unwrap();
    values.add(&[7i32, 8][..]).unwrap();

    assert_eq!(values.get_int32s().unwrap(), &[4, 5, 6, 7, 8]);
}

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
