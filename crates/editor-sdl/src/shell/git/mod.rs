#![allow(unused_imports)]

pub(crate) use std::collections::HashMap;
pub(crate) use std::process::Stdio;

use super::*;

mod commands;
mod commit;
mod diff;
mod fringe;
mod log;
mod merge_rebase;
mod pickers;
mod process;
mod remote;
mod staging;
mod stash;
mod status;
mod worktree;

pub(crate) use commands::*;
pub(crate) use commit::*;
pub(crate) use diff::*;
pub(crate) use fringe::*;
pub(crate) use log::*;
pub(crate) use merge_rebase::*;
pub(crate) use pickers::*;
pub(crate) use process::*;
pub(crate) use remote::*;
pub(crate) use staging::*;
pub(crate) use stash::*;
pub(crate) use status::*;
pub(crate) use worktree::*;

#[cfg(test)]
mod tests;
