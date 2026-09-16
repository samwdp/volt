//! Hook names. Packages still declare bindings; the host fires them.
//!
//! Runtime `vim.api.nvim_create_autocmd` registration is not on this seam yet.

pub use crate::lsp_hooks as lsp;
