mod connect;
mod parse;

use futures_core::future::BoxFuture;
use rbdc::Error;
use rbdc::db::{ConnectOptions, Connection};
use serde::{Deserialize, Serialize};

/// Connection options for Oracle.
///
/// Prefer `OracleConnectOptions::new()` / `Default::default()` plus the builder
/// methods instead of direct struct literals, so new configuration fields can be
/// added without breaking downstream code.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct OracleConnectOptions {
    pub username: String,
    pub password: String,
    pub connect_string: String,
    #[serde(default = "OracleConnectOptions::default_statement_cache_capacity")]
    pub statement_cache_capacity: usize,
    #[serde(default = "OracleConnectOptions::default_row_channel_size")]
    pub row_channel_size: usize,
    #[serde(default = "OracleConnectOptions::default_command_channel_size")]
    pub command_channel_size: usize,
}

impl Default for OracleConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl OracleConnectOptions {
    pub fn new() -> Self {
        Self {
            username: "scott".to_owned(),
            password: "tiger".to_owned(),
            connect_string: "//localhost/XE".to_owned(),
            statement_cache_capacity: Self::default_statement_cache_capacity(),
            row_channel_size: Self::default_row_channel_size(),
            command_channel_size: Self::default_command_channel_size(),
        }
    }

    pub fn with_credentials(
        username: impl Into<String>,
        password: impl Into<String>,
        connect_string: impl Into<String>,
    ) -> Self {
        Self::new()
            .username(username)
            .password(password)
            .connect_string(connect_string)
    }

    fn default_statement_cache_capacity() -> usize {
        100
    }

    fn default_row_channel_size() -> usize {
        50
    }

    fn default_command_channel_size() -> usize {
        50
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = password.into();
        self
    }

    pub fn connect_string(mut self, connect_string: impl Into<String>) -> Self {
        self.connect_string = connect_string.into();
        self
    }

    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }

    pub fn row_channel_size(mut self, size: usize) -> Self {
        self.row_channel_size = size;
        self
    }

    pub fn command_channel_size(mut self, size: usize) -> Self {
        self.command_channel_size = size;
        self
    }
}

impl ConnectOptions for OracleConnectOptions {
    fn connect(&self) -> BoxFuture<'_, Result<Box<dyn Connection>, Error>> {
        Box::pin(async move {
            let connection = self.connect().await?;
            Ok(Box::new(connection) as Box<dyn Connection>)
        })
    }

    fn set_uri(&mut self, uri: &str) -> Result<(), Error> {
        *self = uri.parse()?;
        Ok(())
    }
}
