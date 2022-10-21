use convert::convert_result_set;
use convert::convert_result_set_as_list;
use py_types::PyRows;
use py_types::{py_error, DBError, PySQLXError, PySQLXResult};
use pyo3::prelude::*;
use quaint::prelude::*;
use quaint::single::Quaint;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Connection {
    conn: Quaint,
}

impl Connection {
    pub async fn new(uri: String) -> Result<Self, PySQLXError> {
        let conn = match Quaint::new(uri.as_str()).await {
            Ok(r) => r,
            Err(e) => return Err(py_error(e, DBError::ConnectionError)),
        };
        Ok(Self { conn })
    }

    async fn _query(&self, sql: &str) -> Result<PySQLXResult, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }

    async fn _query_as_list(&self, sql: &str) -> Result<PyRows, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set_as_list(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }

    async fn _execute(&self, sql: &str) -> Result<u64, PySQLXError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(py_error(e, DBError::ExecuteError)),
        }
    }

    async fn _raw_cmd(&self, sql: &str) -> Result<(), PySQLXError> {
        match self.conn.raw_cmd(sql).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::ExecuteError)),
        }
    }
}

#[pymethods]
impl Connection {
    pub fn query<'a>(&self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
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
            let rows = match slf._query_as_list(sql.as_str()).await {
                Ok(r) => r,
                Err(e) => return Err(e.to_pyerr()),
            };
            Python::with_gil(|py| {
                let pyrows = rows.to_object(py);
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

    pub fn raw_cmd<'a>(&mut self, py: Python<'a>, sql: String) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._raw_cmd(sql.as_str()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }
}
