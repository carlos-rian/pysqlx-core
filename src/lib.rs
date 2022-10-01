pub mod base;
pub mod conn;
pub mod record;
pub mod test_conn;
pub mod value;
use base::error::PysqlxDBError;
use conn::PyConnection;
use conn::_connect;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
pub fn connect<'a>(py: Python<'a>, uri: String) -> Result<&'a PyAny, pyo3::PyErr> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let conn = match _connect(uri).await {
            Ok(r) => r,
            Err(e) => {
                return Err(PyErr::from(e));
            }
        };
        Python::with_gil(|py| Ok(PyConnection { conn }.into_py(py)))
    })
}

#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_class::<PysqlxDBError>()?;
    Ok(())
}
