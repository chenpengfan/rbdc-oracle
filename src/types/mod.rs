//! Conversions between Rust and Oracle types.

mod bool;
mod bytes;
mod float;
mod int;
mod str;
mod uint;
pub(crate) mod value;

#[cfg(test)]
mod test {
    #[test]
    fn test_datetime() {}
}
