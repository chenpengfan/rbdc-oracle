use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;
use rbs::Value;

pub(crate) const MISSING_STRING_VALUE: &str = "Missing string value";

impl Decode for Value {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if value.is_null() {
            return Ok(Value::Null);
        }

        if let Some(oracle_type) = value.type_info().oracle_type() {
            return crate::types::int::decode_by_oracle_type(oracle_type, value);
        }

        Ok(Value::String(value.text()?.to_owned()))
    }
}

impl Encode for Value {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        match self {
            Value::Null => Ok(IsNull::Yes),
            Value::Bool(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::I32(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::I64(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::U32(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::U64(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::F32(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::F64(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::String(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::Binary(value) => {
                value.encode(args)?;
                Ok(IsNull::No)
            }
            Value::Array(value) => {
                Value::Array(value).to_string().encode(args)?;
                Ok(IsNull::No)
            }
            Value::Map(value) => {
                Value::Map(value).to_string().encode(args)?;
                Ok(IsNull::No)
            }
            Value::Ext(type_name, value) => match type_name {
                "Date" => {
                    args.push(OracleArgumentValue::Date(
                        value.into_string().unwrap_or_default(),
                    ));
                    Ok(IsNull::No)
                }
                "DateTime" => {
                    args.push(OracleArgumentValue::DateTime(
                        value.into_string().unwrap_or_default(),
                    ));
                    Ok(IsNull::No)
                }
                "Time" => {
                    args.push(OracleArgumentValue::Time(
                        value.into_string().unwrap_or_default(),
                    ));
                    Ok(IsNull::No)
                }
                "Timestamp" => {
                    args.push(OracleArgumentValue::Timestamp(
                        value.as_u64().unwrap_or_default() as i64,
                    ));
                    Ok(IsNull::No)
                }
                "Decimal" => {
                    args.push(OracleArgumentValue::Decimal(
                        value.into_string().unwrap_or_default(),
                    ));
                    Ok(IsNull::No)
                }
                "Json" => Err(Error::from("unimpl")),
                "Uuid" => {
                    args.push(OracleArgumentValue::Uuid(
                        value.into_string().unwrap_or_default(),
                    ));
                    Ok(IsNull::No)
                }
                _ => Err(Error::from("unimpl")),
            },
        }
    }
}
