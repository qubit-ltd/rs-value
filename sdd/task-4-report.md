# Task 4 Report: Compile bilingual Markdown examples

## Status

DONE

## Changes

- Registered `tests/doc_examples_tests.rs` explicitly because Cargo autotests are disabled.
- Replaced hand-copied API examples with a Markdown marker extractor and downstream compiler harness.
- Added stable compile IDs to 24 snippets across the English and Chinese README and user guide pairs.
- Enforced duplicate, malformed, unclosed, and bilingual-ID mismatch diagnostics with document and example identities.
- Compiled every marked fragment through an isolated temporary Cargo package using `cargo check --offline --quiet`, local crate patches, and a shared target directory.
- Made fixture package names content-addressed so Cargo cannot accept stale output for changed snippets.
- Preserved all four literal installation dependency blocks.

## Documentation defects found and corrected

- The named-value guide example did not constrain the generic result of `Value::get`; both languages now bind it explicitly as `u64`.
- The shared decode-session example inferred the resource quantity as `i32`; both languages now use `64usize * 1024`.

## TDD evidence

- RED: `cargo test --locked --all-features --test doc_examples_tests` failed because `README.md` exposed no compile example IDs while `conversion-policy` and `quick-start` were required.
- FIRST_CHECK: `cargo check --locked --all-features --test doc_examples_tests` passed after the extractor and target registration compiled.
- GREEN: `cargo test --locked --all-features --test doc_examples_tests --quiet` passed 7/7 tests.
- GREEN: `cargo test --locked --all-features --test documentation_installation_tests --quiet` passed 1/1 test.
- Hygiene: focused `git diff --check` passed; the pinned formatter was applied only to `tests/doc_examples_tests.rs` because other agents owned concurrent Rust changes.

## Self-review

- All Rust fences intended as runnable examples in the four documents are marked; no example was silently skipped.
- README dependency blocks remain byte-for-byte unchanged.
- English and Chinese example ID sets are asserted exactly for each document pair.
- No workflow, design-document, production API, or runtime dependency was changed.

## Concerns

None.
