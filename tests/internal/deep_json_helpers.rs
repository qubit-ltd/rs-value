// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Helpers for isolated deep JSON integration tests.

use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde_json::Value as JsonValue;

const CHILD_TIMEOUT: Duration = Duration::from_secs(30);
const DIAGNOSTIC_LIMIT: usize = 64 * 1024;

/// Builds nested single-element arrays; callers must avoid recursive
/// destruction at large depths.
#[must_use]
pub(crate) fn build_deep_json(depth: usize) -> JsonValue {
    (0..depth).fold(JsonValue::Bool(true), |value, _| JsonValue::Array(vec![value]))
}

/// Consumes a JSON tree iteratively to avoid overflowing the stack during
/// recursive destruction.
#[cfg(all(feature = "converter", feature = "json"))]
pub(crate) fn dismantle_json(value: JsonValue) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            JsonValue::Array(values) => pending.extend(values),
            JsonValue::Object(values) => pending.extend(values.into_values()),
            _ => {}
        }
    }
}

/// Runs one ignored test in a child process with the supplied environment
/// marker.
///
/// Captures bounded logs and panics after killing and reaping a child that
/// exceeds the timeout.
pub(crate) fn run_isolated_test(test_name: &str, marker: &str) -> Output {
    let directory = unique_temp_dir();
    fs::create_dir(&directory).expect("create isolated test log directory");
    let stdout_path = directory.join("stdout.log");
    let stderr_path = directory.join("stderr.log");
    let stdout = File::create(&stdout_path).expect("create child stdout log");
    let stderr = File::create(&stderr_path).expect("create child stderr log");

    let mut child = Command::new(env::current_exe().expect("locate integration test binary"))
        .arg("--exact")
        .arg(test_name)
        .arg("--ignored")
        .arg("--nocapture")
        .env(marker, "1")
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("spawn isolated deep JSON test");
    let status = match wait_with_timeout(&mut child, CHILD_TIMEOUT) {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let status = child.wait().expect("reap timed-out isolated test");
            let output = read_logs(&stdout_path, &stderr_path, status);
            let _ = fs::remove_dir_all(&directory);
            panic!(
                "isolated test {test_name} exceeded {CHILD_TIMEOUT:?}; status={:?}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
        }
    };
    let output = read_logs(&stdout_path, &stderr_path, status);
    fs::remove_dir_all(&directory).expect("remove isolated test log directory");
    output
}

/// Polls until the child exits or the deadline expires; the caller must reap a
/// timed-out child.
fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait().expect("poll isolated test child") {
            Some(status) => return Some(status),
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            None => return None,
        }
    }
}

/// Combines the child status with bounded stdout and stderr logs.
fn read_logs(stdout_path: &Path, stderr_path: &Path, status: ExitStatus) -> Output {
    Output {
        status,
        stdout: read_bounded(stdout_path),
        stderr: read_bounded(stderr_path),
    }
}

/// Reads at most the diagnostic limit, appending a marker when more bytes are
/// present.
fn read_bounded(path: &Path) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(DIAGNOSTIC_LIMIT + 1);
    File::open(path)
        .expect("open isolated test log")
        .take((DIAGNOSTIC_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .expect("read isolated test log");
    if bytes.len() <= DIAGNOSTIC_LIMIT {
        bytes
    } else {
        bytes.truncate(DIAGNOSTIC_LIMIT);
        let mut bounded = bytes;
        bounded.extend_from_slice(b"\n[diagnostic truncated]\n");
        bounded
    }
}

/// Reserves a process-local sequence number for a temporary log directory path
/// without creating it.
fn unique_temp_dir() -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!("qubit-value-deep-json-{}-{id}", process::id()))
}
