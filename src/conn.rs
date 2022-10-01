use crate::base::error::{DBError, PysqlxDBError};
use pyo3::prelude::*;
use quaint::single::Quaint;

#[pyclass]
pub struct PyConnection {
    pub conn: Quaint,
}

pub async fn _connect(uri: String) -> Result<Quaint, PysqlxDBError> {
    let conn = match Quaint::new(uri.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            if e.original_code().is_none() || e.original_message().is_none() {
                return Err(PysqlxDBError::from(DBError::ConnectionError(
                    String::from("0"),
                    String::from(e.to_string()),
                )));
            } else {
                return Err(PysqlxDBError::from(DBError::ConnectionError(
                    String::from(e.original_code().unwrap_or_default()),
                    String::from(e.original_message().unwrap_or_default()),
                )));
            }
        }
    };
    Ok(conn)
}
