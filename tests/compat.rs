use oracle::sql_type::OracleType;
use rbdc_oracle::decode::Decode;
use rbdc_oracle::{OracleArgumentValue, OracleArguments, OracleTypeInfo, OracleValue};
use rbs::Value;

#[test]
fn test_compat_encode_bool_matches_old_driver() {
    let args = OracleArguments::from_args(vec![Value::Bool(true)]).expect("encode bool");
    assert!(matches!(
        args.values(),
        [OracleArgumentValue::String(v)] if v == "true"
    ));
}

#[test]
fn test_compat_encode_timestamp_matches_old_driver() {
    let args = OracleArguments::from_args(vec![Value::Ext("Timestamp", Box::new(Value::U64(42)))])
        .expect("encode timestamp");

    assert!(matches!(
        args.values(),
        [OracleArgumentValue::Timestamp(42)]
    ));
}

#[test]
fn test_compat_decode_number_integer() {
    let value = OracleValue::new(
        Some("123".to_owned()),
        None,
        OracleTypeInfo::from_oracle_type(OracleType::Number(3, 0)),
        false,
    );

    let decoded = Value::decode(value.as_ref()).expect("decode integer");
    assert_eq!(decoded, Value::I32(123));
}

#[test]
fn test_compat_decode_number_decimal() {
    let value = OracleValue::new(
        Some("123.45".to_owned()),
        None,
        OracleTypeInfo::from_oracle_type(OracleType::Number(5, 2)),
        false,
    );

    let decoded = Value::decode(value.as_ref()).expect("decode decimal");
    assert_eq!(
        decoded,
        Value::String("123.45".to_owned()).into_ext("Decimal")
    );
}

#[test]
fn test_compat_decode_blob() {
    let value = OracleValue::new(
        None,
        Some(vec![1, 2, 3]),
        OracleTypeInfo::from_oracle_type(OracleType::BLOB),
        false,
    );

    let decoded = Value::decode(value.as_ref()).expect("decode blob");
    assert_eq!(decoded, Value::Binary(vec![1, 2, 3]));
}
