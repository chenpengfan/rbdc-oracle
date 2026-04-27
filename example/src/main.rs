#[macro_use]
extern crate rbatis;
extern crate rbdc;

use std::str::FromStr;

use bigdecimal::BigDecimal;
use rbatis::{py_sql, DefaultPool, RBatis};
use rbdc::datetime::DateTime;
use rbdc_oracle::driver::OracleDriver;
use rbdc_oracle::options::OracleConnectOptions;
use rbs::Value;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student {
    pub id_card: i64,
    pub name: String,
    pub score: BigDecimal,
    pub birthday: Option<DateTime>,
    pub sex: i32,
    pub age: Option<i16>,
}
crud!(Student {}, "t_student");

#[py_sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = #{sex} ")]
async fn simple_py_sql_select(rb: &RBatis, sex: i32) -> Vec<Student> {}

#[sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = ? ")]
async fn simple_sql_select(rb: &RBatis, sex: i32) -> Vec<Student> {}

#[html_sql("example.html")]
async fn select_by_condition(rb: &RBatis, name: Option<String>, age: i32) -> Vec<Student> {}

#[html_sql("example.html")]
async fn insert_batch(
    rb: &RBatis,
    students: &Vec<Student>,
) -> Result<rbdc::db::ExecResult, rbdc::Error> {
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentProfile {
    pub id_card: i64,
    pub photo: Option<rbdc::bytes::Bytes>,
    pub resume: Option<String>,
}
crud!(StudentProfile {}, "t_student_profile");

#[sql("select * from t_student_profile where rownum = 1")]
async fn select_first_profile(rb: &RBatis) -> Option<StudentProfile> {}

#[tokio::main]
async fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("");
    let rb = RBatis::new();
    rb.init_option::<OracleDriver, OracleConnectOptions, DefaultPool>(
        OracleDriver {},
        OracleConnectOptions::with_credentials("user", "123456", "//localhost/school"),
    )
    .expect("rbatis link database fail");

    rb.exec(
        "create or replace procedure my_proc(val in nvarchar2,c out number) as \n begin \n select count(*) into c from T_STUDENT where NAME like val; \n end;",
        Vec::with_capacity(0),
    )
    .await
    .unwrap();

    let procedure_res = rb
        .exec(
            "begin\nmy_proc(:name,:val);\nend;",
            vec![Value::String("小张".to_string()), Value::I32(0)],
        )
        .await
        .unwrap();
    assert_eq!(
        procedure_res.last_insert_id.as_array().unwrap()[1],
        Value::String("1".to_string())
    );

    let _ = simple_py_sql_select(&rb, 1).await;
    let _ = simple_sql_select(&rb, 1).await;
    let _ = select_by_condition(&rb, None, 1).await;
    let empty_students = Vec::with_capacity(0);
    let _ = insert_batch(&rb, &empty_students).await;
    let _ = select_first_profile(&rb).await;

    let _ = BigDecimal::from_str("1");
    let _ = DateTime::from_str("2022-09-01 10:33:07");
}
