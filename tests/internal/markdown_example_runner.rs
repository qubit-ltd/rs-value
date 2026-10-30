// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use super::super::ExampleMode;
use super::super::MarkdownExample;

const REQUIRED_EXTERN_NAMES: &[&str] = &[
    "bigdecimal",
    "chrono",
    "qubit_budget",
    "qubit_datatype",
    "qubit_json",
    "qubit_redact",
    "qubit_value",
    "serde",
    "serde_json",
    "url",
    "num_bigint",
];

pub(crate) fn run_markdown_examples(
    root: &Path,
    documents: &[(&str, BTreeMap<String, MarkdownExample>)],
) -> Result<(), String> {
    let total: usize = documents.iter().map(|(_, examples)| examples.len()).sum();
    if total == 0 {
        return Err("Markdown example execution received zero examples".to_owned());
    }
    let runner = Runner::build(root)?;
    check_runner_behavior(root, &runner)?;
    let mut executed = 0;
    for (document_index, (document, examples)) in documents.iter().enumerate() {
        for example in examples.values() {
            runner.run_one(root, document, example, document_index)?;
            executed += 1;
        }
    }
    if executed != total {
        return Err(format!(
            "Markdown example execution count mismatch: expected {total}, executed {executed}"
        ));
    }
    Ok(())
}

fn check_runner_behavior(root: &Path, runner: &Runner) -> Result<(), String> {
    let examples = [
        (
            "ok",
            ExampleMode::Run,
            "let value: i32 = \"42\".parse()?; assert_eq!(value, 42);",
        ),
        (
            "type-error",
            ExampleMode::Compile,
            "let value: i32 = \"not an integer\";",
        ),
        ("panic-run", ExampleMode::Run, "panic!(\"run mode must observe this\");"),
        (
            "panic-compile",
            ExampleMode::Compile,
            "panic!(\"compile mode must not run this\");",
        ),
    ];
    for (id, mode, source) in examples {
        let example = MarkdownExample {
            id: id.to_owned(),
            mode,
            source: source.to_owned(),
        };
        let result = runner.run_one(root, "synthetic.md", &example, 0);
        match id {
            "ok" | "panic-compile" => result?,
            "type-error" | "panic-run" => {
                let error = result.expect_err("synthetic failure should be reported");
                for context in ["synthetic.md", id, source] {
                    if !error.contains(context) {
                        return Err(format!("synthetic diagnostic omitted {context:?}: {error}"));
                    }
                }
                if !error.contains(" failed\n") || !error.contains("stderr:\n") {
                    return Err(format!("synthetic example did not reach rustdoc failure: {error}"));
                }
                runner.discard_expected_failure(&error)?;
            }
            _ => unreachable!(),
        }
    }
    Ok(())
}

pub(crate) struct Runner {
    rustdoc: OsString,
    externs: Vec<(String, PathBuf)>,
    dependency_paths: BTreeSet<PathBuf>,
}

