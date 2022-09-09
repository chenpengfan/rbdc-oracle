use crate::options::OracleConnectOptions;
use crate::connection::OracleConnection;
use futures_core::future::BoxFuture;
use rbdc::db::{ConnectOptions, Connection};
use rbdc::db::{Driver, Placeholder};
use rbdc::{Error, impl_exchange};

#[derive(Debug)]
pub struct OracleDriver {}

impl Driver for OracleDriver {
    fn name(&self) -> &str {
        "oracle"
    }

    fn connect(&self, _url: &str) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
        Box::pin(async move {
            unimplemented!();
        })
    }

    fn connect_opt<'a>(
        &'a self,
        opt: &'a dyn ConnectOptions,
    ) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
        let opt = opt.downcast_ref::<OracleConnectOptions>().unwrap();
        Box::pin(async move {
            let conn = OracleConnection::establish(opt).await?;
            Ok(Box::new(conn) as Box<dyn Connection>)
        })
    }

    fn default_option(&self) -> Box<dyn ConnectOptions> {
        Box::new(OracleConnectOptions::default())
    }
}

impl Placeholder for OracleDriver {
    fn exchange(&self, sql: &str) -> String {
        impl_exchange(":",1,sql)
    }
}

impl OracleDriver{
    pub fn pub_exchange(&self, sql: &str) -> String{
        self.exchange(sql)
    }
}