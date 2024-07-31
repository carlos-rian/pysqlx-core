use database::Connection;
use py_types::{PySQLxError, PySQLxInvalidParamError, PySQLxResult, PySQLxStatement};

use env_logger;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use std::env::{set_var, var};

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
fn new(py: Python, uri: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        match Connection::new(uri).await {
            Ok(r) => Ok(r),
            Err(e) => Err(e.to_pyerr()),
        }
    })
}

fn activate_log() {
    for (k, v) in vec![
        ("PYSQL_CORE_INFO", "info"),
        ("PYSQL_CORE_DEBUG", "debug"),
        ("PYSQL_CORE_TRACE", "trace"),
    ]
    .iter()
    {
        match var(k) {
            Ok(_) => {
                set_var("RUST_LOG", v);
                env_logger::init();
                return;
            }
            Err(_) => {}
        }
    }
}

#[pymodule]
fn pysqlx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__version__", get_version())?;
    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<PySQLxResult>()?;
    m.add_class::<PySQLxError>()?;
    m.add_class::<PySQLxInvalidParamError>()?;
    m.add_class::<PySQLxStatement>()?;

    activate_log();

    Ok(())
}
