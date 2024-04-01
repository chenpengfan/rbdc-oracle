use bigdecimal::BigDecimal;
use oracle::sql_type::OracleType;
use rbdc::{datetime::DateTime, Error};
use rbs::Value;
use std::str::FromStr;

use crate::OracleData;

pub trait Decode {
    fn decode(row: &OracleData) -> Result<Value, Error>;
}

impl Decode for Value {
    fn decode(row: &OracleData) -> Result<Value, Error> {
        if row.is_sql_null {
            return Ok(Value::Null);
        }
        match row.column_type {
            OracleType::Number(p, s) => {
                let value = row.str.as_ref().unwrap().clone();
                if p == 0 && s == -127 {
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
                let a = row.str.as_ref().unwrap().clone().parse::<i32>()?;
                return Ok(Value::I32(a));
            }
            OracleType::Float(p) => {
                return if p >= 24 {
                    let a = row.str.as_ref().unwrap().clone().parse::<f64>()?;
                    Ok(Value::F64(a))
                } else {
                    let a = row.str.as_ref().unwrap().clone().parse::<f32>()?;
                    Ok(Value::F32(a))
                }
            }
            OracleType::Date => {
                let a = DateTime::from_str(&row.str.as_ref().unwrap().clone())?;
                return Ok(Value::from(a));
            }
            OracleType::BLOB => {
                if let Some(a) = &row.bin{
                    return Ok(Value::Binary(a.clone()));
                }
                return Ok(Value::Null);
            }
            OracleType::Long => {
                return Ok(Value::String(row.str.as_ref().unwrap().clone()))
            }
            OracleType::CLOB => {
                return Ok(Value::String(row.str.as_ref().unwrap().clone()))
            }
            OracleType::NCLOB => {
                return Ok(Value::String(row.str.as_ref().unwrap().clone()))
            }
            //TODO: more types!
            _ => {
                if row.str.as_ref().is_some() {
                    return Ok(Value::String(row.str.as_ref().unwrap().clone()))
                }
                return Err( Error::from("unimpl"));
            },
        };
    }
}
