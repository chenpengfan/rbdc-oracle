use crate::options::OracleConnectOptions;
use futures_core::future::BoxFuture;
use rbdc::db::{ConnectOptions, Connection, Driver, Placeholder};
use rbdc::{Error, impl_exchange};

#[derive(Debug)]
pub struct OracleDriver;

impl Driver for OracleDriver {
    fn name(&self) -> &str {
        "oracle"
    }

    fn connect(&self, url: &str) -> BoxFuture<'_, Result<Box<dyn Connection>, Error>> {
        let url = url.to_owned();
        Box::pin(async move {
            let mut options = self.default_option();
            options.set_uri(&url)?;
            let options = options
                .downcast_ref::<OracleConnectOptions>()
                .ok_or_else(|| Error::from("downcast_ref failure"))?;
            let connection = options.connect().await?;
            Ok(Box::new(connection) as Box<dyn Connection>)
        })
    }

    fn connect_opt<'a>(
        &'a self,
        opt: &'a dyn ConnectOptions,
    ) -> BoxFuture<'a, Result<Box<dyn Connection>, Error>> {
        let options = opt
            .downcast_ref::<OracleConnectOptions>()
            .ok_or_else(|| Error::from("OracleDriver::connect_opt requires OracleConnectOptions"));
        Box::pin(async move {
            let options = options?;
            let connection = options.connect().await?;
            Ok(Box::new(connection) as Box<dyn Connection>)
        })
    }

    fn default_option(&self) -> Box<dyn ConnectOptions> {
        Box::new(OracleConnectOptions::default())
    }
}

impl Placeholder for OracleDriver {
    fn exchange(&self, sql: &str) -> String {
        impl_exchange(":", 1, sql)
    }
}

impl OracleDriver {
    pub fn pub_exchange(&self, sql: &str) -> String {
        self.exchange(sql)
    }
}
