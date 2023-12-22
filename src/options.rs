use futures_core::future::BoxFuture;
use rbdc::db::{ConnectOptions, Connection};
use rbdc::Error;
use serde::{Deserialize, Serialize};

use crate::connection::OracleConnection;

#[derive(Serialize, Deserialize, Debug)]
pub struct OracleConnectOptions {
    pub username: String,
    pub password: String,
    pub connect_string: String,
}

impl ConnectOptions for OracleConnectOptions {
    fn connect(&self) -> BoxFuture<Result<Box<dyn Connection>, Error>> {
        Box::pin(async move {
            let v = OracleConnection::establish(self)
                .await
                .map_err(|e| Error::from(e.to_string()))?;
            Ok(Box::new(v) as Box<dyn Connection>)
        })
    }

    fn set_uri(&mut self, url: &str) -> Result<(), Error> {
        *self = OracleConnectOptions::from_str(url)?;
        Ok(())
    }
}

impl Default for OracleConnectOptions {
    fn default() -> Self {
        Self {
            username: "scott".to_owned(),
            password: "tiger".to_owned(),
            connect_string: "//localhost/XE".to_owned(),
        }
    }
}

impl OracleConnectOptions {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        serde_json::from_str(s).map_err(|e| Error::from(e.to_string()))
    }
}
