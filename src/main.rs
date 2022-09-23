use pysqlx_core::base::error::ConversionFailure;
use quaint::{prelude::*, single::Quaint};
//use async_obdc_mssql_core::base::record::try_convert;

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
    let columns: Vec<String> = result.columns().iter().map(|c| c.to_string()).collect();
    for (index, row) in result.into_iter().enumerate() {
        for column in &columns { 
            println!("line: {:?}{:?}", index, row.get(column.as_str()))
        }
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
