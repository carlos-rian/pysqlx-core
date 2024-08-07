use std::fmt::Display;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use quaint::error::Error as QuaintError;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DBError {
    QueryError,
    ExecuteError,
    RawCmdError,
    ConnectError,
    IsoLevelError,
    StartTransactionError,
}

impl ToPyObject for DBError {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_string().to_object(py)
    }
}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            DBError::QueryError => "QueryError".to_string(),
            DBError::ExecuteError => "ExecuteError".to_string(),
            DBError::RawCmdError => "RawCmdError".to_string(),
            DBError::ConnectError => "ConnectError".to_string(),
            DBError::IsoLevelError => "IsoLevelError".to_string(),
            DBError::StartTransactionError => "StartTransactionError".to_string(),
        };
        write!(f, "{}", v)
    }
}

#[pyclass(name = "PySQLxError", extends = PyTypeError)]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PySQLxError {
    pub code: String,
    pub message: String,
    pub error: DBError,
}

impl PySQLxError {
    pub fn py_new(code: String, message: String, error: DBError) -> Self {
        Self {
            code,
            message,
            error,
        }
    }
}

#[pymethods]
impl PySQLxError {
    pub fn __str__(&self) -> String {
        format!(
            "PySQLxError(code='{}', message='{}', error='{}')",
            self.code, self.message, self.error
        )
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn code(&self) -> String {
        self.code.clone()
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }

    pub fn error(&self) -> String {
        self.error.to_string()
    }
}

impl PySQLxError {
    pub fn to_pyerr(&self) -> PyErr {
        PyErr::new::<PySQLxError, _>((
            self.code.clone(),
            self.message.clone(),
            self.error.to_string(),
        ))
    }
}

pub fn py_error(err: QuaintError, typ: DBError) -> PySQLxError {
    if err.original_code().is_none() || err.original_message().is_none() {
        PySQLxError::py_new(String::from("0"), String::from(err.to_string()), typ)
    } else {
        PySQLxError::py_new(
            String::from(err.original_code().unwrap_or_default()),
            String::from(err.original_message().unwrap_or_default()),
            typ,
        )
    }
}

#[pyclass(name = "PySQLxInvalidParamError", extends = PyTypeError)]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PySQLxInvalidParamError {
    typ_from: String,
    typ_to: String,
    details: String,
}

impl PySQLxInvalidParamError {
    pub fn to_pyerr(&self) -> PyErr {
        PyErr::new::<PySQLxInvalidParamError, _>((
            self.typ_from.clone(),
            self.typ_to.clone(),
            self.details.clone(),
        ))
    }
}

#[pymethods]
impl PySQLxInvalidParamError {
    #[new]
    pub fn py_new(typ_from: String, typ_to: String, details: String) -> Self {
        Self {
            typ_from,
            typ_to,
            details,
        }
    }

    pub fn __str__(&self) -> String {
        format!(
            "PySQLxInvalidParamError(typ_from='{}', typ_to='{}', details='{}')",
            self.typ_from, self.typ_to, self.details
        )
    }

    pub fn __repr__(&self) -> String {
        self.__str__()
    }

    pub fn typ_from(&self) -> String {
        self.typ_from.clone()
    }

    pub fn typ_to(&self) -> String {
        self.typ_to.clone()
    }

    pub fn details(&self) -> String {
        self.details.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_sqlx_error() {
        let err = PySQLxError::py_new(
            String::from("0"),
            String::from("test"),
            DBError::ConnectError,
        );
        assert_eq!(err.code(), "0");
        assert_eq!(err.message(), "test");
        assert_eq!(err.error(), "ConnectError");
    }
}
