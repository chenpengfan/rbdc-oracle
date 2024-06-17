use std::sync::{Arc, Mutex};

use futures_core::future::BoxFuture;
use oracle::Connection as OraConnect;
use oracle::sql_type::OracleType;
use rbdc::db::{Connection, ExecResult, Row};
use rbdc::Error;
use rbs::Value;

use crate::{OracleColumn, OracleData, OracleRow};
use crate::driver::OracleDriver;
use crate::encode::Encode;
use crate::options::OracleConnectOptions;

#[derive(Clone)]
pub struct OracleConnection {
    pub conn: Arc<OraConnect>,
    pub is_trans: Arc<Mutex<bool>>,
}

impl Connection for OracleConnection {
    fn get_rows(
        &mut self,
        sql: &str,
        params: Vec<Value>,
    ) -> BoxFuture<Result<Vec<Box<dyn Row>>, Error>> {
        let sql: String = OracleDriver {}.pub_exchange(sql);
        let oc = self.clone();
        let task = tokio::task::spawn_blocking(move || {
            let builder = oc.conn.statement(&sql);
            let mut stmt = builder.build().map_err(|e| Error::from(e.to_string()))?;

            for (idx, x) in params.into_iter().enumerate() {
                x.encode(idx, &mut stmt).map_err(|e| Error::from(e.to_string()))?
            }

            let rows = stmt.query(&[]).map_err(|e| Error::from(e.to_string()))?;
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
                            bin: None,
                            column_type: t.clone(),
                            is_sql_null: true,
                        })
                    } else {
                        if t == OracleType::BLOB {
                            match col.get::<Vec<u8>>() {
                                Ok(bin) => datas.push(OracleData {
                                    str: None,
                                    bin: Some(bin),
                                    column_type: t.clone(),
                                    is_sql_null: false,
                                }),
                                Err(_) => datas.push(OracleData {
                                    str: None,
                                    bin: None,
                                    column_type: t.clone(),
                                    is_sql_null: false,
                                }),
                            }
                        } else {
                            match col.get::<String>() {
                                Ok(str) => datas.push(OracleData {
                                    str: Some(str),
                                    bin: None,
                                    column_type: t.clone(),
                                    is_sql_null: false,
                                }),
                                Err(_) => datas.push(OracleData {
                                    str: None,
                                    bin: None,
                                    column_type: t.clone(),
                                    is_sql_null: false,
                                }),
                            }
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
        });
        Box::pin(async move {
            task.await.map_err(|e| Error::from(e.to_string()))?
        })
    }

    fn exec(
        &mut self,
        sql: &str,
        params: Vec<Value>,
    ) -> BoxFuture<Result<ExecResult, Error>> {
        let oc = self.clone();
        let sql = sql.to_string();
        let task = tokio::task::spawn_blocking(move || {
            let mut trans = oc.is_trans.lock()
                .map_err(|e| Error::from(e.to_string()))?;
            if sql == "begin" {
                *trans = true;
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            } else if sql == "commit" {
                oc.conn.commit().unwrap();
                *trans = false;
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            } else if sql == "rollback" {
                oc.conn.rollback().unwrap();
                *trans = false;
                Ok(ExecResult {
                    rows_affected: 0,
                    last_insert_id: Value::Null,
                })
            } else {
                let sql: String = OracleDriver {}.pub_exchange(&sql);
                let builder = oc.conn.statement(&sql);
                let mut stmt = builder.build().map_err(|e| Error::from(e.to_string()))?;
                for (idx, x) in params.into_iter().enumerate() {
                    x.encode(idx, &mut stmt).map_err(|e| Error::from(e.to_string()))?
                }
                stmt
                    .execute(&[])
                    .map_err(|e| Error::from(e.to_string()))?;
                if !*trans {
                    oc.conn.commit().map_err(|e| Error::from(e.to_string()))?;
                    *trans = false;
                }
                let rows_affected = stmt.row_count().map_err(|e| Error::from(e.to_string()))?;
                let mut ret = vec![];
                for i in 1..=stmt.bind_count() {
                    let res: Result<String, _> = stmt.bind_value(i);
                    match res {
                        Ok(v) => {
                            ret.push(Value::String(v))
                        }
                        Err(_) => {
                            ret.push(Value::Null)
                        }
                    }
                }
                Ok(ExecResult {
                    rows_affected,
                    last_insert_id: Value::Array(ret),
                })
            }
        });
        Box::pin(async {
            task.await.map_err(|e| Error::from(e.to_string()))?
        })
    }

    fn ping(&mut self) -> BoxFuture<Result<(), rbdc::Error>> {
        let oc = self.clone();
        let task = tokio::task::spawn_blocking(move || {
            oc.conn.ping()
                .map_err(|e| Error::from(e.to_string()))?;
            Ok(())
        });
        Box::pin(async {
            task.await.map_err(|e| Error::from(e.to_string()))?
        })
    }

    fn close(&mut self) -> BoxFuture<Result<(), rbdc::Error>> {
        let oc = self.clone();
        let task = tokio::task::spawn_blocking(move || {
            oc.conn.commit().map_err(|e| Error::from(e.to_string()))?;
            oc.conn.close().map_err(|e| Error::from(e.to_string()))?;
            Ok(())
        });
        Box::pin(async {
            task.await.map_err(|e| Error::from(e.to_string()))?
        })
    }
}

impl OracleConnection {
    pub async fn establish(opt: &OracleConnectOptions) -> Result<Self, Error> {
        let conn = OraConnect::connect(opt.username.clone(), opt.password.clone(), opt.connect_string.clone())
            .map_err(|e| Error::from(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(conn),
            is_trans: Arc::new(Mutex::new(false)),
        })
    }
}
