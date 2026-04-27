use crate::{OracleColumn, OracleTypeInfo};
use either::Either;
use std::sync::Arc;

mod handle;
mod r#virtual;

pub(crate) use handle::OracleStatementHandle;
pub(crate) use r#virtual::VirtualStatement;

#[derive(Debug, Clone)]
pub struct OracleStatement {
    pub(crate) sql: String,
    pub(crate) parameters: usize,
    pub(crate) columns: Arc<Vec<OracleColumn>>,
}

impl OracleStatement {
    pub fn to_owned(&self) -> OracleStatement {
        OracleStatement {
            sql: self.sql.clone(),
            parameters: self.parameters,
            columns: Arc::clone(&self.columns),
        }
    }

    pub fn sql(&self) -> &str {
        &self.sql
    }

    pub fn parameters(&self) -> Option<Either<&[OracleTypeInfo], usize>> {
        Some(Either::Right(self.parameters))
    }

    pub fn columns(&self) -> &[OracleColumn] {
        &self.columns
    }
}
