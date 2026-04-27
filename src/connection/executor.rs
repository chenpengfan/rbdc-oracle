use crate::connection::ConnectionState;
use crate::query::OracleQuery;
use crate::{
    OracleColumn, OracleQueryResult, OracleRow, OracleStatement, OracleTypeInfo, OracleValue,
};
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{TryFutureExt, TryStreamExt, pin_mut};
use rbdc::Error;
use rbdc::try_stream;
use std::sync::Arc;

use super::OracleConnection;

type StepResult = Result<Either<OracleQueryResult, OracleRow>, Error>;

pub(crate) fn prepare(conn: &mut ConnectionState, query: &str) -> Result<OracleStatement, Error> {
    let statement = conn.statements.get(query, true)?;

    let mut parameters = 0;
    let mut columns = None;

    while let Some(prepared) = statement.prepare_next(&mut conn.handle)? {
        parameters += count_bind_parameters(&conn.handle, prepared.handle.sql())?;
        if !prepared.columns.is_empty() && columns.is_none() {
            columns = Some(Arc::clone(prepared.columns));
        }
    }

    Ok(OracleStatement {
        sql: query.to_owned(),
        columns: columns.unwrap_or_default(),
        parameters,
    })
}

pub(crate) fn run_prepared(
    handle: &mut crate::connection::ConnectionHandle,
    query: &str,
    columns: &mut Arc<Vec<OracleColumn>>,
    bind: impl FnOnce(&mut oracle::Statement) -> Result<usize, Error>,
) -> Result<(Vec<StepResult>, usize), Error> {
    let mut statement = build_statement(handle, query)?;
    let args_used_now = bind(&mut statement)?;
    let mut results = execute_statement(&mut statement, columns)?;
    results.push(Ok(Either::Left(build_query_result(&statement)?)));

    Ok((results, args_used_now))
}

fn count_bind_parameters(
    handle: &crate::connection::ConnectionHandle,
    query: &str,
) -> Result<usize, Error> {
    let statement = build_statement(handle, query)?;
    Ok(statement.bind_count())
}

fn build_statement(
    handle: &crate::connection::ConnectionHandle,
    query: &str,
) -> Result<oracle::Statement, Error> {
    let arc = handle.as_arc();
    arc.statement(query).build().map_err(to_error)
}

fn execute_statement(
    statement: &mut oracle::Statement,
    columns: &mut Arc<Vec<OracleColumn>>,
) -> Result<Vec<StepResult>, Error> {
    match statement.query(&[]) {
        Ok(rows) => collect_query_rows(rows, columns),
        Err(_) => {
            statement.execute(&[]).map_err(to_error)?;
            Ok(Vec::with_capacity(1))
        }
    }
}

fn collect_query_rows(
    rows: oracle::ResultSet<'_, oracle::Row>,
    columns: &mut Arc<Vec<OracleColumn>>,
) -> Result<Vec<StepResult>, Error> {
    cache_columns(columns, rows.column_info());

    let mut results = Vec::with_capacity(columns.len());
    for row in rows {
        let row = row.map_err(to_error)?;
        results.push(collect_row(&row, columns).map(Either::Right));
    }

    Ok(results)
}

fn cache_columns(columns: &mut Arc<Vec<OracleColumn>>, column_info: &[oracle::ColumnInfo]) {
    if !columns.is_empty() {
        return;
    }

    let mut cached = Vec::with_capacity(column_info.len());
    for (ordinal, info) in column_info.iter().enumerate() {
        cached.push(OracleColumn {
            name: info.name().to_lowercase().into(),
            ordinal,
            type_info: OracleTypeInfo::from_oracle_type(info.oracle_type().clone()),
        });
    }

    *columns = Arc::new(cached);
}

fn collect_row(row: &oracle::Row, columns: &Arc<Vec<OracleColumn>>) -> Result<OracleRow, Error> {
    let sql_values = row.sql_values();
    let mut values = Vec::with_capacity(sql_values.len());
    for value in sql_values.iter() {
        values.push(collect_value(value)?);
    }

    Ok(OracleRow::new(Arc::clone(columns), values))
}

fn build_query_result(statement: &oracle::Statement) -> Result<OracleQueryResult, Error> {
    Ok(OracleQueryResult {
        rows_affected: statement.row_count().map_err(to_error)?,
        out_values: collect_out_values(statement),
    })
}

fn collect_value(value: &oracle::SqlValue) -> Result<OracleValue, Error> {
    let oracle_type = value.oracle_type().map_err(to_error)?.clone();
    let is_null = value.is_null().map_err(to_error)?;
    let (text, binary) = if is_null {
        (None, None)
    } else if oracle_type == oracle::sql_type::OracleType::BLOB {
        (None, value.get::<Vec<u8>>().ok())
    } else {
        (value.get::<String>().ok(), None)
    };

    Ok(OracleValue::new(
        text,
        binary,
        OracleTypeInfo::from_oracle_type(oracle_type),
        is_null,
    ))
}

fn collect_out_values(statement: &oracle::Statement) -> Vec<rbs::Value> {
    let mut out_values = Vec::with_capacity(statement.bind_count());

    for index in 1..=statement.bind_count() {
        let value: Result<String, _> = statement.bind_value(index);
        match value {
            Ok(value) => out_values.push(rbs::Value::String(value)),
            Err(_) => out_values.push(rbs::Value::Null),
        }
    }

    out_values
}

fn to_error(error: impl std::fmt::Display) -> Error {
    Error::from(error.to_string())
}

impl OracleConnection {
    pub fn fetch_many(
        &mut self,
        query: OracleQuery,
    ) -> BoxStream<'_, Result<Either<OracleQueryResult, OracleRow>, Error>> {
        let sql = query.sql().to_owned();
        let persistent = query.persistent() && !query.arguments.is_empty();
        Box::pin(try_stream! {
            let arguments = query.take_arguments()?;
            let stream = self.worker
                .execute(sql, arguments, self.row_channel_size, persistent)
                .map_ok(|rx| rx.into_stream())
                .try_flatten_stream();
            pin_mut!(stream);
            while let Some(item) = stream.try_next().await? {
                r#yield!(item);
            }
            Ok(())
        })
    }

    pub fn fetch_optional(
        &mut self,
        query: OracleQuery,
    ) -> BoxFuture<'_, Result<Option<OracleRow>, Error>> {
        let sql = query.sql().to_owned();
        let persistent = query.persistent() && !query.arguments.is_empty();
        Box::pin(async move {
            let arguments = query.take_arguments()?;
            let stream = self
                .worker
                .execute(sql, arguments, self.row_channel_size, persistent)
                .map_ok(|rx| rx.into_stream())
                .try_flatten_stream();
            pin_mut!(stream);
            while let Some(item) = stream.try_next().await? {
                if let Either::Right(row) = item {
                    return Ok(Some(row));
                }
            }
            Ok(None)
        })
    }

    pub fn prepare_with<'a>(
        &'a mut self,
        sql: &'a str,
        _parameters: &[OracleTypeInfo],
    ) -> BoxFuture<'a, Result<OracleStatement, Error>> {
        Box::pin(async move {
            let statement = self.worker.prepare(sql).await?;
            Ok(OracleStatement {
                sql: sql.into(),
                ..statement
            })
        })
    }
}
