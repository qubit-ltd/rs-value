// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! A counting sink used to measure JSON lexemes without retaining bytes.

use std::io;
use std::io::Write;

/// `Write` implementation that retains only the number of bytes written.
#[derive(Debug, Default)]
pub(crate) struct JsonLengthWriter {
    /// Number of bytes accepted by this sink.
    pub(crate) len: usize,
}

impl Write for JsonLengthWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("JSON length overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
