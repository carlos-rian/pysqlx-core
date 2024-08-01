use database::Connection;
use py_types::{PySQLxError, PySQLxInvalidParamError, PySQLxResponse, PySQLxStatement};

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3_log;

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
async fn new(uri: String) -> PyResult<Connection> {
    match Connection::new(uri).await {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_pyerr()),
    }
}

#[pymodule]
fn pysqlx_core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", get_version())?;
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<PySQLxResponse>()?;
    m.add_class::<PySQLxError>()?;
    m.add_class::<PySQLxInvalidParamError>()?;
    m.add_class::<PySQLxStatement>()?;

    pyo3_log::init();

    Ok(())
}
