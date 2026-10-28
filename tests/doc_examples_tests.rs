// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Extracts and compiles the Rust examples published in the bilingual Markdown.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

/// A Rust example extracted verbatim from one Markdown document.
#[derive(Debug, Eq, Hash, PartialEq)]
struct MarkdownExample {
    id: String,
    mode: ExampleMode,
    source: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ExampleMode {
    Compile,
    Run,
}

impl ExampleMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "compile" => Some(Self::Compile),
            "run" => Some(Self::Run),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Run => "run",
        }
    }
}

/// Produces a diagnostic that always identifies the document and example.
fn diagnostic(file: &str, id: &str, message: &str) -> String {
    format!("{file}: example `{id}`: {message}")
}

/// Extracts fenced Rust fragments carrying an explicit compile or run marker.
fn extract_examples(file: &str, document: &str) -> Result<BTreeMap<String, MarkdownExample>, String> {
    let lines: Vec<&str> = document.lines().collect();
    let mut examples = BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index].trim();
        if !line.starts_with("<!-- example:") {
            index += 1;
            continue;
        }
        if !line.ends_with("-->") {
            return Err(diagnostic(file, "<missing>", "malformed example marker"));
        }
        let marker = line
            .strip_prefix("<!-- example:")
            .and_then(|value| value.strip_suffix("-->"))
            .expect("validated marker delimiters")
            .trim();
        let fields: Vec<&str> = marker.split_whitespace().collect();
        let mode = fields.get(1).and_then(|value| ExampleMode::parse(value));
        if fields.len() != 2 || mode.is_none() || fields[0].is_empty() {
            let id = fields.first().copied().filter(|id| *id != "compile" && *id != "run");
            return Err(diagnostic(
                file,
                id.unwrap_or("<missing>"),
                "expected `<!-- example:<id> compile -->` or `<!-- example:<id> run -->`",
            ));
        }
        let id = fields[0];
        if !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(diagnostic(
                file,
                id,
                "example ID must contain only lowercase ASCII letters, digits, or hyphens",
            ));
        }
        index += 1;
        if lines.get(index).map(|line| line.trim()) != Some("```rust") {
            return Err(diagnostic(
                file,
                id,
                "example marker must be followed immediately by a Rust fence",
            ));
        }
        index += 1;
        let source_start = index;
        while index < lines.len() && lines[index].trim() != "```" {
            index += 1;
        }
        if index == lines.len() {
            return Err(diagnostic(file, id, "Rust fence is not closed"));
        }
        let source = lines[source_start..index].join("\n");
        if source.trim().is_empty() {
            return Err(diagnostic(file, id, "example has no Rust source"));
        }
        let example = MarkdownExample {
            id: id.to_owned(),
            mode: mode.expect("validated example mode"),
            source,
        };
        if examples.insert(id.to_owned(), example).is_some() {
            return Err(diagnostic(file, id, "duplicate example ID"));
        }
        index += 1;
    }
    Ok(examples)
}

/// Verifies that two language variants publish the same stable example IDs.
fn ensure_matching_ids(
    english_file: &str,
    english: &BTreeMap<String, MarkdownExample>,
    chinese_file: &str,
    chinese: &BTreeMap<String, MarkdownExample>,
) -> Result<(), String> {
    let english_ids: BTreeSet<&str> = english.keys().map(String::as_str).collect();
    let chinese_ids: BTreeSet<&str> = chinese.keys().map(String::as_str).collect();
    if english_ids == chinese_ids {
        for id in english_ids {
            if english[id].mode != chinese[id].mode {
                return Err(format!(
                    "{english_file} and {chinese_file}: example `{id}` modes differ; English is `{}`, Chinese is `{}`",
                    english[id].mode.as_str(),
                    chinese[id].mode.as_str(),
                ));
            }
        }
        return Ok(());
    }
    let english_only: Vec<&str> = english_ids.difference(&chinese_ids).copied().collect();
    let chinese_only: Vec<&str> = chinese_ids.difference(&english_ids).copied().collect();
    Err(format!(
        "{english_file} and {chinese_file}: example ID sets differ; \
         English-only IDs: {english_only:?}; Chinese-only IDs: {chinese_only:?}"
    ))
}

