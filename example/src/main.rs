#[macro_use]
extern crate rbatis;
extern crate rbdc;

use std::str::FromStr;

use bigdecimal::BigDecimal;
use rbatis::{py_sql, RBatis};
use rbdc::datetime::DateTime;
use rbdc_oracle::driver::OracleDriver;
use rbdc_oracle::options::OracleConnectOptions;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student{
    pub id_card : i64,
    pub name : String,
    pub score : BigDecimal,
    pub birthday : Option<DateTime>,
    pub sex : i32,
    pub age: Option<i16>,
}
crud!(Student{},"t_student");

#[py_sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = #{sex} ")]
async fn simple_py_sql_select(rb: &RBatis,sex:i32) -> Vec<Student> {}

#[sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = ? ")]
async fn simple_sql_select(rb: &RBatis,sex:i32) -> Vec<Student> {}

impl_delete!(Student{delete_all() => "`where id_card > 0"},"t_student");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StudentProfile{
    pub id_card : i64,
    pub photo: Option<Vec<u8>>,
    pub resume : Option<String>
}
crud!(StudentProfile{},"t_student_profile");

#[sql("select * from t_student_profile where rownum = 1")]
async fn select_first_profile(rb: &RBatis) -> Option<StudentProfile> {}

#[tokio::main]
async fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("");
    let mut rb = RBatis::new();
    rb.init_opt(
        OracleDriver {},
        OracleConnectOptions {
            username: "user".to_string(), 
            password: "123456".to_string(),
            connect_string: "//localhost/school".to_string(),
        },
    )
    .expect("rbatis link database fail");


    //execute init.sql before

    //remode old data
    let deleted =Student::delete_all(&mut rb).await.expect("delete failed");
    println!("{}",deleted.rows_affected);

    //insert some data
    let students = vec![Student{
        id_card : 2300000000,
        name : "小张".to_string(),
        score : BigDecimal::from_str("99.5").unwrap(),
        birthday : Some(DateTime::from_str("2022-09-01 10:33:07").unwrap()),
        sex : 2,
        age: Some(20)
    },Student{
        id_card : 2500000000,
        name : "小强".to_string(),
        score : BigDecimal::from_str("85.5").unwrap(),
        birthday : Some(DateTime::from_str("2002-09-01 16:03:20").unwrap()),
        sex : 1,
        age: Some(20)
    },Student{
        id_card : 2400000000,
        name : "小明".to_string(),
        score : BigDecimal::from_str("91.2").unwrap(),
        birthday : Some(DateTime::from_str("2002-09-01 12:02:49").unwrap()),
        sex : 1,
        age: Some(20)
    },Student{
        id_card : 2200000000,
        name : "小红".to_string(),
        score : BigDecimal::from_str("65.5").unwrap(),
        birthday : Some(DateTime::from_str("2002-09-01 14:03:20").unwrap()),
        sex : 2,
        age: Some(20)
    }];
    //Student::insert_batch(&mut rb,&students) not works
    Student::insert(&mut rb,&students[0]).await.expect("insert failed");
    Student::insert(&mut rb,&students[1]).await.expect("insert failed");
    Student::insert(&mut rb,&students[2]).await.expect("insert failed");
    Student::insert(&mut rb,&students[3]).await.expect("insert failed");

    //sql select
    let select_result = simple_sql_select(&rb,2).await.expect("query failed");
    assert_eq!(select_result.len(),2);
    println!("{:?}",select_result);

    //py_sql select
    let select_result = simple_py_sql_select(&rb,2).await.expect("query failed");
    assert_eq!(select_result.len(),2);
    println!("{:?}",select_result);

    //begin transaction
    let mut tx = rb.acquire_begin().await.unwrap();

    //insert
    let new_stu = Student{
        id_card : 2800000000,
        name : "小王".to_string(),
        score : BigDecimal::from_str("80").unwrap(),
        birthday : Some(DateTime::from_str("2022-10-01 10:00:00.000").unwrap()),
        sex : 1,
        age: Some(20)
    };
    let insert_result = Student::insert(&mut tx,&new_stu).await.expect("insert failed");
    assert_eq!(insert_result.rows_affected,1);

    //update
    let update_stu = Student{
        id_card : 2200000000,
        name : "小红".to_string(),
        score : BigDecimal::from_str("65.5").unwrap(),
        birthday : Some(DateTime::from_str("2002-09-01 14:03:20.000").unwrap()),
        sex : 2,
        age: Some(21)
    };
    let update_result = Student::update_by_column(&mut tx,&update_stu,"id_card").await.expect("update failed");
    assert_eq!(update_result.rows_affected,1);

    //delete
    let delete_result = Student::delete_by_column(&mut tx,"id_card","2500000000").await.expect("delete failed");
    assert_eq!(delete_result.rows_affected,1);

    //select in transaction
    let selected =
        Student::select_by_column(&mut tx,"id_card","2500000000").await.expect("select failed");
    assert_eq!(selected.len(),0);

    //rollback transaction
    tx.rollback().await.expect("rollback failed");

    //select after transaction
    let selected =
        Student::select_by_column(&mut tx,"id_card","2500000000").await.expect("select failed");
    assert_eq!(selected.len(),1);

    //begin new transaction
    let mut tx = rb.acquire_begin().await.unwrap();

    //insert in new transaction
    let new_stu = Student{
        id_card : 2800000000,
        name : "小王".to_string(),
        score : BigDecimal::from_str("80").unwrap(),
        birthday : Some(DateTime::from_str("2022-10-01 10:00:00.000").unwrap()),
        sex : 1,
        age: Some(20)
    };
    let insert_result = Student::insert(&mut tx,&new_stu).await.expect("insert failed");
    assert_eq!(insert_result.rows_affected,1);

    //commit new transaction
    tx.commit().await.expect("commit failed");

    //select after new transaction
    let selected =
        Student::select_by_column(&mut rb,"id_card","2800000000").await.expect("select failed");
    assert_eq!(selected.len(),1);

    //remove new student
    Student::delete_by_column(&mut rb,"id_card","2800000000").await.expect("delete failed");


    //lob test 1: insert empty lob
    let stu_profile = StudentProfile{
        id_card : 2500000000,
        photo: None,
        resume: None
    };
    let insert_result = StudentProfile::insert(&mut rb,&stu_profile).await.expect("insert failed");
    assert_eq!(insert_result.rows_affected,1);

    //query from database
    let inserted : Option<StudentProfile> =
        select_first_profile(&mut rb)
            .await.expect("select failed");
    assert!(inserted.is_some());
    let se = inserted.unwrap();
    assert!(se.clone().resume.is_none());
    assert!(se.clone().photo.is_none());

    //remove
    let delete_result = StudentProfile::delete_by_column(&mut rb,"id_card","2500000000")
        .await.expect("delete failed");
    assert_eq!(delete_result.rows_affected,1);


    //lob test 2: insert big data
    let long_text = "abc".repeat(999_999);
    let long_binary = "def".repeat(999_999).as_bytes().to_vec();
    let stu_profile = StudentProfile{
        id_card : 2300000000,
        photo: Some(long_binary.clone()),
        resume: Some(long_text.clone())
    };
    let insert_result = StudentProfile::insert(&mut rb,&stu_profile).await.expect("insert failed");
    assert_eq!(insert_result.rows_affected,1);

    //query from database
    let inserted : Option<StudentProfile> =
        select_first_profile(&mut rb)
            .await.expect("select failed");
    assert!(inserted.is_some());
    let se = inserted.unwrap();
    assert_eq!(se.clone().resume.unwrap(), long_text);
    assert_eq!(se.clone().photo.unwrap(), long_binary);

    //remove
    let delete_result = StudentProfile::delete_by_column(&mut rb,"id_card","2300000000")
        .await.expect("delete failed");
    assert_eq!(delete_result.rows_affected,1);
}
