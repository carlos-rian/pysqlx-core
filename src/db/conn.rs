use super::uri::{Uri};
//use sqlx::{SqliteConnection, Error, Connection};
use sqlx::{PgConnection, Error, Connection};

pub struct Conn {
    uri: Uri
}

impl Conn {
    pub fn new(uri: Uri) -> Self { Self { uri } }

    /// Returns the connect of this [`Conn`].
    ///
    /// # Errors
    ///
    /// This function will return an error if invalid connection uri.
    pub async fn connect(&self) -> Result<PgConnection, Error> {
        let uri = self.uri.uri.as_str();
        let conn = PgConnection::connect(uri).await?;
        Ok(conn)
    }
}
