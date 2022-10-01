use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fmt::Debug;
use thiserror::Error;

#[pyclass(extends=PyValueError)]
#[derive(Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PysqlxDBError {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    error: String,
}

#[derive(Error, Clone, Debug)]
pub enum DBError {
    #[error("RawQuery(code={0}, message='{1}')")]
    RawQuery(String, String),
    #[error("ConnectionError(code={0}, message='{1}')")]
    ConnectionError(String, String),
    #[error("ConversionError(message='could not convert from `{0}` to `{1}`')")]
    ConversionError(&'static str, &'static str),
}

impl From<DBError> for PysqlxDBError {
    fn from(error: DBError) -> PysqlxDBError {
        PysqlxDBError {
            code: match error.clone() {
                DBError::RawQuery(code, _) => code,
                DBError::ConnectionError(code, _) => code,
                DBError::ConversionError(_, _) => String::from("0"),
            },
            error: match error.clone() {
                DBError::RawQuery(_, msg) => msg,
                DBError::ConnectionError(_, msg) => msg,
                DBError::ConversionError(_, _) => String::from("0"),
            },
        }
    }
}

impl From<PysqlxDBError> for PyErr {
    fn from(err: PysqlxDBError) -> PyErr {
        PyErr::new::<PysqlxDBError, _>((err.code, err.error))
    }
}

#[pymethods]
impl PysqlxDBError {
    #[new]
    fn py_new(code: String, error: String) -> PysqlxDBError {
        PysqlxDBError { code, error }
    }
    fn __str__(&self) -> String {
        format!(
            "PysqlxDBError(code='{}', message='{}')",
            self.code, self.error
        )
    }
    fn __repr__(&self) -> String {
        self.__str__()
    }
}
