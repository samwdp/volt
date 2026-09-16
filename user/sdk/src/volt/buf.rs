//! Buffer primitives. Mirrors `vim.api.nvim_buf_*` at a small depth.

use super::{BufHandle, with_host};

/// Returns the focused buffer, if the host has prepared a call context.
pub fn current() -> Option<BufHandle> {
    with_host(|host| {
        let id = (host.buf_current)();
        (id != 0).then_some(BufHandle(id))
    })
    .flatten()
}

/// Returns the on-disk path for `buf`, when the buffer is a file.
pub fn get_name(buf: BufHandle) -> Option<String> {
    with_host(|host| {
        (host.buf_name)(buf.0)
            .into_option()
            .map(|name| name.into_string())
    })
    .flatten()
}

/// Returns the LSP document URI for `buf`.
pub fn get_uri(buf: BufHandle) -> Option<String> {
    with_host(|host| {
        (host.buf_uri)(buf.0)
            .into_option()
            .map(|uri| uri.into_string())
    })
    .flatten()
}

/// Returns `(line, column)` in LSP positions (0-based, UTF-16 columns).
pub fn get_cursor(buf: BufHandle) -> Option<(u32, u32)> {
    with_host(|host| {
        let line = (host.buf_cursor_line)(buf.0);
        let column = (host.buf_cursor_column)(buf.0);
        u32::try_from(line)
            .ok()
            .and_then(|line| u32::try_from(column).ok().map(|column| (line, column)))
    })
    .flatten()
}

/// Returns the number of lines in `buf`.
pub fn line_count(buf: BufHandle) -> usize {
    with_host(|host| (host.buf_line_count)(buf.0) as usize).unwrap_or(0)
}

/// Returns lines in `[start, end)` (end-exclusive, like nvim).
pub fn get_lines(buf: BufHandle, start: usize, end: usize) -> Vec<String> {
    with_host(|host| {
        (host.buf_get_lines)(buf.0, start as u64, end as u64)
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    })
    .unwrap_or_default()
}
