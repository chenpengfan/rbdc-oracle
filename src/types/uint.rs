use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;

impl Type for u32 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::number()
    }
}

impl Encode for u32 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::U32(self));
        Ok(IsNull::No)
    }
}

impl Decode for u32 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}

impl Type for u64 {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::number()
    }
}

impl Encode for u64 {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::U64(self));
        Ok(IsNull::No)
    }
}

impl Decode for u64 {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.parse()?)
    }
}
