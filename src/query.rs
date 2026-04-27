use crate::{OracleArguments, OracleStatement};
use either::Either;
use rbdc::Error;

#[derive(Debug, Clone)]
pub struct OracleQuery {
    pub statement: Either<String, OracleStatement>,
    pub arguments: Vec<rbs::Value>,
    pub persistent: bool,
}

impl OracleQuery {
    #[inline]
    pub fn sql(&self) -> &str {
        match self.statement {
            Either::Right(ref statement) => &statement.sql,
            Either::Left(ref sql) => sql,
        }
    }

    pub fn statement(&self) -> Option<&OracleStatement> {
        match self.statement {
            Either::Right(ref statement) => Some(statement),
            Either::Left(_) => None,
        }
    }

    #[inline]
    pub fn take_arguments(self) -> Result<Option<OracleArguments>, Error> {
        if self.arguments.is_empty() {
            return Ok(None);
        }
        Ok(Some(OracleArguments::from_args(self.arguments)?))
    }

    #[inline]
    pub fn persistent(&self) -> bool {
        self.persistent
    }
}
