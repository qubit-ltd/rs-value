// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Helpers for isolated deep JSON integration tests.

use std::fs;
use std::fs::File;
use std::io::Read;
use std::process::Child;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde_json::Value as JsonValue;

const CHILD_TIMEOUT: Duration = Duration::from_secs(30);
const DIAGNOSTIC_LIMIT: usize = 64 * 1024;

pub(crate) fn build_deep_json(depth: usize) -> JsonValue {
    (0..depth).fold(JsonValue::Bool(true), |value, _| JsonValue::Array(vec![value]))
}

#[cfg(feature = "all")]
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

pub(crate) fn run_isolated_test(test_name: &str, marker: &str) -> std::process::Output {
    let directory = unique_temp_dir();
    fs::create_dir(&directory).expect("create isolated test log directory");
    let stdout_path = directory.join("stdout.log");
    let stderr_path = directory.join("stderr.log");
    let stdout = File::create(&stdout_path).expect("create child stdout log");
    let stderr = File::create(&stderr_path).expect("create child stderr log");

    let mut child = Command::new(std::env::current_exe().expect("locate integration test binary"))
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

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait().expect("poll isolated test child") {
            Some(status) => return Some(status),
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            None => return None,
        }
    }
}

fn read_logs(
    stdout_path: &std::path::Path,
    stderr_path: &std::path::Path,
    status: std::process::ExitStatus,
) -> std::process::Output {
    std::process::Output {
        status,
        stdout: read_bounded(stdout_path),
        stderr: read_bounded(stderr_path),
    }
}

fn read_bounded(path: &std::path::Path) -> Vec<u8> {
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

fn unique_temp_dir() -> std::path::PathBuf {
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("qubit-value-deep-json-{}-{id}", std::process::id()))
}
