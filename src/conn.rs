use crate::base::error::{DBError, PysqlxDBError};
use pyo3::prelude::PyAny;
use pyo3::prelude::*;
use pyo3_asyncio;
use quaint::single::Quaint;

#[pyclass]
pub struct PyConnection {
    pub conn: Quaint,
}

async fn _connect(uri: String) -> Result<Quaint, PysqlxDBError> {
    let conn = match Quaint::new(uri.as_str()).await {
        Ok(r) => r,
        Err(e) => {
            dbg!("{}", e.to_string());
            dbg!("{:?} {:?}", e.original_code(), e.original_message());
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

#[pyfunction]
fn connect<'a>(py: Python<'a>, uri: String) -> Result<&'a PyAny, pyo3::PyErr> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let conn = match _connect(uri).await {
            Ok(r) => r,
            Err(e) => {
                return Err(pyo3::exceptions::PyException::new_err(e.to_string()));
            }
        };
        Python::with_gil(|py| Ok(PyConnection { conn }.into_py(py)))
    })
}
