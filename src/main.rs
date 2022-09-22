use pysqlx_core::db::params::Params;
use pysqlx_core::db::conn::Connection;
use std::error::Error;
//use sqlx::Row;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let uri = Params::new("postgresql", "postgresql://postgres:password@localhost:5432/fastapi_prisma")?;
    let mut db = Connection::new(uri).await?;
    let sql = "select id, name from peoples";
    let row = db.query(sql).await?;
    println!("{}", row);
    Ok(())
}