impl Runner {
    pub(crate) fn build(root: &Path) -> Result<Self, String> {
        let manifest = root.join("Cargo.toml");
        let cargo = cargo_command();
        let metadata = Command::new(&cargo)
            .args([
                "metadata",
                "--manifest-path",
                manifest.to_str().ok_or("manifest path is not UTF-8")?,
                "--all-features",
                "--locked",
                "--offline",
                "--format-version",
                "1",
            ])
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to run cargo metadata: {error}"))?;
        if !metadata.status.success() {
            return Err(process_error("cargo metadata", &metadata));
        }
        let metadata_json: serde_json::Value = serde_json::from_slice(&metadata.stdout)
            .map_err(|error| format!("could not parse cargo metadata JSON: {error}"))?;
        let root_package_id = metadata_json["resolve"]["root"]
            .as_str()
            .ok_or("cargo metadata did not identify the root package")?;
        let packages = metadata_json["packages"]
            .as_array()
            .ok_or("cargo metadata did not return packages")?;
        let root_package = packages
            .iter()
            .find(|package| package["id"].as_str() == Some(root_package_id))
            .ok_or("root package missing from cargo metadata")?;
        let root_target = root_package["targets"]
            .as_array()
            .and_then(|targets| {
                targets.iter().find(|target| {
                    target["kind"]
                        .as_array()
                        .is_some_and(|kinds| kinds.iter().any(|kind| kind == "lib"))
                })
            })
            .ok_or("root library target missing from cargo metadata")?;
        let root_target_name = root_target["name"]
            .as_str()
            .ok_or("root library target has no name")?
            .to_owned();
        let direct_dependencies: BTreeSet<String> = metadata_json["resolve"]["nodes"]
            .as_array()
            .and_then(|nodes| nodes.iter().find(|node| node["id"] == root_package_id))
            .and_then(|node| node["deps"].as_array())
            .ok_or("root dependency graph missing from cargo metadata")?
            .iter()
            .filter_map(|dependency| dependency["pkg"].as_str().map(str::to_owned))
            .collect();
        let mut package_targets = BTreeMap::new();
        for package in packages {
            let Some(package_id) = package["id"].as_str() else {
                continue;
            };
            if package_id != root_package_id && !direct_dependencies.contains(package_id) {
                continue;
            }
            if let Some(targets) = package["targets"].as_array() {
                for target in targets {
                    if let (Some(target_name), Some(target_kind)) = (target["name"].as_str(), target["kind"].as_array())
                        && target_kind.iter().any(|kind| kind == "lib")
                    {
                        package_targets.insert((package_id.to_owned(), target_name.to_owned()), target_name.to_owned());
                    }
                }
            }
        }

        let build = Command::new(&cargo)
            .args([
                "build",
                "--manifest-path",
                manifest.to_str().ok_or("manifest path is not UTF-8")?,
                "--lib",
                "--all-features",
                "--locked",
                "--offline",
                "--message-format=json",
            ])
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to run cargo build: {error}"))?;
        if !build.status.success() {
            return Err(process_error("cargo build --lib --all-features", &build));
        }
        let mut artifacts: BTreeMap<(String, String), Vec<PathBuf>> = BTreeMap::new();
        for line in String::from_utf8_lossy(&build.stdout).lines() {
            let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if message["reason"] != "compiler-artifact" {
                continue;
            }
            let (Some(package_id), Some(target_name)) =
                (message["package_id"].as_str(), message["target"]["name"].as_str())
            else {
                continue;
            };
            if !package_targets.contains_key(&(package_id.to_owned(), target_name.to_owned())) {
                continue;
            }
            if let Some(filenames) = message["filenames"].as_array() {
                let rlibs: Vec<PathBuf> = filenames
                    .iter()
                    .filter_map(|filename| filename.as_str())
                    .filter(|filename| filename.ends_with(".rlib"))
                    .map(PathBuf::from)
                    .collect();
                if !rlibs.is_empty() {
                    artifacts.insert((package_id.to_owned(), target_name.to_owned()), rlibs);
                }
            }
        }

        let mut externs = Vec::new();
        let mut dependency_paths = BTreeSet::new();
        let mut needed = BTreeSet::new();
        needed.insert((root_package_id.to_owned(), root_target_name.clone()));
        for package in packages {
            let Some(package_id) = package["id"].as_str() else {
                continue;
            };
            if !direct_dependencies.contains(package_id) {
                continue;
            }
            if let Some(targets) = package["targets"].as_array() {
                for target in targets {
                    let (Some(name), Some(kinds)) = (target["name"].as_str(), target["kind"].as_array()) else {
                        continue;
                    };
                    if kinds.iter().any(|kind| kind == "lib") {
                        needed.insert((package_id.to_owned(), name.to_owned()));
                    }
                }
            }
        }
        for key in needed {
            let target_name = &key.1;
            let extern_name = target_name.clone();
            if key.0 != root_package_id && !REQUIRED_EXTERN_NAMES.contains(&extern_name.as_str()) {
                continue;
            }
            let files = artifacts
                .get(&key)
                .ok_or_else(|| format!("Cargo build JSON has no .rlib artifact for {target_name}"))?;
            if files.len() != 1 {
                return Err(format!(
                    "expected exactly one .rlib for {target_name}, found {}",
                    files.len()
                ));
            }
            let path = files[0]
                .canonicalize()
                .map_err(|error| format!("could not resolve artifact {}: {error}", files[0].display()))?;
            dependency_paths.insert(
                path.parent()
                    .ok_or_else(|| format!("artifact has no parent: {}", path.display()))?
                    .to_path_buf(),
            );
            externs.push((extern_name, path));
        }
        externs.sort_by(|left, right| left.0.cmp(&right.0));
        let rustdoc: OsString = std::env::var_os("RUSTDOC").unwrap_or_else(|| "rustdoc".into());
        Ok(Self {
            rustdoc,
            externs,
            dependency_paths,
        })
    }

