use convert::convert_result_set;
use py_types::{DBError, PySQLXError, PySQLXResult};
use pyo3::prelude::*;
use quaint::error::Error;
use quaint::prelude::*;
use quaint::single::Quaint;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Connection {
    conn: Quaint,
}

impl Connection {
    fn error(e: Error, typ: DBError) -> PySQLXError {
        if e.original_code().is_none() || e.original_message().is_none() {
            PySQLXError::py_new(String::from("0"), String::from(e.to_string()), typ)
        } else {
            PySQLXError::py_new(
                String::from(e.original_code().unwrap_or_default()),
                String::from(e.original_message().unwrap_or_default()),
                typ,
            )
        }
    }

    pub async fn new(uri: String) -> Result<Self, PySQLXError> {
        let conn = match Quaint::new(uri.as_str()).await {
            Ok(r) => r,
            Err(e) => return Err(Self::error(e, DBError::ConnectionError)),
        };
        Ok(Self { conn })
    }

    pub async fn _query(&self, sql: &str) -> Result<PySQLXResult, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set(r)),
            Err(e) => Err(Self::error(e, DBError::QueryError)),
        }
    }

    pub async fn _execute(&self, sql: &str) -> Result<u64, PySQLXError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(Self::error(e, DBError::ExecuteError)),
        }
    }
}

#[pymethods]
impl Connection {
    pub fn query<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._query(sql.as_str()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn execute<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._execute(sql.as_str()).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn query_as_list<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let rows = match slf._query(sql.as_str()).await {
                Ok(r) => r,
                Err(e) => return Err(e.to_pyerr()),
            };
            Python::with_gil(|py| {
                let pyrows = rows.get_all(py);
                Ok(pyrows)
            })
        })
    }

    pub fn is_healthy(&self) -> bool {
        self.conn.is_healthy()
    }

    pub fn requires_isolation_first(&self) -> bool {
        self.conn.requires_isolation_first()
    }
}
