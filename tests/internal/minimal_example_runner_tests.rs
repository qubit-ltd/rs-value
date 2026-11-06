// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Checks bilingual examples in isolated consumers with documented minimal
//! dependencies.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use toml::Table;

use super::super::ExampleMode;
use super::super::MarkdownExample;

struct Profile {
    id: &'static str,
    examples: [(&'static str, &'static str); 2],
    dependencies: &'static [&'static str],
}

const PROFILES: &[Profile] = &[
    Profile {
        id: "quick-start",
        examples: [("README.md", "quick-start"), ("README.zh_CN.md", "quick-start")],
        dependencies: &["qubit-value|0.13|converter|false", "qubit-datatype|0.15||false"],
    },
    Profile {
        id: "conversion",
        examples: [
            ("doc/user_guide.md", "conversion"),
            ("doc/user_guide.zh_CN.md", "conversion"),
        ],
        dependencies: &[
            "qubit-value|0.13|converter|false",
            "qubit-datatype|0.15|converter|false",
        ],
    },
    Profile {
        id: "natural-json",
        examples: [
            ("doc/user_guide.md", "natural-json"),
            ("doc/user_guide.zh_CN.md", "natural-json"),
        ],
        dependencies: &["qubit-value|0.13|natural-json|false", "qubit-datatype|0.15||false"],
    },
    Profile {
        id: "redacted-output",
        examples: [
            ("doc/user_guide.md", "redacted-output"),
            ("doc/user_guide.zh_CN.md", "redacted-output"),
        ],
        dependencies: &["qubit-value|0.13|redact|false", "qubit-redact|0.10||false"],
    },
];

/// Checks each documented dependency profile and retains failed fixtures for
/// diagnosis.
pub(crate) fn run_minimal_examples(
    root: &Path,
    documents: &[(&str, BTreeMap<String, MarkdownExample>)],
) -> Result<(), String> {
    let temp_root = temp_root();
    fs::create_dir_all(&temp_root).map_err(|error| format!("cannot create {}: {error}", temp_root.display()))?;
    let result: Result<(), String> = (|| {
        for profile in PROFILES {
            let declarations = profile_declarations(root, profile)?;
            let mut examples = Vec::new();
            for (document, example_id) in profile.examples {
                let extracted = documents
                    .iter()
                    .find(|(path, _)| *path == document)
                    .map(|(_, examples)| examples)
                    .ok_or_else(|| format!("{document}: minimal profile `{}` has no extracted document", profile.id))?;
                let example = extracted.get(example_id).ok_or_else(|| {
                    format!(
                        "{document}: minimal profile `{}` is missing example `{example_id}`",
                        profile.id
                    )
                })?;
                examples.push((document, example));
            }
            check_profile(root, &temp_root, profile.id, &examples, &declarations)?;
        }
        check_negative_examples(root, &temp_root)?;
        Ok(())
    })();
    if let Err(error) = result {
        // A failed fixture is intentionally retained for diagnosis.
        return Err(format!("{error}\nfixture retained at {}", temp_root.display()));
    }
    fs::remove_dir_all(&temp_root).map_err(|error| format!("cannot remove {}: {error}", temp_root.display()))?;
    Ok(())
}

/// Extracts matching bilingual dependency blocks and checks the required
/// versions and features.
fn profile_declarations(root: &Path, profile: &Profile) -> Result<String, String> {
    let mut blocks = Vec::new();
    for (document, _) in profile.examples {
        let text = fs::read_to_string(root.join(document)).map_err(|error| format!("{document}: {error}"))?;
        let block = marked_toml_block(&text, profile.id).ok_or_else(|| {
            format!(
                "{document}: missing marked TOML dependencies for profile `{}`",
                profile.id
            )
        })?;
        blocks.push(block);
    }
    let block = matching_blocks(profile.id, &blocks)?;
    validate_declarations(profile, &block)?;
    Ok(block)
}

/// Returns the shared dependency declaration, rejecting missing or divergent
/// language blocks.
fn matching_blocks(profile: &str, blocks: &[String]) -> Result<String, String> {
    let first = blocks.first().ok_or("profile has no language documents")?;
    if blocks.iter().any(|block| block != first) {
        return Err(format!("bilingual dependency blocks differ for profile `{profile}`"));
    }
    Ok(first.clone())
}

/// Extracts the first TOML fence following the dependency marker for one
/// profile.
#[must_use]
fn marked_toml_block(document: &str, profile: &str) -> Option<String> {
    let marker = format!("<!-- minimal-dependencies: {profile} -->");
    let tail = document.split_once(&marker)?.1;
    let fence = tail.find("```toml")? + "```toml".len();
    let body = tail[fence..].trim_start_matches(['\r', '\n']);
    Some(body.split_once("```")?.0.trim().to_owned())
}

