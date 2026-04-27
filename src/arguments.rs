use crate::encode::{Encode, IsNull};
use oracle::Statement;
use rbdc::Error;
use rbs::Value;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum OracleArgumentValue {
    Null,
    String(String),
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Binary(Vec<u8>),
    Date(String),
    DateTime(String),
    Time(String),
    Decimal(String),
    Timestamp(i64),
    Uuid(String),
}

#[derive(Default, Debug, Clone)]
pub struct OracleArguments {
    pub(crate) values: Vec<OracleArgumentValue>,
}

impl OracleArguments {
    pub fn add<T>(&mut self, value: T) -> Result<(), Error>
    where
        T: Encode,
    {
        if let IsNull::Yes = value.encode(&mut self.values)? {
            self.values.push(OracleArgumentValue::Null);
        }
        Ok(())
    }

    pub fn from_args(args: Vec<Value>) -> Result<Self, Error> {
        let mut arguments = Self {
            values: Vec::with_capacity(args.len()),
        };
        for value in args {
            arguments.add(value)?;
        }
        Ok(arguments)
    }

    pub(crate) fn into_static(self) -> OracleArguments {
        let mut values = Vec::with_capacity(self.values.len());
        for value in self.values {
            values.push(OracleArgumentValue::into_static(value));
        }

        OracleArguments { values }
    }

    pub fn reserve(&mut self, len: usize, _size_hint: usize) {
        self.values.reserve(len);
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn values(&self) -> &[OracleArgumentValue] {
        &self.values
    }

    pub(super) fn bind(&self, statement: &mut Statement, offset: usize) -> Result<usize, Error> {
        let mut used = 0;
        for (index, value) in self.values.iter().enumerate().skip(offset) {
            value.bind(statement, index + 1 - offset)?;
            used += 1;
        }
        Ok(used)
    }
}

impl OracleArgumentValue {
    fn into_static(self) -> OracleArgumentValue {
        self
    }

    fn bind(&self, statement: &mut Statement, index: usize) -> Result<(), Error> {
        match self {
            OracleArgumentValue::Null => statement
                .bind(index, &Option::<String>::None)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::String(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::U32(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::U64(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::I32(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::I64(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::F32(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::F64(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::Binary(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::Date(value) => {
                let value = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                    .map_err(|e| Error::from(e.to_string()))?;
                statement
                    .bind(index, &value)
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            OracleArgumentValue::DateTime(value) => {
                let value = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f%z")
                    .map_err(|e| Error::from(e.to_string()))?;
                statement
                    .bind(index, &value)
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            OracleArgumentValue::Time(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::Decimal(value) => {
                let value = bigdecimal::BigDecimal::from_str(value)
                    .map_err(|e| Error::from(e.to_string()))?
                    .to_string();
                statement
                    .bind(index, &value)
                    .map_err(|e| Error::from(e.to_string()))?;
            }
            OracleArgumentValue::Timestamp(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
            OracleArgumentValue::Uuid(value) => statement
                .bind(index, value)
                .map_err(|e| Error::from(e.to_string()))?,
        }
        Ok(())
    }
}
