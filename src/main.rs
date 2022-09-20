use pysqlx_core::db::params::Uri;
use pysqlx_core::db::postgresql::Conn;
use std::error::Error;
//use sqlx::Row;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let uri = Uri::new("postgresql", "postgresql://postgres:password@localhost:5432/fastapi_prisma")?;
    let conn = Conn::new(uri);
    let mut exec = conn.connect().await?;
    let rows = sqlx::query!("select id, name from peoples").fetch_all(&mut exec).await?;

    for row in rows{
        println!("{:?}", row.);
        //let columns = row.columns();
        //for column in columns {
        //    println!("{:?}", row.get(0));
        //}
    }

    Ok(())
}