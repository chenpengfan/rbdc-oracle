use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::type_info::{OracleTypeInfo, Type};
use crate::{OracleArgumentValue, OracleValueRef};
use rbdc::Error;

impl Type for bool {
    fn type_info(&self) -> OracleTypeInfo {
        OracleTypeInfo::number()
    }
}

impl Encode for bool {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error> {
        args.push(OracleArgumentValue::String(self.to_string()));
        Ok(IsNull::No)
    }
}

impl Decode for bool {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error> {
        let value = value.text()?;
        Ok(matches!(value, "1" | "true" | "TRUE" | "Y" | "y"))
    }
}
