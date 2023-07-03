use std::sync::Arc;
use oracle::sql_type::OracleType;
use rbdc::db::{Row, MetaData};
use rbs::Value;
use crate::decode::Decode;

pub mod decode;
pub mod driver;
pub mod encode;
pub mod options;
pub mod connection;

#[derive(Debug, Clone)]
pub struct OracleColumn {
    pub name: String,
    pub column_type: OracleType,
}

#[derive(Debug)]
pub struct OracleMetaData(pub Arc<Vec<OracleColumn>>);

impl MetaData for OracleMetaData {
    fn column_len(&self) -> usize {
        self.0.len()
    }

    fn column_name(&self, i: usize) -> String {
        self.0[i].name.to_string()
    }

    fn column_type(&self, i: usize) -> String {
        format!("{:?}", self.0[i].column_type)
    }
}

#[derive(Debug)]
pub struct OracleData {
    pub str: Option<String>,
    pub bin: Option<Vec<u8>>,
    pub column_type: OracleType,
    pub is_sql_null: bool,
}


#[derive(Debug)]
pub struct OracleRow {
    pub columns: Arc<Vec<OracleColumn>>,
    pub datas: Vec<OracleData>,
}


impl Row for OracleRow {
    fn meta_data(&self) -> Box<dyn MetaData> {
        Box::new(OracleMetaData(self.columns.clone()))
    }

    fn get(&mut self, i: usize) -> Result<Value, rbdc::Error> {
        Value::decode(
            &self.datas[i],
        )
    }
}