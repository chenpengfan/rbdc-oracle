use crate::query::OracleQuery;
use crate::type_info::Type;
use crate::{OracleArguments, OracleConnectOptions, OracleConnection, OracleQueryResult};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::FutureExt;
use futures_util::{StreamExt, TryStreamExt};
use rbdc::Error;
use rbdc::db::{Connection, ExecResult, Row};
use rbdc::try_stream;
use rbs::Value;

impl OracleConnectOptions {
    pub fn connect(&self) -> BoxFuture<'_, Result<OracleConnection, Error>> {
        Box::pin(async move { OracleConnection::establish(self).await })
    }
}

impl Connection for OracleConnection {
    fn exec_rows(
        &mut self,
        sql: &str,
        params: Vec<Value>,
    ) -> BoxFuture<'_, Result<BoxStream<'_, Result<Box<dyn Row>, Error>>, Error>> {
        let sql = crate::OracleDriver.pub_exchange(sql);
        let row_channel_size = self.row_channel_size;
        let has_args = !params.is_empty();

        Box::pin(async move {
            let rx = if has_args {
                let arguments = OracleArguments::from_args(params)?;
                self.worker
                    .execute(sql, Some(arguments.into_static()), row_channel_size, true)
                    .await
                    .map_err(|_| Error::from("WorkerCrashed"))?
            } else {
                self.worker
                    .execute(sql, None, row_channel_size, false)
                    .await
                    .map_err(|_| Error::from("WorkerCrashed"))?
            };

            let stream = try_stream! {
                let mut stream = rx.into_stream();
                while let Some(item) = stream.next().await {
                    match item? {
                        Either::Left(_) => {}
                        Either::Right(row) => {
                            r#yield!(Box::new(row) as Box<dyn Row>);
                        }
                    }
                }
                Ok(())
            }
            .boxed();

            Ok(stream as BoxStream<'_, Result<Box<dyn Row>, Error>>)
        })
    }

    fn exec(&mut self, sql: &str, params: Vec<Value>) -> BoxFuture<'_, Result<ExecResult, Error>> {
        let sql = crate::OracleDriver.pub_exchange(sql);
        Box::pin(async move {
            let many = {
                if params.is_empty() {
                    self.fetch_many(OracleQuery {
                        statement: Either::Left(sql),
                        arguments: params,
                        persistent: false,
                    })
                } else {
                    let mut type_info = Vec::with_capacity(params.len());
                    for value in &params {
                        type_info.push(value.type_info());
                    }
                    let stmt = self.prepare_with(&sql, &type_info).await?;
                    self.fetch_many(OracleQuery {
                        statement: Either::Right(stmt),
                        arguments: params,
                        persistent: true,
                    })
                }
            };

            let stream: BoxStream<'_, Result<OracleQueryResult, Error>> = many
                .try_filter_map(|step| async move {
                    Ok(match step {
                        Either::Left(result) => Some(result),
                        Either::Right(_) => None,
                    })
                })
                .boxed();
            let result: OracleQueryResult = stream.try_collect().boxed().await?;
            Ok(result.into_exec_result())
        })
    }

    fn close(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { self.do_close().await })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move { self.worker.ping().await })
    }
}
