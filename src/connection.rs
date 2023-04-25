use futures_core::future::BoxFuture;
use oracle::Connection as OraConnect;
use oracle::sql_type::ToSql;
use rbdc::db::{Connection, ExecResult, Row};
use rbdc::Error;
use rbs::Value;
use std::sync::Arc;

use crate::driver::OracleDriver;
use crate::encode::Encode;
use crate::options::OracleConnectOptions;
use crate::{OracleColumn, OracleData, OracleRow};

#[derive(Clone)]
pub struct OracleConnection {
    pub conn: Arc<OraConnect>,
}

impl Connection for OracleConnection {
    fn get_rows(
        &mut self,
        sql: &str,
        params: Vec<Value>,
    ) -> BoxFuture<Result<Vec<Box<dyn Row>>, rbdc::Error>> {
        let sql: String = OracleDriver {}.pub_exchange(sql);
        Box::pin(async move {
            let mut p = Vec::with_capacity(params.len());
            for x in params {
                x.encode(&mut p).map_err(|e| Error::from(e.to_string()))?
            }
            let p: Vec<&dyn ToSql> = p.iter().map(|s| &**s).collect();
            let builder = self.conn.statement(&sql);
            let mut stmt = builder.build().map_err(|e| Error::from(e.to_string()))?;
            let rows = stmt.query(&p).map_err(|e| Error::from(e.to_string()))?;
            let col_infos = rows.column_info();
            let col_count = col_infos.len();
            let mut results = Vec::with_capacity(col_count);
            let mut columns = Vec::with_capacity(col_count);
            for info in col_infos.iter() {
                columns.push(OracleColumn {
                    name: info.name().to_string().to_lowercase(),
                    column_type: info.oracle_type().clone(),
                })
            }
            for row_result in rows {
                let row = row_result.map_err(|e| Error::from(e.to_string()))?;
                let mut datas = Vec::with_capacity(col_count);
                for col in row.sql_values().iter() {
                    let t = col.oracle_type().map_err(|e| Error::from(e.to_string()))?;
                    let t = t.clone();
                    if let Ok(true) = col.is_null() {
                        datas.push(OracleData {
                            str: None,
                            column_type: t.clone(),
                        })
                    } else {
                        match col.get::<String>() {
                            Ok(str) => datas.push(OracleData {
                                str: Some(str),
                                column_type: t.clone(),
                            }),
                            Err(_) => datas.push(OracleData {
                                str: None,
                                column_type: t.clone(),
                            }),
                        }
                    }
                }
                let row = OracleRow {
                    columns: Arc::new(columns.clone()),
                    datas: datas,
                };
                results.push(Box::new(row) as Box<dyn Row>);
            }
            Ok(results)
        })
    }

    fn exec(
        &mut self,
        sql: &str,
        params: Vec<Value>,
    ) -> BoxFuture<Result<ExecResult, rbdc::Error>> {
        if sql == "begin" {
            Box::pin(async move {
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            })
        } else if sql == "commit" {
            Box::pin(async move {
                self.conn.commit().unwrap();
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            })
        } else if sql == "rollback" {
            Box::pin(async move {
                self.conn.rollback().unwrap();
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            })
        } else {
            let sql: String = OracleDriver {}.pub_exchange(sql);
            Box::pin(async move {
                let mut p = Vec::with_capacity(params.len());
                for x in params {
                    x.encode(&mut p).map_err(|e| Error::from(e.to_string()))?
                }
                let p: Vec<&dyn ToSql> = p.iter().map(|s| &**s).collect();
                let v = self
                    .conn
                    .execute(&sql, &p)
                    .map_err(|e| Error::from(e.to_string()))?;
                let rows_affected = v.row_count().map_err(|e| Error::from(e.to_string()))?;
                // self.conn.commit().map_err(|e| Error::from(e.to_string()))?;
                Ok(ExecResult {
                    rows_affected,
                    last_insert_id: Value::Null,
                })
            })
        }
    }

    fn close(&mut self) -> BoxFuture<Result<(), rbdc::Error>> {
        Box::pin(async move {
            self.conn.commit().map_err(|e| Error::from(e.to_string()))?;
            self.conn.close().map_err(|e| Error::from(e.to_string()))?;
            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<Result<(), rbdc::Error>> {
        Box::pin(async move {
            self.conn.ping()
                .map_err(|e| Error::from(e.to_string()))?;
            Ok(())
        })
    }
}

impl OracleConnection {
    pub async fn establish(opt: &OracleConnectOptions) -> Result<Self, Error> {
        let conn = OraConnect::connect(opt.username.clone(), opt.password.clone(), opt.connect_string.clone())
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(conn),
        })
    }
}