/// Extracts the first literal Cargo dependency block from a document.
fn dependencies<'a>(file: &str, document: &'a str) -> Result<&'a str, String> {
    document
        .split("```toml\n")
        .find_map(|block| {
            let block = block.split("```").next()?;
            block.starts_with("[dependencies]\n").then_some(block)
        })
        .ok_or_else(|| diagnostic(file, "<dependencies>", "missing Cargo dependency block"))
}

/// Returns the shared target directory used by all generated example packages.
fn fixture_target(root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("markdown-examples")
}

/// Derives a content-addressed package name so Cargo cannot reuse stale output.
fn fixture_package_name(file: &str, dependencies: &str, example: &MarkdownExample) -> String {
    let mut hasher = DefaultHasher::new();
    file.hash(&mut hasher);
    dependencies.hash(&mut hasher);
    example.hash(&mut hasher);
    format!("documented-markdown-example-{:016x}", hasher.finish())
}

/// Extracts the unique executable emitted for the fixture package by Cargo.
fn executable_from_messages(messages: &str, package_name: &str) -> Result<PathBuf, String> {
    let mut executables = BTreeSet::new();
    for (index, line) in messages.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let message: serde_json::Value =
            serde_json::from_str(line).map_err(|error| format!("invalid Cargo JSON on line {}: {error}", index + 1))?;
        let is_artifact = message.get("reason").and_then(serde_json::Value::as_str) == Some("compiler-artifact");
        let target = message.get("target");
        let name_matches = target
            .and_then(|value| value.get("name"))
            .and_then(serde_json::Value::as_str)
            == Some(package_name);
        let is_bin = target
            .and_then(|value| value.get("kind"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|kinds| kinds.iter().any(|value| value.as_str() == Some("bin")));
        if is_artifact
            && name_matches
            && is_bin
            && let Some(path) = message.get("executable").and_then(serde_json::Value::as_str)
        {
            executables.insert(PathBuf::from(path));
        }
    }
    match executables.len() {
        0 => Err(format!("no executable artifact found for package `{package_name}`")),
        1 => Ok(executables.into_iter().next().expect("one executable")),
        count => Err(format!(
            "expected one executable artifact for package `{package_name}`, found {count}"
        )),
    }
}

fn read_diagnostic(path: &Path) -> String {
    let Ok(file) = File::open(path) else {
        return String::new();
    };
    let mut bytes = Vec::new();
    if file.take(64 * 1024).read_to_end(&mut bytes).is_err() {
        return String::new();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[test]
fn test_read_diagnostic_returns_ascii_and_chinese_text() {
    let directory = DiagnosticFixture::new();
    let path = directory.write("text.log", "ASCII".as_bytes());
    assert_eq!(read_diagnostic(&path), "ASCII");

    let path = directory.write("chinese.log", "中文路径".as_bytes());
    assert_eq!(read_diagnostic(&path), "中文路径");
}

#[test]
fn test_read_diagnostic_replaces_invalid_utf8() {
    let directory = DiagnosticFixture::new();
    let path = directory.write("invalid.log", &[0xff, b'a']);
    assert_eq!(read_diagnostic(&path), "\u{fffd}a");
}

#[test]
fn test_read_diagnostic_replaces_utf8_split_at_limit() {
    let directory = DiagnosticFixture::new();
    let mut bytes = vec![b'x'; 65_535];
    bytes.extend_from_slice("中".as_bytes());
    bytes.extend_from_slice(b"suffix");
    let path = directory.write("split.log", &bytes);
    let expected = format!("{}\u{fffd}", "x".repeat(65_535));
    let actual = read_diagnostic(&path);
    assert_eq!(actual, expected);
    assert!(!actual.contains("suffix"));
}

#[test]
fn test_read_diagnostic_reads_at_most_64_kibibytes() {
    let directory = DiagnosticFixture::new();
    let mut bytes = vec![b'x'; 65_536];
    bytes.extend_from_slice(b"suffix");
    let path = directory.write("large.log", &bytes);
    let actual = read_diagnostic(&path);
    assert_eq!(actual, "x".repeat(65_536));
    assert!(!actual.contains("suffix"));
}

#[test]
fn test_read_diagnostic_returns_empty_for_missing_file() {
    let directory = DiagnosticFixture::new();
    assert_eq!(read_diagnostic(&directory.path.join("missing.log")), "");
}

struct DiagnosticFixture {
    path: PathBuf,
}

impl DiagnosticFixture {
    fn new() -> Self {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("qubit-value-diagnostic-{}-{serial}", std::process::id(),));
        fs::create_dir(&path).expect("create diagnostic fixture directory");
        Self { path }
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, bytes).expect("write diagnostic fixture");
        path
    }
}

