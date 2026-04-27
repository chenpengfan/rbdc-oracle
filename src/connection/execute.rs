use crate::connection::ConnectionState;
use crate::{OracleArguments, OracleQueryResult, OracleRow};
use either::Either;
use rbdc::Error;
use std::collections::VecDeque;

pub struct ExecuteIter<'a> {
    handle: &'a mut crate::connection::ConnectionHandle,
    transaction_active: &'a mut bool,
    query: &'a str,
    statement: &'a mut crate::statement::VirtualStatement,
    args: Option<OracleArguments>,
    args_used: usize,
    goto_next: bool,
    finished: bool,
    buffered: VecDeque<Result<Either<OracleQueryResult, OracleRow>, Error>>,
}

pub(crate) fn iter<'a>(
    conn: &'a mut ConnectionState,
    query: &'a str,
    args: Option<OracleArguments>,
    persistent: bool,
) -> Result<ExecuteIter<'a>, Error> {
    let statement = conn.statements.get(query, persistent)?;

    Ok(ExecuteIter {
        handle: &mut conn.handle,
        transaction_active: &mut conn.transaction_active,
        query,
        statement,
        args,
        args_used: 0,
        goto_next: true,
        finished: false,
        buffered: VecDeque::with_capacity(1),
    })
}

fn bind(
    statement: &mut oracle::Statement,
    arguments: &Option<OracleArguments>,
    offset: usize,
) -> Result<usize, Error> {
    let mut used = 0;
    if let Some(arguments) = arguments {
        used = arguments.bind(statement, offset)?;
    }
    Ok(used)
}

fn transaction_command(query: &str) -> Option<&'static str> {
    let query = query.trim();

    if query.eq_ignore_ascii_case("begin") {
        Some("begin")
    } else if query.eq_ignore_ascii_case("commit") {
        Some("commit")
    } else if query.eq_ignore_ascii_case("rollback") {
        Some("rollback")
    } else {
        None
    }
}

impl Iterator for ExecuteIter<'_> {
    type Item = Result<Either<OracleQueryResult, OracleRow>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if let Some(item) = self.buffered.pop_front() {
            return Some(item);
        }

        if self.goto_next {
            if let Some(command) = transaction_command(self.query) {
                self.finished = true;

                if command == "begin" {
                    *self.transaction_active = true;
                    return Some(Ok(Either::Left(OracleQueryResult::default())));
                }

                let result = match command {
                    "commit" => self.handle.connection().commit(),
                    "rollback" => self.handle.connection().rollback(),
                    _ => unreachable!(),
                }
                .map_err(|e| Error::from(e.to_string()))
                .map(|_| {
                    *self.transaction_active = false;
                    Either::Left(OracleQueryResult::default())
                });
                return Some(result);
            }
        }

        let prepared = if self.goto_next {
            let prepared = match self.statement.prepare_next(self.handle) {
                Ok(Some(statement)) => statement,
                Ok(None) => return None,
                Err(e) => return Some(Err(e)),
            };
            self.goto_next = false;
            prepared
        } else {
            self.statement.current()?
        };

        let items = match super::executor::run_prepared(
            self.handle,
            prepared.handle.sql(),
            prepared.columns,
            |statement| bind(statement, &self.args, self.args_used),
        ) {
            Ok((items, args_used)) => {
                self.args_used += args_used;
                items
            }
            Err(e) => return Some(Err(e)),
        };

        if !*self.transaction_active {
            if let Err(e) = self
                .handle
                .connection()
                .commit()
                .map_err(|e| Error::from(e.to_string()))
            {
                return Some(Err(e));
            }
        }

        self.goto_next = true;
        self.buffered = items.into();
        self.buffered.pop_front()
    }
}

impl Drop for ExecuteIter<'_> {
    fn drop(&mut self) {
        self.statement.reset().ok();
    }
}
