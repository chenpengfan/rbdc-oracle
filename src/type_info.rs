use oracle::sql_type::OracleType;
use rbs::Value;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone)]
pub struct OracleTypeInfo {
    pub(crate) oracle_type: Option<OracleType>,
    pub(crate) name: String,
}

pub trait Type {
    fn type_info(&self) -> OracleTypeInfo;
}

impl<T: Type> Type for Option<T> {
    fn type_info(&self) -> OracleTypeInfo {
        match self {
            Some(value) => value.type_info(),
            None => OracleTypeInfo::null(),
        }
    }
}

impl OracleTypeInfo {
    pub fn from_oracle_type(oracle_type: OracleType) -> Self {
        Self {
            name: format!("{oracle_type:?}"),
            oracle_type: Some(oracle_type),
        }
    }

    pub fn null() -> Self {
        Self {
            oracle_type: None,
            name: "NULL".to_owned(),
        }
    }

    pub fn number() -> Self {
        Self {
            oracle_type: None,
            name: "NUMBER".to_owned(),
        }
    }

    pub fn float() -> Self {
        Self {
            oracle_type: None,
            name: "FLOAT".to_owned(),
        }
    }

    pub fn text() -> Self {
        Self {
            oracle_type: None,
            name: "VARCHAR2".to_owned(),
        }
    }

    pub fn binary() -> Self {
        Self {
            oracle_type: None,
            name: "BLOB".to_owned(),
        }
    }

    pub fn date() -> Self {
        Self {
            oracle_type: None,
            name: "DATE".to_owned(),
        }
    }

    pub fn oracle_type(&self) -> Option<&OracleType> {
        self.oracle_type.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for OracleTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(self.name())
    }
}

impl Type for Value {
    fn type_info(&self) -> OracleTypeInfo {
        match self {
            Value::Null => OracleTypeInfo::null(),
            Value::Bool(_) => OracleTypeInfo::number(),
            Value::I32(_) | Value::I64(_) | Value::U32(_) | Value::U64(_) => {
                OracleTypeInfo::number()
            }
            Value::F32(_) | Value::F64(_) => OracleTypeInfo::float(),
            Value::String(_) => OracleTypeInfo::text(),
            Value::Binary(_) => OracleTypeInfo::binary(),
            Value::Array(_) | Value::Map(_) => OracleTypeInfo::text(),
            Value::Ext(type_name, _) => match *type_name {
                "Date" | "DateTime" | "Time" => OracleTypeInfo::date(),
                "Timestamp" => OracleTypeInfo::number(),
                "Decimal" => OracleTypeInfo::number(),
                "Json" => OracleTypeInfo::binary(),
                "Uuid" => OracleTypeInfo::text(),
                _ => OracleTypeInfo::null(),
            },
        }
    }
}
