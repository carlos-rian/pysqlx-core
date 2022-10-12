use pysqlx_core::{base::error::DBError, test_conn::Connection};
use serde_json;

async fn sql_test() -> Result<(), DBError> {
    let uri = "postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public";
    let sql = "select * from peoples;";

    let conn = Connection::new(uri.to_string()).await?;
    let rows = conn.query(sql).await?;

    rows.rows().iter().for_each(|row| {
        let r = serde_json::to_string(&row).unwrap();
        println!("{:?}", r);
    });

    println!("{:#?}", rows);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), DBError> {
    sql_test().await?;
    Ok(())
}
