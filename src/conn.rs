use crate::{
    base::{
        error::DBError,
        types::{PysqlxRow, PysqlxRows},
    },
    record::try_convert,
};
use quaint::{prelude::Queryable, single::Quaint};

pub struct Connection {
    conn: Quaint,
}

impl Connection {
    async fn new(uri: String) -> Result<Self, DBError> {
        let conn = match Quaint::new(uri.as_str()).await {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e.to_string());
                println!("{:?} {:?}", e.original_code(), e.original_message());
                if e.original_code().is_none() || e.original_message().is_none() {
                    return Err(DBError::ConnectionError(
                        String::from("0"),
                        String::from(e.to_string()),
                    ));
                } else {
                    return Err(DBError::ConnectionError(
                        String::from(e.original_code().unwrap_or_default()),
                        String::from(e.original_message().unwrap_or_default()),
                    ));
                }
            }
        };
        Ok(Self { conn })
    }
    pub async fn query(&self, sql: &str) -> Result<PysqlxRows, DBError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => match try_convert(r) {
                Ok(mut rows) => {
                    rows.load_types();
                    return Ok(rows);
                }
                Err(error) => return Err(error),
            },
            Err(e) => Err(DBError::RawQuery(
                String::from(e.original_code().unwrap_or_default()),
                String::from(e.original_message().unwrap_or_default()),
            )),
        }
    }
    pub async fn query_one(&self, sql: &str) -> PysqlxRow {
        let rows = self.query(sql).await?;
        Ok(rows.first())
    }
    pub async fn execute(&self, sql: &str) -> Result<u64, DBError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(DBError::RawQuery(
                String::from(e.original_code().unwrap_or_default()),
                String::from(e.original_message().unwrap_or_default()),
            )),
        }
    }
}
