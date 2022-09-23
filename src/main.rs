use pysqlx_core::base::error::ConversionFailure;
//use async_obdc_mssql_core::base::record::try_convert;
use quaint::{prelude::*, single::Quaint};

async fn sql_test() -> Result<(), ConversionFailure> {
    let conn = match Quaint::new("postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public").await {
        Ok(r) => r,
        Err(_) => return Err(
          ConversionFailure {
              from: "Infinity",
              to: "",
          })  
      };
    let sql = "select * from peoples;";
    let result = match conn.query_raw(sql, &[]).await {
      Ok(r) => r,
      Err(_) => return Err(
        ConversionFailure {
            from: "Infinity",
            to: "",
        })  
    };
    for row in result.into_iter() {
        println!("{:#?}", row)
    }
    //let rows = try_convert(result)?;
    //println!("{:#?}", rows);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), ConversionFailure> {
    sql_test().await?;
    Ok(())
}
