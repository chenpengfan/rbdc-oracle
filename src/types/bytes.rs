use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;
use rbs::Value;

pub(crate) fn decode_binary(value: OracleValueRef<'_>) -> Value {
    value
        .blob()
        .map(|binary| Value::Binary(binary.to_vec()))
        .unwrap_or(Value::Null)
}

impl Type for Vec<u8> {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::binary()
    }
}

impl Encode for Vec<u8> {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::Binary(self));
        Ok(IsNull::No)
    }
}

impl Decode for Vec<u8> {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.blob().unwrap_or_default().to_vec())
    }
}
