use bigdecimal::BigDecimal;
use oracle::sql_type::OracleType;
use rbdc::{datetime::DateTime, Error};
use rbs::Value;
use std::str::FromStr;

use crate::OracleData;

const MISSING_STRING_VALUE: &str = "Missing string value";
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
                let value = row.str.as_ref()
                .ok_or_else(|| Error::from(MISSING_STRING_VALUE))?;
                if p == 0 && s == -127 {
                    // it means number(*)
                    let dec =
                        BigDecimal::from_str(value).map_err(|e| Error::from(e.to_string()))?;
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
                        BigDecimal::from_str(value).map_err(|e| Error::from(e.to_string()))?;
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
                let value = row.str.as_ref()
                .ok_or_else(|| Error::from(MISSING_STRING_VALUE))?;
                let a = value.parse::<i32>()?;
                return Ok(Value::I32(a));
            }
            OracleType::Float(p) => {
                let value = row.str.as_ref()
                .ok_or_else(|| Error::from(MISSING_STRING_VALUE))?;
            
                return if p >= 24 {
                    let a = value.parse::<f64>()?;
                    Ok(Value::F64(a))
                } else {
                    let a = value.parse::<f32>()?;
                    Ok(Value::F32(a))
                }
            }
            OracleType::Date => {
                let value = row.str.as_ref()
                .ok_or_else(|| Error::from(MISSING_STRING_VALUE))?;
                let a = DateTime::from_str(value)?;
                return Ok(Value::from(a));
            }
            OracleType::BLOB => {
                return Ok(row.bin
                    .as_ref()
                    .map(|bin| Value::Binary((**bin).to_vec()))
                    .unwrap_or(Value::Null));
            }
            OracleType::Long | OracleType::CLOB | OracleType::NCLOB => {
                let value = row
                .str
                .as_ref()
                .ok_or_else(|| Error::from(MISSING_STRING_VALUE))?;
                return Ok(Value::String((**value).to_string()));
            }
            //TODO: more types!
            _ => {
                return row
                .str
                .as_ref()
                .map(|s| Value::String((**s).to_string()))
                .ok_or_else(|| Error::from("unimpl"))
            },
        };
    }
}