/// Checks dependency declarations against the exact package, version, feature
/// and defaults contract.
fn validate_declarations(profile: &Profile, block: &str) -> Result<(), String> {
    let mut parsed = BTreeMap::new();
    let mut in_dependencies = false;
    for line in block.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line == "[dependencies]" {
            in_dependencies = true;
            continue;
        }
        if !in_dependencies {
            return Err(format!("profile `{}` TOML must begin with [dependencies]", profile.id));
        }
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| format!("profile `{}` has invalid dependency line `{line}`", profile.id))?;
        let (version, features, default_features) = parse_inline_dependency(value.trim())
            .ok_or_else(|| format!("profile `{}` has invalid inline dependency `{line}`", profile.id))?;
        parsed.insert(name.trim().to_owned(), (version, features, default_features));
    }
    let expected: BTreeMap<String, (String, Vec<String>, bool)> = profile
        .dependencies
        .iter()
        .map(|entry| {
            let mut fields = entry.split('|');
            let name = fields.next().unwrap().to_owned();
            let version = fields.next().unwrap().to_owned();
            let features = fields
                .next()
                .unwrap()
                .split(',')
                .filter(|feature| !feature.is_empty())
                .map(str::to_owned)
                .collect();
            let defaults = fields.next().unwrap() == "true";
            (name, (version, features, defaults))
        })
        .collect();
    if parsed != expected {
        return Err(format!(
            "profile `{}` dependencies differ from the required package/version/feature set; got {parsed:?}, expected {expected:?}",
            profile.id
        ));
    }
    Ok(())
}

/// Parses a TOML inline dependency table, rejecting unknown keys and invalid
/// field types.
#[must_use]
fn parse_inline_dependency(value: &str) -> Option<(String, Vec<String>, bool)> {
    let document = format!("dependency = {value}").parse::<Table>().ok()?;
    if document.len() != 1 {
        return None;
    }
    let table = document.get("dependency")?.as_table()?;
    if table
        .keys()
        .any(|key| !matches!(key.as_str(), "version" | "features" | "default-features"))
    {
        return None;
    }
    let version = table.get("version")?.as_str()?.to_owned();
    let features = match table.get("features") {
        None => Vec::new(),
        Some(value) => value
            .as_array()?
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()?,
    };
    let defaults = match table.get("default-features") {
        None => true,
        Some(value) => value.as_bool()?,
    };
    Some((version, features, defaults))
}

