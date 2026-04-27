use crate::decode::Decode;
use crate::{OracleColumn, OracleValue, OracleValueRef};
use rbdc::Error;
use rbdc::db::{MetaData, Row};
use rbs::Value;
use std::sync::Arc;

#[derive(Debug)]
pub struct OracleRow {
    pub(crate) values: Vec<OracleValue>,
    pub(crate) columns: Arc<Vec<OracleColumn>>,
}

#[derive(Debug)]
pub struct OracleMetaData {
    pub(crate) columns: Arc<Vec<OracleColumn>>,
}

impl OracleRow {
    pub(crate) fn new(columns: Arc<Vec<OracleColumn>>, values: Vec<OracleValue>) -> Self {
        Self { values, columns }
    }

    fn try_get_raw(&self, index: usize) -> Result<OracleValueRef<'_>, Error> {
        self.values
            .get(index)
            .map(OracleValueRef::value)
            .ok_or_else(|| Error::from("Index out of bounds"))
    }
}

impl MetaData for OracleMetaData {
    fn column_len(&self) -> usize {
        self.columns.len()
    }

    fn column_name(&self, i: usize) -> String {
        self.columns[i].name().to_owned()
    }

    fn column_type(&self, i: usize) -> String {
        self.columns[i].type_info().to_string()
    }
}

impl Row for OracleRow {
    fn meta_data(&self) -> Box<dyn MetaData> {
        Box::new(OracleMetaData {
            columns: self.columns.clone(),
        })
    }

    fn get(&mut self, i: usize) -> Result<Value, Error> {
        self.try_get_raw(i).and_then(Value::decode)
    }
}
