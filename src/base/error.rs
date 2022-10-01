use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[pyclass(name = "PysqlxDBError", extends = PyException)]
#[derive(Error, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct PysqlxDBError {
    #[pyo3(get, set)]
    code: String,
    #[pyo3(get, set)]
    error: String,
    #[pyo3(get, set)]
    type_: String,
}

impl Display for PysqlxDBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PysqlxDBError(code='{}', error='{}', type_='{}')",
            self.code, self.error, self.type_
        )
    }
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
            type_: match error.clone() {
                DBError::RawQuery(_, _) => String::from("RawQuery"),
                DBError::ConnectionError(_, _) => String::from("ConnectionError"),
                DBError::ConversionError(_, _) => String::from("ConversionError"),
            },
        }
    }
}

impl From<PysqlxDBError> for PyErr {
    fn from(err: PysqlxDBError) -> PyErr {
        PyErr::new::<PysqlxDBError, _>((err.code, err.error, err.type_))
    }
}

#[pymethods]
impl PysqlxDBError {
    #[new]
    fn py_new(code: String, error: String, type_: String) -> PysqlxDBError {
        PysqlxDBError { code, error, type_ }
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
