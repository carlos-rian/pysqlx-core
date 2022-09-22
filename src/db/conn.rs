use super::errors::DefaultError;
use super::params::Params;
use serde_json::Value;
use super::serializer::row_to_json;
use sqlx::AnyConnection;
use sqlx::Connection as Conn;
use sqlx::Error;


pub struct Connection {
    conn: AnyConnection
}

impl Connection {
    pub async fn new(params: Params) -> Result<Self, DefaultError> { 
        let conn = match AnyConnection::connect(&params.uri.as_str()).await{
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
    pub async fn query(&mut self, sql: &str) -> Result<Value, Error>{
        let row = sqlx::query(sql).fetch_one(&mut self.conn).await?;
        Ok(row_to_json(row))
    }
}
