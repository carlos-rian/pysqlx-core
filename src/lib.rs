pub mod base;
pub mod db;
pub mod record;
pub mod value;

use base::error::PysqlxDBError;
use base::types::PysqlxRows;
use db::Connection;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION").to_string();
    // cargo uses "1.0-alpha1" etc. while python uses "1.0.0a1", this is not full compatibility,
    // but it's good enough for now
    // see https://docs.rs/semver/1.0.9/semver/struct.Version.html#method.parse for rust spec
    // see https://peps.python.org/pep-0440/ for python spec
    // it seems the dot after "alpha/beta" e.g. "-alpha.1" is not necessary, hence why this works
    version.replace("-alpha", "a").replace("-beta", "b")
}

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
    m.add("__version__", get_version())?;
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<PysqlxRows>()?;
    m.add_class::<PysqlxDBError>()?;
    Ok(())
}
