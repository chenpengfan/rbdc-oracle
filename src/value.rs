use crate::OracleTypeInfo;
use crate::types::value::MISSING_STRING_VALUE;
use rbdc::Error;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct OracleValueRef<'r>(&'r OracleValue);

impl<'r> OracleValueRef<'r> {
    pub(crate) fn value(value: &'r OracleValue) -> Self {
        Self(value)
    }

    pub fn to_owned(&self) -> OracleValue {
        self.0.clone()
    }

    pub fn type_info(&self) -> Cow<'_, OracleTypeInfo> {
        Cow::Borrowed(&self.0.type_info)
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null
    }

    pub fn text(&self) -> Result<&'r str, Error> {
        self.0
            .text
            .as_deref()
            .ok_or_else(|| Error::from(MISSING_STRING_VALUE))
    }

    pub fn blob(&self) -> Option<&'r [u8]> {
        self.0.binary.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct OracleValue {
    pub(crate) text: Option<Arc<str>>,
    pub(crate) binary: Option<Arc<[u8]>>,
    pub(crate) type_info: OracleTypeInfo,
    pub(crate) is_null: bool,
}

impl OracleValue {
    pub fn new(
        text: Option<String>,
        binary: Option<Vec<u8>>,
        type_info: OracleTypeInfo,
        is_null: bool,
    ) -> Self {
        Self {
            text: text.map(Into::into),
            binary: binary.map(Into::into),
            type_info,
            is_null,
        }
    }

    pub fn as_ref(&self) -> OracleValueRef<'_> {
        OracleValueRef::value(self)
    }
}
