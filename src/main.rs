use pysqlx_core::db::uri::Uri;
use pysqlx_core::db::conn::Conn;
use std::error::Error;
use sqlx::Row;
use sqlx::postgres::types::;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let uri = Uri::new("postgresql", "postgresql://postgres:password@localhost:5432/fastapi_prisma")?;
    let conn = Conn::new(uri);
    let mut exec = conn.connect().await?;
    let rows = sqlx::query("select * from peoples").fetch_all(&mut exec).await?;

    for row in rows{
        let columns = row.columns();
        for column in columns {
            println!("{:?}", column);
        }
    }

    Ok(())
}