/// Checks one isolated consumer, returning compiler diagnostics for
/// negative-case assertions.
fn check_consumer(
    root: &Path,
    temp_root: &Path,
    profile: &str,
    document: &str,
    example: &MarkdownExample,
    declarations: &str,
) -> Result<(), String> {
    let bin = format!("{}_{}", profile.replace('-', "_"), language_suffix(document));
    let fixture = temp_root.join(profile).join(language_suffix(document));
    fs::create_dir_all(fixture.join("src/bin")).map_err(|error| error.to_string())?;
    let value_path = toml_string(root.to_str().ok_or("local qubit-value path is not UTF-8")?);
    let dependency_block = declarations.replace("qubit-value = {", &format!("qubit-value = {{ path = {value_path},"));
    let manifest = format!(
        "[package]\nname = \"minimal-{bin}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n{dependency_block}\n\n[workspace]\n"
    );
    fs::write(fixture.join("Cargo.toml"), manifest).map_err(|error| error.to_string())?;
    let source = format!(
        "fn main() -> Result<(), Box<dyn std::error::Error>> {{\n{}\nOk(())\n}}\n",
        example.source
    );
    fs::write(fixture.join("src/bin/main.rs"), source).map_err(|error| error.to_string())?;
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let target = fixture.join("target");
    let output = Command::new(&cargo)
        .args(["check", "--manifest-path"])
        .arg(fixture.join("Cargo.toml"))
        .args(["--bin", "main"])
        .env("CARGO_TARGET_DIR", &target)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!(
                "cannot run cargo for {document} example `{}` profile `{profile}`: {error}",
                example.id
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "{document}: example `{}` profile `{profile}` failed: cargo check --manifest-path {} --bin main\nstderr:\n{}",
            example.id,
            fixture.join("Cargo.toml").display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Checks both language examples together using only the documented dependency
/// profile.
fn check_profile(
    root: &Path,
    temp_root: &Path,
    profile: &str,
    examples: &[(&str, &MarkdownExample)],
    declarations: &str,
) -> Result<(), String> {
    let fixture = temp_root.join(profile);
    fs::create_dir_all(fixture.join("src/bin")).map_err(|error| error.to_string())?;
    let manifest = consumer_manifest(root, profile, declarations)?;
    fs::write(fixture.join("Cargo.toml"), manifest).map_err(|error| error.to_string())?;
    for (document, example) in examples {
        let bin = format!("{}_{}", profile.replace('-', "_"), language_suffix(document));
        let source = format!(
            "fn main() -> Result<(), Box<dyn std::error::Error>> {{\n{}\nOk(())\n}}\n",
            example.source
        );
        fs::write(fixture.join("src/bin").join(format!("{bin}.rs")), source).map_err(|error| error.to_string())?;
    }
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let target = fixture.join("target");
    let output = Command::new(&cargo)
        .args(["check", "--manifest-path"])
        .arg(fixture.join("Cargo.toml"))
        .arg("--bins")
        .env("CARGO_TARGET_DIR", &target)
        .current_dir(root)
        .output()
        .map_err(|error| format!("cannot run cargo for profile `{profile}`: {error}"))?;
    if !output.status.success() {
        let source_examples = examples
            .iter()
            .map(|(document, example)| format!("{document} example `{}`", example.id))
            .collect::<Vec<_>>();
        return Err(format!(
            "profile `{profile}` examples {source_examples:?} failed: cargo check --manifest-path {} --bins\nstderr:\n{}",
            fixture.join("Cargo.toml").display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

/// Builds an isolated consumer manifest with the candidate qubit-value source.
fn consumer_manifest(root: &Path, profile: &str, declarations: &str) -> Result<String, String> {
    let value_path = toml_string(root.to_str().ok_or("local qubit-value path is not UTF-8")?);
    let dependency_block = declarations.replace("qubit-value = {", &format!("qubit-value = {{ path = {value_path},"));
    let manifest = format!(
        "[package]\nname = \"minimal-{}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n{}\n\n[workspace]\n",
        profile.replace('-', "_"),
        dependency_block
    );
    Ok(manifest)
}

/// Confirms isolated consumers cannot use undeclared crates or disabled feature
/// APIs.
fn check_negative_examples(root: &Path, temp_root: &Path) -> Result<(), String> {
    const QUICK_START: &str = "[dependencies]\nqubit-value = { version = \"0.13\", default-features = false, features = [\"converter\"] }\nqubit-datatype = { version = \"0.15\", default-features = false }";
    for (id, source, expected) in [
        (
            "undeclared-crate",
            "let _: serde_json::Value = serde_json::Value::Null;",
            "serde_json",
        ),
        (
            "missing-feature",
            "let _ = qubit_value::Value::new(1).to_json_value()?;",
            "to_json_value",
        ),
    ] {
        let example = MarkdownExample {
            id: id.to_owned(),
            mode: ExampleMode::Run,
            source: source.to_owned(),
        };
        let error = check_consumer(root, temp_root, "quick-start", "README.md", &example, QUICK_START)
            .expect_err("negative isolated consumers must fail to compile");
        if !error.contains(expected) {
            return Err(format!(
                "negative example `{id}` failed for an unexpected reason: {error}"
            ));
        }
        let _ = fs::remove_dir_all(temp_root.join("quick-start").join("en"));
    }
    Ok(())
}

/// Maps a document path to the binary suffix used for its language.
#[must_use]
#[inline]
fn language_suffix(document: &str) -> &'static str {
    if document.ends_with("zh_CN.md") { "zh" } else { "en" }
}

/// Quotes a local path as a TOML basic string, escaping delimiters and control
/// characters.
#[must_use]
fn toml_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

/// Builds a process- and timestamp-qualified fixture path without creating it.
#[must_use]
fn temp_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("qubit-value-minimal-examples-{}-{nanos}", std::process::id()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_consumer_manifest_registry_dependencies() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let declarations = "[dependencies]\nqubit-value = { version = \"0.13\", default-features = false, features = [\"redact\"] }\nqubit-redact = { version = \"0.10\", default-features = false }";
        let manifest = super::consumer_manifest(root, "redacted-output", declarations).unwrap();
        let parsed = manifest.parse::<toml::Table>().unwrap();
        assert!(
            !parsed.contains_key("patch"),
            "manifest must not patch crates.io: {manifest}"
        );
        let dependencies = parsed["dependencies"].as_table().unwrap();
        let value = dependencies["qubit-value"].as_table().unwrap();
        assert_eq!(value["path"].as_str(), Some(root.to_str().unwrap()));
        let redact = dependencies["qubit-redact"].as_table().unwrap();
        assert_eq!(redact["version"].as_str(), Some("0.10"));
        assert_eq!(redact["default-features"].as_bool(), Some(false));
        assert!(!redact.contains_key("features"));
        assert!(!redact.contains_key("path"));
        assert!(!redact.contains_key("git"));
    }

    #[test]
    fn test_consumer_manifest_candidate_path() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let declarations = "[dependencies]\nqubit-value = { version = \"0.13\", default-features = false, features = [\"converter\"] }\nqubit-datatype = { version = \"0.15\", default-features = false }";
        let manifest = super::consumer_manifest(root, "quick-start", declarations).unwrap();
        let parsed = manifest.parse::<toml::Table>().unwrap();
        let dependencies = parsed["dependencies"].as_table().unwrap();
        let value = dependencies["qubit-value"].as_table().unwrap();
        assert_eq!(value["path"].as_str(), Some(root.to_str().unwrap()));
        assert_eq!(value["version"].as_str(), Some("0.13"));
        assert_eq!(value["default-features"].as_bool(), Some(false));
        assert_eq!(value["features"].as_array().unwrap()[0].as_str(), Some("converter"));
        assert!(!parsed.contains_key("patch"));
    }

    #[test]
    fn test_inline_dependency_table_cases() {
        let cases = [
            (
                "empty feature array",
                r#"{ version = "0.13", features = [] }"#,
                Some(("0.13".to_owned(), vec![], true)),
            ),
            (
                "single feature",
                r#"{ version = "0.13", features = ["converter"] }"#,
                Some(("0.13".to_owned(), vec!["converter".to_owned()], true)),
            ),
            (
                "reordered fields",
                r#"{ features = ["converter", "json"], default-features = false, version = "0.13" }"#,
                Some((
                    "0.13".to_owned(),
                    vec!["converter".to_owned(), "json".to_owned()],
                    false,
                )),
            ),
            (
                "default feature and default-features values",
                r#"{ version = "0.13" }"#,
                Some(("0.13".to_owned(), vec![], true)),
            ),
            (
                "comma in feature string",
                r#"{ version = "0.13", features = ["json,wire"] }"#,
                Some(("0.13".to_owned(), vec!["json,wire".to_owned()], true)),
            ),
            ("missing version", r#"{ features = [] }"#, None),
            ("non-string version", r#"{ version = 13 }"#, None),
            (
                "non-boolean default-features",
                r#"{ version = "0.13", default-features = "false" }"#,
                None,
            ),
            ("non-array features", r#"{ version = "0.13", features = "json" }"#, None),
            (
                "non-string feature",
                r#"{ version = "0.13", features = ["json", 1] }"#,
                None,
            ),
            ("unknown key", r#"{ version = "0.13", optional = true }"#, None),
            (
                "unterminated array",
                r#"{ version = "0.13", features = ["json" }"#,
                None,
            ),
        ];

        for (name, input, expected) in cases {
            assert_eq!(super::parse_inline_dependency(input), expected, "{name}");
        }
    }

    #[test]
    fn test_inline_dependency_multiple_features() {
        let actual = super::parse_inline_dependency(
            r#"{ version = "0.13", default-features = false, features = ["converter", "json"] }"#,
        );
        assert_eq!(
            actual,
            Some((
                "0.13".to_owned(),
                vec!["converter".to_owned(), "json".to_owned()],
                false,
            ))
        );
    }

    #[test]
    fn test_validate_declarations_multiple_features() {
        let profile = super::Profile {
            id: "multi-feature",
            examples: [("README.md", "example"), ("README.zh_CN.md", "example")],
            dependencies: &["qubit-value|0.13|converter,json|false"],
        };
        let valid = "[dependencies]\nqubit-value = { version = \"0.13\", default-features = false, features = [\"converter\", \"json\"] }";
        assert!(super::validate_declarations(&profile, valid).is_ok());

        let missing_feature = "[dependencies]\nqubit-value = { version = \"0.13\", default-features = false, features = [\"converter\"] }";
        let error = super::validate_declarations(&profile, missing_feature).unwrap_err();
        assert!(error.contains("dependencies differ"), "{error}");

        let wrong_version = "[dependencies]\nqubit-value = { version = \"0.14\", default-features = false, features = [\"converter\", \"json\"] }";
        let error = super::validate_declarations(&profile, wrong_version).unwrap_err();
        assert!(error.contains("dependencies differ"), "{error}");
    }

    #[test]
    fn test_minimal_example_profiles_reject_missing_and_different_blocks() {
        assert!(super::marked_toml_block("no marker", "quick-start").is_none());
        assert!(super::matching_blocks("quick-start", &["a".into(), "b".into()]).is_err());
        let profile = super::Profile {
            id: "quick-start",
            examples: [("README.md", "quick-start"), ("README.zh_CN.md", "quick-start")],
            dependencies: &["qubit-value|0.13|converter|false", "qubit-datatype|0.15||false"],
        };
        let result = super::profile_declarations(std::path::Path::new(env!("CARGO_MANIFEST_DIR")), &profile);
        assert!(result.is_ok(), "{result:?}");
    }
}