impl Drop for DiagnosticFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).expect("remove diagnostic fixture directory");
    }
}

fn cargo_artifact_message(package_name: &str, kind: &str, executable: Option<&str>, fresh: bool) -> String {
    let mut message = serde_json::json!({
        "reason": "compiler-artifact",
        "target": { "name": package_name, "kind": [kind] },
        "fresh": fresh,
    });
    if let Some(executable) = executable {
        message["executable"] = serde_json::Value::String(executable.to_owned());
    }
    message.to_string()
}

#[test]
fn test_artifact_parser_accepts_supported_executable_paths() {
    for path in [
        "target/debug/example",
        "target/x86_64-unknown-linux-gnu/debug/example",
        r"target\debug\example.exe",
    ] {
        let message = cargo_artifact_message("fixture", "bin", Some(path), true);
        assert_eq!(
            executable_from_messages(&message, "fixture").unwrap(),
            PathBuf::from(path),
        );
    }
}

#[test]
fn test_artifact_parser_ignores_blank_and_unrelated_messages() {
    let messages = format!(
        "\n{}\n{}\n{}\n",
        serde_json::json!({ "reason": "build-finished", "success": true }),
        cargo_artifact_message("dependency", "bin", Some("dependency"), false),
        cargo_artifact_message("fixture", "lib", Some("library"), false),
    );
    let error = executable_from_messages(&messages, "fixture").unwrap_err();
    assert!(error.contains("no executable artifact"), "{error}");
    assert!(error.contains("fixture"), "{error}");
}

#[test]
fn test_artifact_parser_requires_executable_for_matching_bin() {
    let message = cargo_artifact_message("fixture", "bin", None, false);
    let error = executable_from_messages(&message, "fixture").unwrap_err();
    assert!(error.contains("no executable artifact"), "{error}");
}

#[test]
fn test_artifact_parser_reports_invalid_json_line_number() {
    let error = executable_from_messages("{}\nnot-json\n", "fixture").unwrap_err();
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("invalid Cargo JSON"), "{error}");
}

#[test]
fn test_artifact_parser_deduplicates_identical_paths() {
    let message = cargo_artifact_message("fixture", "bin", Some("target/debug/example"), false);
    let messages = format!("{message}\n{message}");
    assert_eq!(
        executable_from_messages(&messages, "fixture").unwrap(),
        PathBuf::from("target/debug/example"),
    );
}

#[test]
fn test_artifact_parser_rejects_multiple_different_executables() {
    let messages = format!(
        "{}\n{}",
        cargo_artifact_message("fixture", "bin", Some("target/debug/one"), false),
        cargo_artifact_message("fixture", "bin", Some("target/debug/two"), false),
    );
    let error = executable_from_messages(&messages, "fixture").unwrap_err();
    assert!(error.contains("expected one executable artifact"), "{error}");
    assert!(error.contains("found 2"), "{error}");
}

