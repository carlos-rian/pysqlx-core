use crate::{
    base::{
        error::{DBError, PysqlxDBError},
        types::{PysqlxRow, PysqlxRows},
    },
    record::try_convert,
};
use pyo3::prelude::*;
use pythonize::pythonize;
use quaint::{prelude::Queryable, single::Quaint};

#[pyclass]
#[derive(Clone, Debug)]
pub struct Connection {
    conn: Quaint,
}

impl Connection {
    pub async fn _new(uri: String) -> Result<Self, DBError> {
        let con = match Quaint::new(uri.as_str()).await {
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
        Ok(Self { conn: con })
    }
    pub async fn _query(&self, sql: &str) -> Result<PysqlxRows, DBError> {
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
    pub async fn _query_one(&self, sql: &str) -> PysqlxRow {
        let rows = self._query(sql).await?;
        Ok(rows.first())
    }
    pub async fn _execute(&self, sql: &str) -> Result<u64, DBError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(DBError::RawQuery(
                String::from(e.original_code().unwrap_or_default()),
                String::from(e.original_message().unwrap_or_default()),
            )),
        }
    }
}

#[pymethods]
impl Connection {
    pub fn query<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let rows = match slf._query(sql.as_str()).await {
                Ok(r) => r,
                Err(e) => return Err(PyErr::from(PysqlxDBError::from(e))),
            };
            Python::with_gil(|py| {
                let pyrows = pythonize(py, &rows.rows()).unwrap();
                Ok(pyrows)
            })
        })
    }

    pub fn execute<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._execute(sql.as_str()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(PyErr::from(PysqlxDBError::from(e))),
            }
        })
    }
}
