use bigdecimal::BigDecimal;
use oracle::sql_type::OracleType;
use rbdc::{datetime::FastDateTime, Error};
use rbs::Value;
use std::str::FromStr;

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
        match row.column_type {
            OracleType::Number(p, s) => {
                if p ==0 && s == -127{
                    // it means number(*)
                    let dec =
                        BigDecimal::from_str(&value).map_err(|e| Error::from(e.to_string()))?;
                    if dec.is_integer() {
                        let d = dec.digits();
                        if 1 <= d && d <= 9 {
                            let a = value.parse::<i32>()?;
                            return Ok(Value::I32(a));
                        } else if 10 <= d && d <= 18 {
                            let a = value.parse::<i64>()?;
                            return Ok(Value::I64(a));
                        }
                        return Ok(Value::String(dec.to_string()).into_ext("Decimal"));
                    }
                    return Ok(Value::String(dec.to_string()).into_ext("Decimal"));
                }
                if s > 0 {
                    let dec =
                        BigDecimal::from_str(&value).map_err(|e| Error::from(e.to_string()))?;
                    return Ok(Value::String(dec.to_string()).into_ext("Decimal"));
                } else if 1 <= p && p <= 9 {
                    let a = value.parse::<i32>()?;
                    return Ok(Value::I32(a));
                } else if 10 <= p && p <= 18 {
                    let a = value.parse::<i64>()?;
                    return Ok(Value::I64(a));
                }
                let dec = BigDecimal::from_str(&value).map_err(|e| Error::from(e.to_string()))?;
                return Ok(Value::String(dec.to_string()).into_ext("Decimal"));
            }
            //OracleType::Int64 is integer
            OracleType::Int64 => {
                let a = value.parse::<i32>()?;
                return Ok(Value::I32(a));
            }
            OracleType::Long => {
                let a = value.parse::<i64>()?;
                return Ok(Value::I64(a));
            }
            OracleType::Date => {
                let a = FastDateTime::from_str(&value)?;
                return Ok(Value::from(a));
            }
            //TODO: more types!
            _ => return Ok(Value::String(value)),
        };
    }
}
