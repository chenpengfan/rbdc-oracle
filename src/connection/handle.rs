use oracle::Connection as RawOracleConnection;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct ConnectionHandle(Arc<RawOracleConnection>);

unsafe impl Send for ConnectionHandle {}

impl ConnectionHandle {
    pub(crate) fn new(connection: RawOracleConnection) -> Self {
        Self(Arc::new(connection))
    }

    #[inline]
    pub(crate) fn connection(&self) -> &RawOracleConnection {
        self.0.as_ref()
    }

    #[inline]
    pub(crate) fn as_arc(&self) -> Arc<RawOracleConnection> {
        Arc::clone(&self.0)
    }
}
