use pyo3::exceptions::PyException;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::fmt::Debug;
use std::fmt::Display;
use thiserror::Error;

#[pyclass(extends=PyException)]
#[derive(Clone, Error)]
pub struct PysqlxDBError {
    #[pyo3(get)]
    code: String,
    #[pyo3(get)]
    error: String,
}

#[derive(Error, Clone)]
pub enum DBError {
    #[error("RawQuery(code={0}, message='{1}')")]
    RawQuery(String, String),
    #[error("ConnectionError(code={0}, message='{1}')")]
    ConnectionError(String, String),
    #[error("ConversionError(message='could not convert from `{0}` to `{1}`')")]
    ConversionError(&'static str, &'static str),
}

impl Debug for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::RawQuery(code, msg) => write!(f, "RawQuery(code={}, message='{}')", code, msg),
            DBError::ConnectionError(code, msg) => {
                write!(f, "ConnectionError(code={}, message='{}')", code, msg)
            }
            DBError::ConversionError(from, to) => {
                write!(
                    f,
                    "ConversionError(message='could not convert from `{0}` to `{1}`')",
                    from, to
                )
            }
        }
    }
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

impl Debug for PysqlxDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PysqlxDBError(code={}, message='{}')",
            self.code, self.error
        )
    }
}

impl Display for PysqlxDBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PysqlxDBError(code={}, message='{}')",
            self.code, self.error
        )
    }
}

impl From<PysqlxDBError> for PyErr {
    fn from(err: PysqlxDBError) -> Self {
        PyTypeError::new_err(err.to_string())
    }
}