    pub(crate) fn run_one(
        &self,
        root: &Path,
        document: &str,
        example: &MarkdownExample,
        document_index: usize,
    ) -> Result<(), String> {
        let mut temp = TempDir::create()?;
        let markdown = temp.path.join(format!("{document_index}-{}.md", example.id));
        let output_dir = temp.path.join("doctests");
        fs::create_dir(&output_dir).map_err(|error| {
            format!(
                "{}: could not create doctest output directory: {error}",
                markdown.display()
            )
        })?;
        let fence = match example.mode {
            ExampleMode::Run => "rust",
            ExampleMode::Compile => "rust,no_run",
        };
        let content = format!(
            "```{fence}\nfn main() -> Result<(), Box<dyn std::error::Error>> {{\n{}\n    Ok(())\n}}\n```\n",
            indent_source(&example.source),
        );
        fs::write(&markdown, content)
            .map_err(|error| format!("{}: could not write temporary Markdown: {error}", markdown.display()))?;
        let mut command = Command::new(&self.rustdoc);
        command
            .arg("--test")
            .arg("--edition=2024")
            .arg(&markdown)
            .arg("--out-dir")
            .arg(&output_dir)
            .arg("--test-run-directory")
            .arg(root)
            .env("TMPDIR", &output_dir)
            .current_dir(root);
        for (name, path) in &self.externs {
            command.arg("--extern").arg(format!("{name}={}", path.display()));
        }
        for path in &self.dependency_paths {
            command.arg("-L").arg(format!("dependency={}", path.display()));
        }
        let output = command.output().map_err(|error| {
            temp.preserve_on_drop = true;
            format!(
                "{}: example `{}` ({}) could not start rustdoc: {error}; source:\n{}\ntemporary output: {}",
                document,
                example.id,
                example.mode.as_str(),
                example.source,
                temp.path.display(),
            )
        })?;
        if !output.status.success() {
            temp.preserve_on_drop = true;
            return Err(format!(
                "{}: example `{}` ({}) failed\nsource:\n{}\nstdout:\n{}\nstderr:\n{}\ntemporary output: {}",
                document,
                example.id,
                example.mode.as_str(),
                example.source,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
                temp.path.display(),
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let expected_summary = match example.mode {
            ExampleMode::Run => "1 passed",
            ExampleMode::Compile => "1 passed",
        };
        if !stdout.contains(expected_summary) {
            temp.preserve_on_drop = true;
            return Err(format!(
                "{}: example `{}` ({}) did not report exactly one expected doctest; stdout:\n{}\nstderr:\n{}\nsource:\n{}\ntemporary output: {}",
                document,
                example.id,
                example.mode.as_str(),
                stdout,
                String::from_utf8_lossy(&output.stderr),
                example.source,
                temp.path.display(),
            ));
        }
        Ok(())
    }

    fn discard_expected_failure(&self, error: &str) -> Result<(), String> {
        let Some((_, directory)) = error.rsplit_once("\ntemporary output: ") else {
            return Err(format!("expected failure has no temporary output path: {error}"));
        };
        let directory = Path::new(directory);
        let expected_parent = std::env::temp_dir();
        let expected_prefix = format!("qubit-value-markdown-{}-", std::process::id());
        if directory.parent() != Some(expected_parent.as_path())
            || !directory
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&expected_prefix))
        {
            return Err(format!(
                "refusing to remove unexpected synthetic failure directory {}",
                directory.display()
            ));
        }
        fs::remove_dir_all(directory).map_err(|error| {
            format!(
                "could not remove expected synthetic failure directory {}: {error}",
                directory.display()
            )
        })
    }
}

fn indent_source(source: &str) -> String {
    source
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn process_error(name: &str, output: &Output) -> String {
    format!(
        "{name} exited with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn cargo_command() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

struct TempDir {
    path: PathBuf,
    preserve_on_drop: bool,
}

impl TempDir {
    fn create() -> Result<Self, String> {
        let parent = std::env::temp_dir();
        for attempt in 0..100_u32 {
            let path = parent.join(format!("qubit-value-markdown-{}-{attempt}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        preserve_on_drop: false,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "could not create temporary Markdown runner directory {}: {error}",
                        path.display()
                    ));
                }
            }
        }
        Err(format!(
            "could not allocate a unique temporary Markdown runner directory under {}",
            parent.display()
        ))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !self.preserve_on_drop {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
