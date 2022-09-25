use pysqlx_core::base::error::ConversionFailure;
use pysqlx_core::record::try_convert;
use quaint::{prelude::*, single::Quaint};

async fn sql_test() -> Result<(), ConversionFailure> {
    let conn = match Quaint::new(
        "postgresql://postgres:password@localhost:5432/fastapi_prisma?schema=public",
    )
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return Err(ConversionFailure {
                from: "Infinity",
                to: "",
            })
        }
    };
    //let sql = "select * from peoples;";
    let sql = r#"
            INSERT INTO peoples (
                name,
                age,
                created_at,
                updated_at
            ) VALUES (
                'carlos',
                '12',
                '2022-06-29T21:39:35.511',
                '2022-06-29T21:39:35.511'
            );
            SELECT * FROM peoples;
            "#;
    let result = match conn.query_raw(sql, &[]).await {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e.to_string());
            return Err(ConversionFailure {
                from: "not mapping",
                to: "",
            });
        }
    };
    //let columns: Vec<String> = result.columns().iter().map(|c| c.to_string()).collect();
    //for (index, row) in result.into_iter().enumerate() {
    //    println!("line: {:#?} ", index);
    //    for column in &columns {
    //        println!("{:?}", row.get(column.as_str()));
    //    }
    //    println!()
    //}
    println!("{:?}", result.last_insert_id());
    let rows = try_convert(result)?;
    println!("{:#?}", rows);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), ConversionFailure> {
    sql_test().await?;
    Ok(())
}
