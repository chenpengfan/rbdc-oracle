use std::str::FromStr;

use bigdecimal::BigDecimal;
use oracle::Statement;
use rbdc::Error;
use rbs::Value;

pub trait Encode {
    fn encode(self, idx: usize, statement: &mut Statement) -> Result<(), Error>;
}

impl Encode for Value {
    fn encode(self, idx: usize, statement: &mut Statement) -> Result<(), Error> {
        let idx = idx + 1;//oracle is one-based
        match self {
            Value::Ext(t, v) => match t {
                "Date" => {
                    let s = v.as_str().unwrap_or_default();
                    let d = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
                    statement.bind(idx, &d).map_err(|e| e.to_string())?
                }
                "DateTime" => {
                    let s = v.as_str().unwrap_or_default();
                    let d = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap();
                    statement.bind(idx, &d).map_err(|e| e.to_string())?
                }
                "Time" => {
                    //TODO: need to fix this
                    let s = v.into_string().unwrap();
                    statement.bind(idx, &s).map_err(|e| e.to_string())?
                }
                "Decimal" => {
                    let d = BigDecimal::from_str(&v.into_string().unwrap_or_default()).unwrap().to_string();
                    statement.bind(idx, &d).map_err(|e| e.to_string())?
                }
                "Json" => {
                    return Err(Error::from("unimpl"));
                }
                "Timestamp" => {
                    let t = v.as_u64().unwrap_or_default() as i64;
                    statement.bind(idx, &t).map_err(|e| e.to_string())?
                }
                "Uuid" => {
                    let d = v.into_string().unwrap();
                    statement.bind(idx, &d).map_err(|e| e.to_string())?
                }
                _ => {
                    return Err(Error::from("unimpl"));
                }
            }
            Value::String(str) => {
                statement.bind(idx, &str).map_err(|e| e.to_string())?
            }
            Value::U32(u) => {
                statement.bind(idx, &u).map_err(|e| e.to_string())?
            }
            Value::U64(u) => {
                statement.bind(idx, &u).map_err(|e| e.to_string())?
            }
            Value::I32(int) => {
                statement.bind(idx, &int).map_err(|e| e.to_string())?
            }
            Value::I64(long) => {
                statement.bind(idx, &long).map_err(|e| e.to_string())?
            }
            Value::F32(float) => {
                statement.bind(idx, &float).map_err(|e| e.to_string())?
            }
            Value::F64(double) => {
                statement.bind(idx, &double).map_err(|e| e.to_string())?
            }
            Value::Binary(bin) => {
                statement.bind(idx, &bin).map_err(|e| e.to_string())?
            }
            Value::Null => {
                statement.bind(idx, &Option::<String>::None).unwrap();
            }
            //TODO: more types!
            _ => {
                statement.bind(idx, &self.to_string()).map_err(|e| e.to_string())?
            }
        }
        Ok(())
    }
}