/// Returns whether the Vim-style command line is enabled.
pub const fn enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::enabled;

    #[test]
    fn command_line_is_enabled_by_default() {
        assert!(enabled());
    }
}
