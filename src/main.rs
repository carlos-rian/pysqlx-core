use pysqlx_core::db::params::Params;
use pysqlx_core::db::postgresql::Postgresql;
use std::error::Error;
use sqlx::Row;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let uri = Params::new("postgresql", "postgresql://postgres:password@localhost:5432/fastapi_prisma")?;
    let mut db = Postgresql::new(uri).await?;
    let sql = "select id, name from peoples";
    let rows = db.query(sql).await?;

    for row in rows {
        println!("{:?}", row.len());
        //let columns = row.columns();
        //for column in columns {
        //    println!("{:?}", row.get(0));
        //}
    }

    Ok(())
}