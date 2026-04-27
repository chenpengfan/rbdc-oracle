#![allow(clippy::rc_buffer)]

use crate::OracleColumn;
use crate::connection::ConnectionHandle;
use crate::statement::OracleStatementHandle;
use rbdc::Error;
use std::sync::Arc;

#[derive(Debug)]
pub struct VirtualStatement {
    persistent: bool,
    index: Option<usize>,
    // Oracle keeps a single SQL unit here; unlike SQLite we do not split on `;`
    // because PL/SQL blocks legitimately contain semicolons.
    sql: Option<String>,
    pub(crate) handles: Vec<OracleStatementHandle>,
    pub(crate) columns: Vec<Arc<Vec<OracleColumn>>>,
}

pub struct PreparedStatement<'a> {
    pub(crate) handle: &'a mut OracleStatementHandle,
    pub(crate) columns: &'a mut Arc<Vec<OracleColumn>>,
}

impl VirtualStatement {
    pub(crate) fn new(query: &str, persistent: bool) -> Result<Self, Error> {
        Ok(Self {
            persistent,
            index: None,
            sql: Some(query.trim().to_owned()),
            handles: Vec::with_capacity(1),
            columns: Vec::with_capacity(1),
        })
    }

    pub(crate) fn prepare_next(
        &mut self,
        _conn: &mut ConnectionHandle,
    ) -> Result<Option<PreparedStatement<'_>>, Error> {
        self.index = self.index.map(|index| index + 1).or(Some(0));

        while self.handles.len() <= self.index.unwrap_or(0) {
            let Some(sql) = self.sql.take() else {
                return Ok(None);
            };

            if sql.is_empty() {
                return Ok(None);
            }

            self.handles.push(OracleStatementHandle::new(&sql));
            self.columns.push(Arc::new(Vec::with_capacity(0)));
        }

        Ok(self.current())
    }

    pub fn current(&mut self) -> Option<PreparedStatement<'_>> {
        self.index
            .filter(|&index| index < self.handles.len())
            .map(move |index| PreparedStatement {
                handle: &mut self.handles[index],
                columns: &mut self.columns[index],
            })
    }

    pub fn reset(&mut self) -> Result<(), Error> {
        self.index = None;
        let _ = self.persistent;
        Ok(())
    }
}
