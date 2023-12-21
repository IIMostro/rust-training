use mysql::Pool;
use mysql::prelude::Queryable;

#[derive(Debug, PartialEq, Eq)]
struct Payment{
    id: u32,
    amount: String,
    currency: String,
    status: String,
}


fn main() {
    let url = "mysql://root:123456@192.168.205.10:3306/nebula";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    conn.query_map("select id, amount, currency, status from t_payment", |(id, amount, currency, status)| {
        Payment { id, amount, currency, status }
    }).unwrap().iter().for_each(|payment| {
        println!("{:?}", payment);
    });
}
