use crate::OracleConnectOptions;
use crate::connection::{ConnectionHandle, ConnectionState, Statements};
use oracle::Connection as RawOracleConnection;
use rbdc::Error;
use std::sync::atomic::{AtomicU64, Ordering};

static THREAD_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(crate) struct EstablishParams {
    username: String,
    password: String,
    connect_string: String,
    statement_cache_capacity: usize,
    pub(crate) thread_name: String,
    pub(crate) command_channel_size: usize,
}

impl EstablishParams {
    pub(crate) fn from_options(options: &OracleConnectOptions) -> Result<Self, Error> {
        Ok(Self {
            username: options.username.clone(),
            password: options.password.clone(),
            connect_string: options.connect_string.clone(),
            statement_cache_capacity: options.statement_cache_capacity,
            thread_name: format!(
                "rbdc-oracle-worker-{}",
                THREAD_ID.fetch_add(1, Ordering::AcqRel)
            ),
            command_channel_size: options.command_channel_size,
        })
    }

    pub(crate) fn establish(&self) -> Result<ConnectionState, Error> {
        let connection =
            RawOracleConnection::connect(&self.username, &self.password, &self.connect_string)
                .map_err(|e| Error::from(e.to_string()))?;

        Ok(ConnectionState {
            handle: ConnectionHandle::new(connection),
            transaction_active: false,
            statements: Statements::new(self.statement_cache_capacity),
        })
    }
}
