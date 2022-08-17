use std::str::FromStr;

use bigdecimal::BigDecimal;
use oracle::sql_type::ToSql;
use rbdc::Error;
use rbs::Value;

pub trait Encode {
    fn encode(self, vec: & mut Vec<Box<dyn ToSql>>,) -> Result<(), Error>;
}

impl Encode for Value {
    fn encode(self, vec: & mut Vec<Box<dyn ToSql>>) -> Result<(), Error> {
        match self{
            Value::Ext(t, v) => match t {
                "Date" => {
                    let s = v.as_str().unwrap_or_default();
                    let d = chrono::NaiveDate::parse_from_str(s,"%Y-%m-%d").unwrap();
                    vec.push(Box::new(d));
                },
                "DateTime" => {
                    let s = v.as_str().unwrap_or_default();
                    let d = chrono::NaiveDateTime::parse_from_str(s,"%Y-%m-%d %H:%M:%S").unwrap();
                    vec.push(Box::new(d));
                }
                "Time" => {
                    //TODO: need to fix this
                    let s = v.into_string().unwrap();
                    vec.push(Box::new(s));
                }
                "Decimal" => {
                    let d = BigDecimal::from_str(&v.into_string().unwrap_or_default()).unwrap().to_string();
                    vec.push(Box::new(d));
                }
                "Json" => {
                    return Err(Error::from("unimpl"));
                }
                "Timestamp" => {
                    let t = v.as_u64().unwrap_or_default() as i64;
                    vec.push(Box::new(t));
                }
                "Uuid" => {
                    let d = v.into_string().unwrap();
                    vec.push(Box::new(d));
                }
                _ => {
                    return Err(Error::from("unimpl"));
                },
            }
            Value::String(str)=>{
                vec.push(Box::new(str));
            }
            //TODO: more types!
            _=>{
                vec.push(Box::new(self.to_string()));
            }
        }
        Ok(())
    }
}