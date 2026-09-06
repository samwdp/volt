#![allow(unused_imports)]
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::sync::Mutex;

use editor_plugin_api::{
    DbActionSpec, DbBrowserContext, DbBrowserItemContext, DbBrowserItemKind, DbBrowserItemSpec,
    DbBrowserKind,
};
use keyring_core::Entry;
use postgres::{Client as PostgresClient, NoTls, SimpleQueryMessage};
use rusqlite::{Connection as SqliteConnection, types::ValueRef as SqliteValueRef};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tiberius::{Client as SqlServerClient, Config as SqlServerConfig, Row as SqlServerRow};
use tokio::{net::TcpStream, runtime::Runtime};
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[allow(unused_imports)]
use crate::connection::*;
#[allow(unused_imports)]
use crate::engines::*;
#[allow(unused_imports)]
use crate::service::*;
#[allow(unused_imports)]
use crate::types::*;

pub(crate) trait SecretStore: Send + Sync {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String>;
    fn get_secret(&self, secret_ref: &str) -> Result<String, String>;
    fn delete_secret(&self, secret_ref: &str) -> Result<(), String>;
}

#[derive(Debug)]
pub(crate) struct OsSecretStore {
    pub(crate) service_name: String,
}

impl OsSecretStore {
    pub(crate) fn new(service_name: impl Into<String>) -> Result<Self, String> {
        let service_name = service_name.into();
        initialize_native_keyring()?;
        Ok(Self { service_name })
    }

    pub(crate) fn entry(&self, secret_ref: &str) -> Result<Entry, String> {
        Entry::new(&self.service_name, secret_ref).map_err(|error| error.to_string())
    }
}

impl SecretStore for OsSecretStore {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String> {
        self.entry(secret_ref)?
            .set_password(secret)
            .map_err(|error| error.to_string())
    }

    fn get_secret(&self, secret_ref: &str) -> Result<String, String> {
        self.entry(secret_ref)?
            .get_password()
            .map_err(|error| error.to_string())
    }

    fn delete_secret(&self, secret_ref: &str) -> Result<(), String> {
        self.entry(secret_ref)?
            .delete_credential()
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug)]
pub(crate) struct DisabledSecretStore {
    pub(crate) reason: String,
}

impl SecretStore for DisabledSecretStore {
    fn set_secret(&self, _: &str, _: &str) -> Result<(), String> {
        Err(self.reason.clone())
    }

    fn get_secret(&self, _: &str) -> Result<String, String> {
        Err(self.reason.clone())
    }

    fn delete_secret(&self, _: &str) -> Result<(), String> {
        Err(self.reason.clone())
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct InMemorySecretStore {
    pub(crate) secrets: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl SecretStore for InMemorySecretStore {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .insert(secret_ref.to_owned(), secret.to_owned());
        Ok(())
    }

    fn get_secret(&self, secret_ref: &str) -> Result<String, String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| format!("secret `{secret_ref}` is missing"))
    }

    fn delete_secret(&self, secret_ref: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .remove(secret_ref);
        Ok(())
    }
}
