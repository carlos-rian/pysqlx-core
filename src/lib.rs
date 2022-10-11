pub mod base;
pub mod db;
pub mod record;
pub mod test_conn;
pub mod value;
mod pyvalue;
use base::error::PysqlxDBError;
use db::PyConnection;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
pub fn connect<'a>(py: Python<'a>, uri: String) -> Result<&'a PyAny, pyo3::PyErr> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let conn = match PyConnection::new(uri).await {
            Ok(r) => r,
            Err(e) => {
                return Err(PyErr::from(e));
            }
        };
        Python::with_gil(|py| Ok(conn.into_py(py)))
    })
}

#[pyfunction]
pub fn query<'a>(py: Python<'a>, conn: Py<PyAny>, sql: String) -> Result<&'a PyAny, pyo3::PyErr> {
    let db = conn.extract::<PyConnection>(py)?;
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let rows = match db.query(sql).await {
            Ok(r) => r,
            Err(e) => {
                return Err(PyErr::from(e));
            }
        };
        Python::with_gil(|py| Ok(rows.into_py(py)))
    })
}

#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_class::<PysqlxDBError>()?;
    Ok(())
}
