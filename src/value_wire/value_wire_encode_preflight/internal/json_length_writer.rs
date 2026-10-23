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

/// Counts bytes written through `Write` without retaining their contents.
///
/// The counter uses checked arithmetic; writes that would overflow return an
/// error and leave the previous count unchanged.
#[derive(Debug, Default)]
pub(crate) struct JsonLengthWriter {
    /// Number of bytes accepted by this sink.
    pub(crate) len: usize,
}

impl Write for JsonLengthWriter {
    /// Adds the input length to the byte count without storing the input.
    ///
    /// Returns an error if the updated count would exceed `usize::MAX`; in
    /// that case, the existing count is preserved.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("JSON length overflow"))?;
        Ok(bytes.len())
    }

    /// Completes a no-op flush because this sink retains no buffered data.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::JsonLengthWriter;

    #[test]
    fn test_json_length_writer_flush_preserves_count() {
        let mut writer = JsonLengthWriter::default();
        writer.write_all(b"value").expect("count bytes");
        writer.flush().expect("flush counting sink");

        assert_eq!(writer.len, 5);
    }

    #[test]
    fn test_json_length_writer_rejects_counter_overflow() {
        let mut writer = JsonLengthWriter { len: usize::MAX };
        let error = writer.write(b"x").expect_err("byte count must not wrap");

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert_eq!(error.to_string(), "JSON length overflow");
        assert_eq!(writer.len, usize::MAX);
    }
}
