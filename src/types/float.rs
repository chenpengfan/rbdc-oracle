use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;
use rbs::Value;

pub(crate) fn decode_float(value: OracleValueRef<'_>, precision: u8) -> Result<Value, Error> {
    if precision >= 24 {
        Ok(Value::F64(f64::decode(value)?))
    } else {
        Ok(Value::F32(f32::decode(value)?))
    }
}

impl Type for f32 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::float()
    }
}

impl Encode for f32 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::F32(self));
        Ok(IsNull::No)
    }
}

impl Decode for f32 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}

impl Type for f64 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::float()
    }
}

impl Encode for f64 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::F64(self));
        Ok(IsNull::No)
    }
}

impl Decode for f64 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}
