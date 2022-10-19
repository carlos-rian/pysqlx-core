use std::collections::HashMap;

use quaint::prelude::Queryable;
use quaint::single::Quaint;
use quaint::Value;

use serde_json::ser;

#[tokio::main]
async fn main() {
    let uri = "postgresql://postgres:postgrespw@localhost:49153";
    let conn = Quaint::new(uri).await.unwrap();
    let rows = conn.query_raw("SELECT * FROM test2", &[]).await.unwrap();

    let row = rows.into_iter().next().unwrap();
    let id = row.get("id").unwrap();

    let test: HashMap<String, Value> = HashMap::new();

    println!("{:?}", test);
}
