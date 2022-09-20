use super::errors::DefaultError;
use super::params::Params;
use super::serializer::row_to_json;
use sqlx::PgConnection;
use sqlx::Connection as Conn;
use sqlx::postgres::PgRow;
use sqlx::Error;


pub struct Postgresql {
    conn: PgConnection
}

impl Postgresql {
    pub async fn new(params: Params) -> Result<Self, DefaultError> { 
        let conn = match PgConnection::connect(&params.uri.as_str()).await{
            Ok(s) => s,
            Err(e) => return Err(DefaultError { message: e.to_string()})
        };
        Ok(Self { conn })
    }
    pub async fn disconnect(self) {
        match self.conn.close().await {
            _ => "ok",
        };
    }
    pub async fn query(&mut self, sql: &str) -> Result<Vec, Error>{
        let records = sqlx::query(sql).fetch_all(&mut self.conn).await?;
        Ok(records.iter().map(|row| row_to_json(row)).collect())
    }
}
