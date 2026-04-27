use crate::OracleArgumentValue;
use rbdc::Error;

pub trait Encode {
    fn encode(self, args: &mut Vec<OracleArgumentValue>) -> Result<IsNull, Error>;
}

pub enum IsNull {
    Yes,
    No,
}