fn run_process(
    mut command: std::process::Command,
    file: &str,
    id: &str,
    phase: &str,
    timeout: Duration,
    stdout_path: &Path,
    stderr_path: &Path,
) -> Result<(), String> {
    let stdout = fs::File::create(stdout_path)
        .map_err(|error| diagnostic(file, id, &format!("create {phase} stdout: {error}")))?;
    let stderr = fs::File::create(stderr_path)
        .map_err(|error| diagnostic(file, id, &format!("create {phase} stderr: {error}")))?;
    let mut child = command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| diagnostic(file, id, &format!("start {phase}: {error}")))?;
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                return Err(diagnostic(
                    file,
                    id,
                    &format!(
                        "{phase} failed with {status}:\nstdout:\n{}\nstderr:\n{}",
                        read_diagnostic(stdout_path),
                        read_diagnostic(stderr_path),
                    ),
                ));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(diagnostic(
                    file,
                    id,
                    &format!(
                        "{phase} timed out after {} seconds; stdout:\n{}\nstderr:\n{}",
                        timeout.as_secs(),
                        read_diagnostic(stdout_path),
                        read_diagnostic(stderr_path),
                    ),
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => return Err(diagnostic(file, id, &format!("poll {phase}: {error}"))),
        }
    }
}

/// Compiles one extracted fragment inside a complete downstream binary crate.
fn compile_example(
    root: &Path,
    target: &Path,
    file: &str,
    dependencies: &str,
    example: &MarkdownExample,
) -> Result<(), String> {
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
    let workspace = target.join(format!("fixture-{}-{serial}", std::process::id()));
    fs::create_dir_all(workspace.join("src"))
        .map_err(|error| diagnostic(file, &example.id, &format!("create fixture: {error}")))?;
    let sibling = root.parent().expect("crate parent").join("rs-datatype");
    let datatype_patch = if sibling.join("Cargo.toml").is_file() {
        format!("qubit-datatype = {{ path = {sibling:?} }}\n")
    } else {
        String::new()
    };
    let json_patch = root.parent().expect("crate parent").join("rs-json");
    let json_patch = if json_patch.join("Cargo.toml").is_file() {
        format!("qubit-json = {{ path = {:?} }}\n", json_patch)
    } else {
        String::new()
    };
    let budget_patch = root.parent().expect("crate parent").join("rs-budget");
    let budget_patch = if budget_patch.join("Cargo.toml").is_file() {
        format!("qubit-budget = {{ path = {:?} }}\n", budget_patch)
    } else {
        String::new()
    };
    let package_name = fixture_package_name(file, dependencies, example);
    let manifest = format!(
        r#"[package]
name = "{package_name}"
version = "0.0.0"
edition = "2024"

[workspace]

{dependencies}
[patch.crates-io]
qubit-value = {{ path = {root:?} }}
{datatype_patch}{json_patch}{budget_patch}"#,
    );
    fs::write(workspace.join("Cargo.toml"), manifest)
        .map_err(|error| diagnostic(file, &example.id, &format!("write fixture manifest: {error}")))?;
    let source = format!(
        "fn main() -> Result<(), Box<dyn std::error::Error>> {{\n{}\n    Ok(())\n}}\n",
        example.source
    );
    fs::write(workspace.join("src/main.rs"), source)
        .map_err(|error| diagnostic(file, &example.id, &format!("write fixture source: {error}")))?;
    let phase = match example.mode {
        ExampleMode::Compile => "compile",
        ExampleMode::Run => "build",
    };
    let stdout_path = workspace.join("stdout.log");
    let stderr_path = workspace.join("stderr.log");
    let mut command = Command::new(env!("CARGO"));
    command
        .args(if example.mode == ExampleMode::Compile {
            &["check", "--offline", "--quiet", "--manifest-path"][..]
        } else {
            &[
                "build",
                "--offline",
                "--quiet",
                "--message-format=json",
                "--manifest-path",
            ][..]
        })
        .arg(workspace.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target.join("build"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    let mut result = run_process(
        command,
        file,
        &example.id,
        phase,
        Duration::from_secs(180),
        &stdout_path,
        &stderr_path,
    );
    if result.is_ok() && example.mode == ExampleMode::Run {
        let messages = fs::read_to_string(&stdout_path)
            .map_err(|error| diagnostic(file, &example.id, &format!("read Cargo messages: {error}")));
        result = messages
            .and_then(|messages| executable_from_messages(&messages, &package_name))
            .map_err(|error| diagnostic(file, &example.id, &error))
            .and_then(|executable| {
                let mut command = Command::new(executable);
                command.env_remove("RUSTFLAGS").env_remove("CARGO_ENCODED_RUSTFLAGS");
                run_process(
                    command,
                    file,
                    &example.id,
                    "run",
                    Duration::from_secs(10),
                    &stdout_path,
                    &stderr_path,
                )
            });
    }
    let cleanup = fs::remove_dir_all(&workspace)
        .map_err(|error| diagnostic(file, &example.id, &format!("remove fixture: {error}")));
    result.and(cleanup)
}

#[test]
fn test_extract_rejects_marker_without_id() {
    let document = "<!-- example:compile -->\n```rust\nlet value = 1;\n```\n";
    let error = extract_examples("missing-id.md", document).unwrap_err();
    assert!(error.contains("missing-id.md"), "{error}");
    assert!(error.contains("<missing>"), "{error}");
}

#[test]
fn test_extract_rejects_duplicate_id() {
    let document = "<!-- example:value compile -->\n```rust\nlet value = 1;\n```\n\
                    <!-- example:value compile -->\n```rust\nlet value = 2;\n```\n";
    let error = extract_examples("duplicate.md", document).unwrap_err();
    assert!(error.contains("duplicate.md"), "{error}");
    assert!(error.contains("value"), "{error}");
    assert!(error.contains("duplicate"), "{error}");
}

#[test]
fn test_extract_rejects_unclosed_fence() {
    let document = "<!-- example:open compile -->\n```rust\nlet value = 1;\n";
    let error = extract_examples("unclosed.md", document).unwrap_err();
    assert!(error.contains("unclosed.md"), "{error}");
    assert!(error.contains("open"), "{error}");
    assert!(error.contains("not closed"), "{error}");
}

#[test]
fn test_extract_rejects_unknown_mode() {
    let document = "<!-- example:value execute -->\n```rust\nlet value = 1;\n```\n";
    let error = extract_examples("unknown-mode.md", document).unwrap_err();
    assert!(error.contains("unknown-mode.md"), "{error}");
    assert!(error.contains("value"), "{error}");
    assert!(error.contains("compile") && error.contains("run"), "{error}");
}

#[test]
fn test_bilingual_example_ids_must_match() {
    let english = extract_examples(
        "README.md",
        "<!-- example:english-only compile -->\n```rust\nlet value = 1;\n```\n",
    )
    .unwrap();
    let chinese = BTreeMap::new();
    let error = ensure_matching_ids("README.md", &english, "README.zh_CN.md", &chinese).unwrap_err();
    assert!(error.contains("README.md"), "{error}");
    assert!(error.contains("README.zh_CN.md"), "{error}");
    assert!(error.contains("english-only"), "{error}");
}

#[test]
fn test_compile_diagnostic_identifies_incomplete_example() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let document = fs::read_to_string(root.join("README.md")).expect("read README");
    let example = MarkdownExample {
        id: "incomplete".to_owned(),
        mode: ExampleMode::Compile,
        source: "let value = @;".to_owned(),
    };
    let error = compile_example(
        root,
        &fixture_target(root),
        "incomplete.md",
        dependencies("README.md", &document).unwrap(),
        &example,
    )
    .unwrap_err();
    assert!(error.contains("incomplete.md"), "{error}");
    assert!(error.contains("incomplete"), "{error}");
    assert!(error.contains("compile failed"), "{error}");
}

#[test]
fn test_run_diagnostic_reports_run_phase_and_exit_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let document = fs::read_to_string(root.join("README.md")).expect("read README");
    let example = MarkdownExample {
        id: "failing-run".to_owned(),
        mode: ExampleMode::Run,
        source: "assert_eq!(1, 2);".to_owned(),
    };
    let error = compile_example(
        root,
        &fixture_target(root),
        "failing-run.md",
        dependencies("README.md", &document).unwrap(),
        &example,
    )
    .unwrap_err();
    assert!(error.contains("failing-run.md"), "{error}");
    assert!(error.contains("failing-run"), "{error}");
    assert!(error.contains("run failed with"), "{error}");
}

