#![allow(unused_imports)]
use super::super::*;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    error::Error,
    fmt,
    io::{self, BufRead, BufReader, Read, Write},
    mem,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use dap_types::{
    AttachRequestArguments, ContinueArguments, DisconnectArguments, EvaluateArguments,
    EvaluateArgumentsContext, InitializeRequestArguments, LaunchRequestArguments, NextArguments,
    PauseArguments, RestartArguments, ScopesArguments, SetBreakpointsArguments, Source,
    SourceBreakpoint, StackTraceArguments, StepInArguments, StepOutArguments, VariablesArguments,
    requests::{
        Attach, Continue, Disconnect, Evaluate, Initialize, Launch, Next, Pause,
        Request as DapRequest, Restart, Scopes, SetBreakpoints, StackTrace, StepIn, StepOut,
        Threads, Variables,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    BreakpointStore, BreakpointToggle, DebugAdapterRegistry, DebugAdapterSpec,
    DebugAdapterTransport, DebugConfiguration, DebugRequestKind, DebugSessionPlan,
    StoredBreakpoint,
};

#[allow(unused_imports)]
use super::session::*;
#[allow(unused_imports)]
use super::types::*;

/// One transport log entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DapLogEntry {
    pub(crate) adapter_id: String,
    pub(crate) direction: DapLogDirection,
    pub(crate) message: String,
}

/// Snapshot of recent DAP transport traffic.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DapLogSnapshot {
    pub(crate) entries: Vec<DapLogEntry>,
}

/// Ring buffer of DAP transport log entries.
#[derive(Debug)]
pub struct DapTransportLog {
    pub(crate) max_entries: usize,
    pub(crate) entries: Vec<DapLogEntry>,
}

impl DapTransportLog {
    pub(crate) fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, entry: DapLogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub(crate) fn snapshot(&self) -> DapLogSnapshot {
        DapLogSnapshot {
            entries: self.entries.clone(),
        }
    }
}

pub(crate) type TransportLog = Arc<Mutex<DapTransportLog>>;
