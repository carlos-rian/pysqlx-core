use crate::{
    base::{
        error::{DBError, PysqlxDBError},
        types::{PysqlxRow, PysqlxRows},
    },
    record::try_convert,
};
use pyo3::prelude::PyResult;
use pyo3::prelude::*;
use pyo3::{prelude::PyAny, types::PyDict};
use pyo3_asyncio;
use quaint::{prelude::Queryable, single::Quaint};

#[pyclass]
pub struct Connection {
    uri: String,
    conn: Option<Quaint>,
}

#[pymethods]
impl Connection {
    #[new]
    pub fn new(uri: String) -> Self {
        Self { uri, conn: None }
    }

    pub fn connect(&self) -> PyResult<&PyAny> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let conn = match Quaint::new(self.uri.as_str()).await {
                Ok(r) => r,
                Err(e) => {
                    dbg!("{}", e.to_string());
                    dbg!("{:?} {:?}", e.original_code(), e.original_message());
                    if e.original_code().is_none() || e.original_message().is_none() {
                        return Err(PysqlxDBError::from(DBError::ConnectionError(
                            String::from("0"),
                            String::from(e.to_string()),
                        ))
                        .into());
                    } else {
                        return Err(PysqlxDBError::from(DBError::ConnectionError(
                            String::from(e.original_code().unwrap_or_default()),
                            String::from(e.original_message().unwrap_or_default()),
                        ))
                        .into());
                    }
                }
            };
            self.conn = Some(conn);
            Python::with_gil(|py| Ok(py.None()))
        })
    }

    pub fn query(&self, sql: &str) -> PyResult<&PyAny> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match &self.conn {
                Some(conn) => {
                    let new_rows = match conn.query_raw(sql, &[]).await {
                        Ok(r) => match try_convert(r) {
                            Ok(mut rows) => {
                                rows.load_types();
                                rows.types()
                            }
                            Err(error) => return Err(PysqlxDBError::from(error).into()),
                        },
                        Err(e) => {
                            return Err(PysqlxDBError::from(DBError::RawQuery(
                                String::from(e.original_code().unwrap_or_default()),
                                String::from(e.original_message().unwrap_or_default()),
                            ))
                            .into())
                        }
                    };
                    Python::with_gil(|py| Ok(new_rows.to_object(py)))
                }
                _ => Err(PysqlxDBError::from(DBError::ConnectionError(
                    String::from("0"),
                    String::from("Connection is not established"),
                ))
                .into()),
            }
        })
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
