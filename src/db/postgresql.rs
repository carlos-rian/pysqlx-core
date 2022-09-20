use super::errors::DefaultError;
use super::params::Params;
use sqlx::PgConnection;
use sqlx::Connection as Conn;


pub struct Postgresql {
    params: Params,
    conn: PgConnection
}

impl Postgresql {
    pub async fn new(params: Params) -> Result<Self, DefaultError> { 
        let conn = match PgConnection::connect(&params.uri.as_str()).await{
            Ok(s) => s,
            Err(e) => return Err(DefaultError { message: e.to_string()})
        };
        Ok(Self { params, conn })
    }
    pub async fn disconnect(self) {
        match self.conn.close().await {
            _ => "ok",
        };
    }
    pub async fn query(&self){//, q: str) {
        let q = "SELECT * FROM peoples";
        let records = sqlx::query!(q).fetch_all(&mut self.conn).await?;
        return records
    }
}
