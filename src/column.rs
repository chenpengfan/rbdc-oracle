use crate::OracleTypeInfo;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OracleColumn {
    pub(crate) name: Arc<str>,
    pub(crate) ordinal: usize,
    pub(crate) type_info: OracleTypeInfo,
}

impl OracleColumn {
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_info(&self) -> &OracleTypeInfo {
        &self.type_info
    }
}
