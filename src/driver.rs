use crate::options::OracleConnectOptions;
use crate::connection::OracleConnection;
use futures_core::future::BoxFuture;
use rbdc::db::{ConnectOptions, Connection};
use rbdc::db::{Driver, Placeholder};
use rbdc::Error;

#[derive(Debug)]
pub struct OracleDriver {}

impl Driver for OracleDriver {
    fn name(&self) -> &str {
        "oracle"
    }

    fn connect(&self, url: &str) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
        Box::pin(async move {
            let opt: OracleConnectOptions = OracleConnectOptions::default();
            let conn = OracleConnection::establish(&opt).await?;
            Ok(Box::new(conn) as Box<dyn Connection>)
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
        let mut last = ' ' as u8;
        let mut sql_bytes = sql.as_bytes().to_vec();
        let mut placeholder_idx = 1;
        let mut index = 0;
        loop {
            if index == sql_bytes.len() {
                break;
            }
            let x = sql_bytes[index];
            if x == '?' as u8 && last != '\\' as u8 {
                sql_bytes[index] = ':' as u8;
                let bytes = placeholder_idx.to_string().into_bytes();
                let mut idx = 0;
                for x in bytes {
                    sql_bytes.insert(index + 1 + idx, x);
                    last = x;
                    idx += 1;
                }
                placeholder_idx += 1;
            } else {
                last = x;
            }
            index += 1;
        }
        String::from_utf8(sql_bytes).unwrap_or_default()
    }
}

impl OracleDriver{
    pub fn pub_exchange(&self, sql: &str) -> String{
        self.exchange(sql)
    }
}