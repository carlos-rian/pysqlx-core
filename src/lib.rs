mod convert;
mod database;
mod py_types;

use database::tokio_runtime;
use database::Connection;
use log::debug;
use py_types::{PySQLxError, PySQLxInvalidParamError, PySQLxResponse, PySQLxStatement};

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

// async new
#[pyfunction]
async fn new(uri: String) -> PyResult<Connection> {
    debug!("new connection to {}", uri);
    match tokio_runtime().spawn(Connection::new(uri)).await.unwrap() {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_pyerr()),
    }
}

// sync new
#[pyfunction]
fn new_sync(uri: String) -> PyResult<Connection> {
    debug!("new connection to {}", uri);
    match tokio_runtime().block_on(Connection::new(uri)) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_pyerr()),
    }
}

#[pymodule]
fn pysqlx_core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", get_version())?;

    m.add_function(wrap_pyfunction!(new, m)?)?;
    m.add_function(wrap_pyfunction!(new_sync, m)?)?;
    m.add_class::<Connection>()?;
    m.add_class::<PySQLxResponse>()?;
    m.add_class::<PySQLxError>()?;
    m.add_class::<PySQLxInvalidParamError>()?;
    m.add_class::<PySQLxStatement>()?;

    /*
    let _handle = Logger::new(py, Caching::LoggersAndLevels)?
        .filter(LevelFilter::Trace)
        .filter_target("py_types::types".to_owned(), LevelFilter::Debug)
        .filter_target("src".to_owned(), LevelFilter::Debug)
        .install()
        .expect("Someone installed a logger before us :-(");

    debug!("Logger installed");
    info!("Logger installed");
     */
    env_logger::init();
    // pyo3_log::init();
    Ok(())
}
