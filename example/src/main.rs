use bigdecimal::BigDecimal;
use rbatis::{Rbatis, py_sql};
use rbdc::{datetime::FastDateTime};
use rbdc_oracle::driver::OracleDriver;
use rbdc_oracle::options::OracleConnectOptions;
use serde::{Serialize, Deserialize};
use std::str::FromStr;

#[macro_use]
extern crate rbatis;
extern crate rbdc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student{
    pub id_card : i64,
    pub name : String,
    pub score : BigDecimal,
    pub birthday : Option<FastDateTime>,
    pub sex : i32,
    pub age: Option<i16>,
}
crud!(Student{},"t_student");

#[py_sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = #{sex} ")]
async fn simple_py_sql_select(rb: &Rbatis,sex:i32) -> Vec<Student> {}

#[sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = ? ")]
async fn simple_sql_select(rb: &Rbatis,sex:i32) -> Vec<Student> {}

#[tokio::main]
async fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("");
    let mut rb = Rbatis::new();
    rb.init_opt(
        OracleDriver {},
        OracleConnectOptions {
            username: "user".to_string(), 
            password: "123456".to_string(),
            connect_string: "//localhost/school".to_string(),
        },
    )
    .expect("rbatis link database fail");
    
    //sql select
    let select_result = simple_sql_select(&rb,2).await.expect("query failed");
    assert_eq!(select_result.len(),2);
    println!("{:?}",select_result);
    
    //py_sql select
    let select_result = simple_py_sql_select(&rb,2).await.expect("query failed");
    assert_eq!(select_result.len(),2);
    println!("{:?}",select_result);

    //insert
    let new_stu = Student{
        id_card : 2800000000,
        name : "小王".to_string(),
        score : BigDecimal::from_str("80").unwrap(),
        birthday : Some(FastDateTime::from_str("2022-10-01 10:00:00.000").unwrap()),
        sex : 1,
        age: Some(20)
    };
    let insert_result = Student::insert(&mut rb,&new_stu).await.expect("insert failed");
    assert_eq!(insert_result.rows_affected,1);

    //update
    let update_stu = Student{
        id_card : 2200000000,
        name : "小红".to_string(),
        score : BigDecimal::from_str("65.5").unwrap(),
        birthday : Some(FastDateTime::from_str("2002-09-01 14:03:20.000").unwrap()),
        sex : 2,
        age: Some(21)
    };
    let insert_result = Student::update_by_column(&mut rb,&update_stu,"id_card").await.expect("update failed");
    assert_eq!(insert_result.rows_affected,1);

    //delete
    let insert_result = Student::delete_by_column(&mut rb,"id_card","2500000000").await.expect("delete failed");
    assert_eq!(insert_result.rows_affected,1);

}
