//! Nvim-style editor interface for Builtin Plugins.
//!
//! Plugins talk to the host through handles and functions here. They do not
//! import engine crates. The host installs [`AbiVoltHost`] at load time.

use std::{
    collections::BTreeMap,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use abi_stable::{
    StableAbi,
    std_types::{ROption, RString, RVec},
};
use serde_json::Value;

pub mod buf;
pub mod hook;
pub mod lsp;
pub mod ui;

/// Integer buffer handle. Stable for the life of the open buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufHandle(pub u64);

/// Language-server id as declared by [`crate::LanguageServerSpec`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientId(pub String);

/// One location returned by an LSP jump request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub uri: String,
    pub line: u32,
    pub column: u32,
}

/// Host function table installed into the User Library (including cdylib copies).
#[repr(C)]
#[derive(Debug, Clone, Copy, StableAbi)]
pub struct AbiVoltHost {
    pub buf_current: extern "C" fn() -> u64,
    pub buf_name: extern "C" fn(u64) -> ROption<RString>,
    pub buf_uri: extern "C" fn(u64) -> ROption<RString>,
    pub buf_cursor_line: extern "C" fn(u64) -> u64,
    pub buf_cursor_column: extern "C" fn(u64) -> u64,
    pub buf_line_count: extern "C" fn(u64) -> u64,
    pub buf_get_lines: extern "C" fn(u64, u64, u64) -> RVec<RString>,
    pub lsp_client_ids: extern "C" fn(u64) -> RVec<RString>,
    pub lsp_request: extern "C" fn(RString, RString, RString, u64),
    pub ui_open_locations: extern "C" fn(RString, RString),
}

type VoltCallback = Box<dyn FnOnce(Result<Value, String>) + Send>;

struct VoltRuntime {
    host: AbiVoltHost,
    callbacks: BTreeMap<u64, VoltCallback>,
}

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);
static RUNTIME: OnceLock<Mutex<Option<VoltRuntime>>> = OnceLock::new();

fn runtime_lock() -> std::sync::MutexGuard<'static, Option<VoltRuntime>> {
    RUNTIME
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Installs the host function table. The host calls this after loading the User Library.
pub fn install_host(host: AbiVoltHost) {
    let mut slot = runtime_lock();
    *slot = Some(VoltRuntime {
        host,
        callbacks: BTreeMap::new(),
    });
}

/// C ABI entry used by the User Library module.
pub extern "C" fn install_host_c(host: AbiVoltHost) {
    install_host(host);
}

/// Completes an async LSP request posted by the host.
pub extern "C" fn dispatch_c(token: u64, payload: RString) {
    dispatch(token, payload.as_str());
}

pub(crate) fn with_host<T>(call: impl FnOnce(&AbiVoltHost) -> T) -> Option<T> {
    let host = runtime_lock().as_ref().map(|runtime| runtime.host)?;
    Some(call(&host))
}

pub(crate) fn take_callback(token: u64) -> Option<VoltCallback> {
    runtime_lock()
        .as_mut()
        .and_then(|runtime| runtime.callbacks.remove(&token))
}

pub(crate) fn store_callback(callback: VoltCallback) -> u64 {
    let token = NEXT_TOKEN.fetch_add(1, Ordering::AcqRel).max(1);
    if let Some(runtime) = runtime_lock().as_mut() {
        runtime.callbacks.insert(token, callback);
    }
    token
}

pub(crate) fn dispatch(token: u64, payload: &str) {
    let Some(callback) = take_callback(token) else {
        return;
    };
    if let Some(error) = payload.strip_prefix("err:") {
        callback(Err(error.to_owned()));
        return;
    }
    match serde_json::from_str::<Value>(payload) {
        Ok(value) => callback(Ok(value)),
        Err(error) => callback(Err(error.to_string())),
    }
}
