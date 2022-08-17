use bigdecimal::BigDecimal;
use std::str::FromStr;
use oracle::sql_type::OracleType;
use rbs::Value;
use rbdc::{Error, datetime::FastDateTime};

use crate::OracleData;

pub trait Decode {
    fn decode(row: &OracleData) -> Result<Value, Error>;
}

impl Decode for Value {
    fn decode(row: &OracleData) -> Result<Value, Error> {
        let s = row.str.as_ref();
        if s.is_none() {
            return Ok(Value::Null);
        }
        let value = s.unwrap().clone(); 
        let result:Value = match row.column_type {
            OracleType::Number(_v,_i) => {
                let dec = BigDecimal::from_str(&value).map_err(|e|Error::from(e.to_string()))?;
                    Value::String(dec.to_string()).into_ext("Decimal")
            }
            //OracleType::Int64 is integer 
            OracleType::Int64 => {
                let a = value.parse::<i32>()?;
                Value::I32(a)
            }
            OracleType::Long => {
                let a = value.parse::<i64>()?;
                Value::I64(a)
            }
            OracleType::Date => {
                let a = FastDateTime::from_str(&value)?;
                Value::from(a)
            }
            //TODO: more types!
            _=>{
                Value::String(value)
            }
        };
        Ok(result)
    }
}
