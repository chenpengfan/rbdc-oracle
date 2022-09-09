use bigdecimal::BigDecimal;
use rbatis::{Rbatis, py_sql};
use rbdc::{datetime::FastDateTime};
use rbdc_oracle::driver::OracleDriver;
use rbdc_oracle::options::OracleConnectOptions;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student{
    pub id_card : i64,
    pub name : String,
    pub score : BigDecimal,
    pub birthday : Option<FastDateTime>,
    pub sex : i32,
}

#[py_sql("select name,age,birthday,sex,id_card,score,id_card from t_student where sex = #{sex} ")]
async fn simple_select(rb: &Rbatis,sex:i32) -> Vec<Student> {}

#[tokio::main]
async fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("");
    let rb = Rbatis::new();
    rb.init_opt(
        OracleDriver {},
        OracleConnectOptions {
            username: "user".to_string(), 
            password: "123456".to_string(),
            connect_string: "//localhost/school".to_string(),
        },
    )
    .expect("rbatis link database fail");
    let a = simple_select(&rb,2).await.expect("query failed");
    println!("{:?}",a);
}
