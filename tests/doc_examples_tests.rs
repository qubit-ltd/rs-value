// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Checks example markers published in the bilingual Markdown.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

mod internal;

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
fn test_bilingual_markdown_example_ids_are_stable() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
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
    for (english_file, chinese_file, expected_ids) in pairs {
        let english_document = std::fs::read_to_string(root.join(english_file)).expect("read English document");
        let chinese_document = std::fs::read_to_string(root.join(chinese_file)).expect("read Chinese document");
        let english = extract_examples(english_file, &english_document).unwrap();
        let chinese = extract_examples(chinese_file, &chinese_document).unwrap();
        ensure_matching_ids(english_file, &english, chinese_file, &chinese).unwrap();
        let actual_ids: Vec<&str> = english.keys().map(String::as_str).collect();
        assert_eq!(actual_ids, expected_ids, "{english_file}: example IDs");
    }
}

#[test]
fn test_bilingual_markdown_examples_execute() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let pairs = [
        ("README.md", "README.zh_CN.md"),
        ("doc/user_guide.md", "doc/user_guide.zh_CN.md"),
    ];
    let mut documents = Vec::new();
    for (english, chinese) in pairs {
        for path in [english, chinese] {
            let source = std::fs::read_to_string(root.join(path)).expect("read marked document");
            let examples = extract_examples(path, &source).unwrap();
            documents.push((path, examples));
        }
    }
    let extracted: usize = documents.iter().map(|(_, examples)| examples.len()).sum();
    assert_eq!(extracted, 32, "baseline marked-example count");
    internal::markdown_example_runner::run_markdown_examples(root, &documents).unwrap();
}
