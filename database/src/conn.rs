use convert::convert_result_set;
use convert::convert_result_set_as_list;
use py_types::PyRows;
use py_types::{py_error, DBError, PySQLXError, PySQLXResult};
use pyo3::prelude::*;
use quaint::connector::IsolationLevel;
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

    pub async fn _query(&self, sql: &str) -> Result<PySQLXResult, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }

    pub async fn _query_as_list(&self, sql: &str) -> Result<PyRows, PySQLXError> {
        match self.conn.query_raw(sql, &[]).await {
            Ok(r) => Ok(convert_result_set_as_list(r)),
            Err(e) => Err(py_error(e, DBError::QueryError)),
        }
    }

    pub async fn _execute(&self, sql: &str) -> Result<u64, PySQLXError> {
        match self.conn.execute_raw(sql, &[]).await {
            Ok(r) => Ok(r),
            Err(e) => Err(py_error(e, DBError::ExecuteError)),
        }
    }

    fn get_isolation(&self, isolation_level: String) -> Result<IsolationLevel, PySQLXError> {
        match isolation_level.as_str() {
            "ReadUncommitted" => Ok(IsolationLevel::ReadUncommitted),
            "ReadCommitted" => Ok(IsolationLevel::ReadCommitted),
            "RepeatableRead" => Ok(IsolationLevel::RepeatableRead),
            "Snapshot" => Ok(IsolationLevel::Snapshot),
            "Serializable" => Ok(IsolationLevel::Serializable),
            _ => {
                return Err(PySQLXError::new(
                    "I1001".to_string(),
                    "Invalid isolation level".to_string(),
                    DBError::IsoLevelError,
                ))
            }
        }
    }

    pub async fn _set_tx_isolation_level(
        &self,
        isolation_level: String,
    ) -> Result<(), PySQLXError> {
        let level = self.get_isolation(isolation_level)?;
        match self.conn.set_tx_isolation_level(level).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::IsoLevelError)),
        }
    }

    pub async fn _start_transaction(
        &self,
        isolation_level: Option<String>,
    ) -> Result<(), PySQLXError> {
        let level: Option<IsolationLevel>;

        if let Some(iso_level) = isolation_level {
            let iso = self.get_isolation(iso_level)?;
            level = Some(iso);
        } else {
            level = None;
        }

        match self.conn.start_transaction(level).await {
            Ok(_) => Ok(()),
            Err(e) => Err(py_error(e, DBError::IsoLevelError)),
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

    pub fn set_tx_isolation_level<'a>(
        &mut self,
        py: Python<'a>,
        isolation_level: String,
    ) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._set_tx_isolation_level(isolation_level).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }

    pub fn start_transaction<'a>(
        &mut self,
        py: Python<'a>,
        isolation_level: String,
    ) -> PyResult<&'a PyAny> {
        let slf = self.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            match slf._set_tx_isolation_level(isolation_level).await {
                Ok(r) => Ok(r),
                Err(e) => Err(e.to_pyerr()),
            }
        })
    }
}
