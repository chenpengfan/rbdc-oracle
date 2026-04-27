use crate::OracleValueRef;
use rbdc::Error;

pub trait Decode {
    fn decode(value: OracleValueRef<'_>) -> Result<Self, Error>
    where
        Self: Sized;
}
