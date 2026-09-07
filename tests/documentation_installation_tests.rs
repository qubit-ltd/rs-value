// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiles installation snippets as independent downstream packages.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Extracts the documented dependency block without substituting its versions.
fn dependencies(document: &str) -> &str {
    document
        .split("```toml\n")
        .find_map(|block| {
            let block = block.split("```").next().expect("code block body");
            block.starts_with("[dependencies]\n").then_some(block)
        })
        .expect("documented Cargo dependency block")
}

/// Uses each bilingual installation block verbatim in a separate Cargo package.
#[test]
fn test_documented_installation_compiles_downstream() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("documentation-installation");
    fs::create_dir_all(&target).expect("create isolated fixture target");
    let sibling = root.parent().expect("crate parent").join("rs-datatype");
    let datatype_patch = if sibling.join("Cargo.toml").is_file() {
        format!("qubit-datatype = {{ path = {sibling:?} }}\n")
    } else {
        // Published source packages resolve the documented datatype from the registry.
        String::new()
    };
    for (index, file) in [
        "README.md",
        "README.zh_CN.md",
        "doc/user_guide.md",
        "doc/user_guide.zh_CN.md",
    ]
    .iter()
    .enumerate()
    {
        let document = fs::read_to_string(root.join(file)).expect("read documentation");
        let workspace = target.join(format!("fixture-{}-{index}", std::process::id()));
        fs::create_dir(&workspace).expect("create fresh fixture directory");
        fs::create_dir(workspace.join("src")).expect("create fixture source directory");
        fs::copy(
            root.join("tests/fixtures/documentation_installation/src/main.rs"),
            workspace.join("src/main.rs"),
        )
        .expect("copy downstream source");
        let manifest = format!(
            r#"[package]
name = "documented-value-installation"
version = "0.0.0"
edition = "2024"

[workspace]

{}
[patch.crates-io]
qubit-value = {{ path = {:?} }}
{}
"#,
            dependencies(&document),
            root,
            datatype_patch
        );
        fs::write(workspace.join("Cargo.toml"), manifest).expect("write literal dependency fixture");
        let output = Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet", "--manifest-path"])
            .arg(workspace.join("Cargo.toml"))
            .arg("--target-dir")
            .arg(target.join("build"))
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .output()
            .expect("compile downstream fixture");
        assert!(
            output.status.success(),
            "{file}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        fs::remove_dir_all(workspace).expect("remove completed fixture inputs");
    }
}
