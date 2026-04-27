#[derive(Debug, Clone)]
pub(crate) struct OracleStatementHandle {
    sql: String,
}

impl OracleStatementHandle {
    pub(crate) fn new(sql: &str) -> Self {
        Self {
            sql: sql.to_owned(),
        }
    }

    pub(crate) fn sql(&self) -> &str {
        &self.sql
    }
}
