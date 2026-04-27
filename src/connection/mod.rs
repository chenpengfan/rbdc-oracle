use futures_core::future::BoxFuture;
use futures_util::future;
use rbdc::Error;
use rbdc::StatementCache;
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::Ordering;

pub(crate) use handle::ConnectionHandle;

use crate::OracleConnectOptions;
use crate::connection::establish::EstablishParams;
use crate::connection::worker::ConnectionWorker;
use crate::statement::VirtualStatement;

mod establish;
mod execute;
mod executor;
mod handle;
mod worker;

pub use worker::Command;

pub struct OracleConnection {
    pub(crate) worker: ConnectionWorker,
    pub(crate) row_channel_size: usize,
}

unsafe impl Sync for OracleConnection {}

pub struct ConnectionState {
    pub(crate) handle: ConnectionHandle,
    pub(crate) transaction_active: bool,
    pub(crate) statements: Statements,
}

pub(crate) struct Statements {
    cached: StatementCache<VirtualStatement>,
    temp: Option<VirtualStatement>,
}

impl OracleConnection {
    pub(crate) async fn establish(options: &OracleConnectOptions) -> Result<Self, Error> {
        let params = EstablishParams::from_options(options)?;
        let worker = ConnectionWorker::establish(params).await?;
        Ok(Self {
            worker,
            row_channel_size: options.row_channel_size,
        })
    }
}

impl Debug for OracleConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OracleConnection")
            .field("row_channel_size", &self.row_channel_size)
            .field("cached_statements_size", &self.cached_statements_size())
            .finish()
    }
}

impl OracleConnection {
    pub async fn do_close(&mut self) -> Result<(), Error> {
        self.worker.shutdown().await
    }

    pub fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(self.worker.ping())
    }

    pub fn cached_statements_size(&self) -> usize {
        self.worker
            .shared
            .cached_statements_size
            .load(Ordering::Acquire)
    }

    pub fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.worker.clear_cache().await?;
            Ok(())
        })
    }

    #[doc(hidden)]
    pub fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(future::ok(()))
    }

    #[doc(hidden)]
    pub fn should_flush(&self) -> bool {
        false
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        self.statements.clear();
    }
}

impl Statements {
    fn new(capacity: usize) -> Self {
        Statements {
            cached: StatementCache::new(capacity),
            temp: None,
        }
    }

    fn get(&mut self, query: &str, persistent: bool) -> Result<&mut VirtualStatement, Error> {
        if !persistent || !self.cached.is_enabled() {
            return Ok(self.temp.insert(VirtualStatement::new(query, false)?));
        }

        let exists = self.cached.contains_key(query);

        if !exists {
            let statement = VirtualStatement::new(query, true)?;
            self.cached.insert(query, statement);
        }

        let statement = self.cached.get_mut(query).unwrap();
        if exists {
            statement.reset()?;
        }

        Ok(statement)
    }

    fn len(&self) -> usize {
        self.cached.len()
    }

    fn clear(&mut self) {
        self.cached.clear();
        self.temp = None;
    }
}
