//! Size limits for document I/O and editor sync.

pub const MAX_READ_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_WRITE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_SYNC_TEXT_CHARS: usize = 8 * 1024 * 1024;

pub fn read_limit_error(size: usize) -> String {
    format!(
        "file too large to read in the viewer ({size} bytes; limit {MAX_READ_BYTES})",
        size = size,
        MAX_READ_BYTES = MAX_READ_BYTES
    )
}

pub fn write_limit_error(size: usize) -> String {
    format!(
        "payload too large to write ({size} bytes; limit {MAX_WRITE_BYTES})",
        size = size,
        MAX_WRITE_BYTES = MAX_WRITE_BYTES
    )
}

pub fn sync_limit_error(chars: usize) -> String {
    format!(
        "document too large to sync ({chars} characters; limit {MAX_SYNC_TEXT_CHARS})",
        chars = chars,
        MAX_SYNC_TEXT_CHARS = MAX_SYNC_TEXT_CHARS
    )
}
