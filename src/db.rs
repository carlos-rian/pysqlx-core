use crate::base::error::{DBError, PysqlxDBError};
use crate::base::types::PysqlxRows;
use crate::record::try_convert;
use pyo3::prelude::*;
use quaint::connector::Queryable;
use quaint::single::Quaint;

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyConnection {
    pub conn: Quaint,
    #[pyo3(get)]
    pub connected: bool,
}

impl PyConnection {
    pub async fn new(uri: String) -> Result<Self, PysqlxDBError> {
        let db = match Quaint::new(uri.as_str()).await {
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
        Ok(Self {
            conn: db,
            connected: true,
        })
    }

    pub async fn query(&self, sql: String) -> Result<PysqlxRows, PysqlxDBError> {
        match self.conn.query_raw(sql.as_str(), &[]).await {
            Ok(r) => match try_convert(r) {
                Ok(mut rows) => {
                    rows.load_types();
                    return Ok(rows);
                }
                Err(error) => return Err(PysqlxDBError::from(error)),
            },
            Err(e) => Err(PysqlxDBError::from(DBError::RawQuery(
                String::from(e.original_code().unwrap_or_default()),
                String::from(e.original_message().unwrap_or_default()),
            ))),
        }
    }
}
