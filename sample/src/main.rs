use rbatis::{Rbatis, py_sql};
use rbdc::{datetime::FastDateTime};
use rbdc_oracle::driver::OracleDriver;
use rbdc_oracle::options::OracleConnectOptions;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Student{
    pub name : String,
    pub age : i32,
    pub birthday : Option<FastDateTime>,
    pub sex : i32,
}

#[py_sql("select name,age,birthday,sex from t_student where sex = #{sex} ")]
async fn simple_select(rb: &Rbatis,sex:i32) -> Vec<Student> {}

#[tokio::main]
async fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("");
    let rb = Rbatis::new();
    rb.link_opt(
        OracleDriver {},
        OracleConnectOptions {
            username: "user".to_string(), 
            password: "123456".to_string(),
            connect_string: "//localhost/school".to_string(),
        },
    )
    .await
    .expect("rbatis link database fail");
    let a = simple_select(&rb,2).await.expect("quert failed");
    println!("{:?}",a);
}
