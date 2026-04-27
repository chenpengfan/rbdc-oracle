use rbdc::db::Placeholder;
use rbdc_oracle::OracleDriver;

#[test]
fn test_exchange() {
    let sql = "insert into biz_activity (id,name,pc_link,h5_link,pc_banner_img,h5_banner_img,sort,status,remark,create_time,version,delete_flag) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)";
    let driver = OracleDriver;
    let sql = driver.exchange(sql);
    assert_eq!(
        "insert into biz_activity (id,name,pc_link,h5_link,pc_banner_img,h5_banner_img,sort,status,remark,create_time,version,delete_flag) VALUES (:1,:2,:3,:4,:5,:6,:7,:8,:9,:10,:11,:12)",
        sql
    );
}
