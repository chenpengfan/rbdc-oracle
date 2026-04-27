use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;

impl Type for String {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::text()
    }
}

impl Encode for String {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::String(self));
        Ok(IsNull::No)
    }
}

impl Encode for &str {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::String(self.to_owned()));
        Ok(IsNull::No)
    }
}

impl Decode for String {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        Ok(value.text()?.to_owned())
    }
}
