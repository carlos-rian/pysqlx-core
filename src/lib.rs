pub mod base;
pub mod conn;
pub mod db;
pub mod record;
pub mod value;

use base::error::PysqlxDBError;
use base::types::PysqlxRows;
use conn::Connection;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn new<'a>(py: Python<'a>, uri: String) -> Result<&'a PyAny, pyo3::PyErr> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        match Connection::_new(uri).await {
            Ok(r) => Ok(r),
            Err(e) => return Err(PyErr::from(PysqlxDBError::from(e))),
        }
    })
}

#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<PysqlxRows>()?;
    m.add_class::<PysqlxDBError>()?;
    Ok(())
}
