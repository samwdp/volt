use super::{path_separator, path_with_install_bins};
use crate::{ToolKind, bin_dir};

#[test]
fn path_with_install_bins_appends_once() {
    let lsp = bin_dir(ToolKind::LanguageServer);
    let dap = bin_dir(ToolKind::DebugAdapter);
    let first = path_with_install_bins("/usr/bin");
    assert!(first.contains(lsp.to_string_lossy().as_ref()));
    assert!(first.contains(dap.to_string_lossy().as_ref()));
    assert!(first.starts_with("/usr/bin"));
    let second = path_with_install_bins(&first);
    assert_eq!(first, second);
    let sep = path_separator();
    assert_eq!(
        second
            .split(sep)
            .filter(|entry| *entry == lsp.to_string_lossy())
            .count(),
        1
    );
}