#[test]
fn test_compile_mode_does_not_execute_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let document = fs::read_to_string(root.join("README.md")).expect("read README");
    let example = MarkdownExample {
        id: "compile-only".to_owned(),
        mode: ExampleMode::Compile,
        source: "panic!(\"must not execute in compile mode\");".to_owned(),
    };
    compile_example(
        root,
        &fixture_target(root),
        "compile-only.md",
        dependencies("README.md", &document).unwrap(),
        &example,
    )
    .unwrap();
}

#[test]
fn test_package_name_changes_when_example_source_changes() {
    let first = MarkdownExample {
        id: "same-id".to_owned(),
        mode: ExampleMode::Compile,
        source: "let value = 1;".to_owned(),
    };
    let second = MarkdownExample {
        id: "same-id".to_owned(),
        mode: ExampleMode::Compile,
        source: "let value = 2;".to_owned(),
    };
    assert_ne!(
        fixture_package_name("README.md", "[dependencies]\n", &first),
        fixture_package_name("README.md", "[dependencies]\n", &second),
    );
}

#[test]
fn test_bilingual_markdown_examples_compile_and_run_from_source() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pairs = [
        (
            "README.md",
            "README.zh_CN.md",
            &["conversion-policy", "quick-start"][..],
        ),
        (
            "doc/user_guide.md",
            "doc/user_guide.zh_CN.md",
            &[
                "borrowed-wire",
                "bounded-serializable",
                "conversion",
                "embedded-payload-budget",
                "multi-values",
                "named-values",
                "natural-json",
                "natural-json-map",
                "numeric-comparison",
                "runtime-config",
                "shared-decode-session",
                "single-value",
                "wire-rich-types",
                "wire-round-trip",
            ][..],
        ),
    ];
    let target = fixture_target(root);
    fs::create_dir_all(&target).expect("create Markdown example target");
    for (english_file, chinese_file, expected_ids) in pairs {
        let english_document = fs::read_to_string(root.join(english_file)).expect("read English document");
        let chinese_document = fs::read_to_string(root.join(chinese_file)).expect("read Chinese document");
        let english = extract_examples(english_file, &english_document).unwrap();
        let chinese = extract_examples(chinese_file, &chinese_document).unwrap();
        ensure_matching_ids(english_file, &english, chinese_file, &chinese).unwrap();
        let actual_ids: Vec<&str> = english.keys().map(String::as_str).collect();
        assert_eq!(actual_ids, expected_ids, "{english_file}: compile example IDs");
        for (file, document, examples) in [
            (english_file, &english_document, &english),
            (chinese_file, &chinese_document, &chinese),
        ] {
            let dependencies = dependencies(file, document).unwrap();
            for example in examples.values() {
                compile_example(root, &target, file, dependencies, example)
                    .unwrap_or_else(|error| panic!("{}: {error}", example.id));
            }
        }
    }
}
