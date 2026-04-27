use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::types::value::MISSING_STRING_VALUE;
use crate::{OracleArgumentValue, OracleValueRef};
use bigdecimal::BigDecimal;
use oracle::sql_type::OracleType;
use rbdc::Error;
use rbdc::datetime::DateTime;
use rbs::Value;
use std::str::FromStr;

pub(crate) fn decode_by_oracle_type(
    oracle_type: &OracleType,
    value: OracleValueRef<'_>,
) -> Result<Value, Error> {
    match oracle_type {
        OracleType::Number(precision, scale) => decode_number(value, *precision, *scale),
        OracleType::Int64 => Ok(Value::I32(i32::decode(value)?)),
        OracleType::Float(precision) => crate::types::float::decode_float(value, *precision),
        OracleType::Date => Ok(Value::from(DateTime::from_str(value.text()?)?)),
        OracleType::BLOB => Ok(crate::types::bytes::decode_binary(value)),
        OracleType::Long | OracleType::CLOB | OracleType::NCLOB => {
            Ok(Value::String(String::decode(value)?))
        }
        _ => value
            .text()
            .map(|text| Value::String(text.to_owned()))
            .map_err(|_| Error::from("unimpl")),
    }
}

fn decode_number(value: OracleValueRef<'_>, precision: u8, scale: i8) -> Result<Value, Error> {
    let value = value
        .text()
        .map_err(|_| Error::from(MISSING_STRING_VALUE))?;

    if precision == 0 && scale == -127 {
        let decimal = BigDecimal::from_str(value).map_err(|e| Error::from(e.to_string()))?;
        if decimal.is_integer() {
            let digits = decimal.digits();
            if (1..=9).contains(&digits) {
                return Ok(Value::I32(value.parse()?));
            }
            if (10..=18).contains(&digits) {
                return Ok(Value::I64(value.parse()?));
            }
        }
        return Ok(Value::String(decimal.to_string()).into_ext("Decimal"));
    }

    if scale > 0 {
        let decimal = BigDecimal::from_str(value).map_err(|e| Error::from(e.to_string()))?;
        return Ok(Value::String(decimal.to_string()).into_ext("Decimal"));
    }

    if (1..=9).contains(&precision) {
        return Ok(Value::I32(value.parse()?));
    }

    if (10..=18).contains(&precision) {
        return Ok(Value::I64(value.parse()?));
    }

    let decimal = BigDecimal::from_str(value).map_err(|e| Error::from(e.to_string()))?;
    Ok(Value::String(decimal.to_string()).into_ext("Decimal"))
}

impl Type for i32 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::number()
    }
}

impl Encode for i32 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::I32(self));
        Ok(IsNull::No)
    }
}

impl Decode for i32 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}

impl Type for i64 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::number()
    }
}

impl Encode for i64 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::I64(self));
        Ok(IsNull::No)
    }
}

impl Decode for i64 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}
