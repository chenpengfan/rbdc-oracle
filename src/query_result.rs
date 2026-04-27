use rbdc::db::ExecResult;
use rbs::Value;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct OracleQueryResult {
    pub(crate) rows_affected: u64,
    pub(crate) out_values: Vec<Value>,
}

impl OracleQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }

    pub fn out_values(&self) -> &[Value] {
        &self.out_values
    }

    pub fn into_exec_result(self) -> ExecResult {
        ExecResult {
            rows_affected: self.rows_affected,
            last_insert_id: Value::Array(self.out_values),
        }
    }
}

impl Extend<OracleQueryResult> for OracleQueryResult {
    fn extend<T: IntoIterator<Item = OracleQueryResult>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            self.out_values = elem.out_values;
        }
    }
}
