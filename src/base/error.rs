use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[pyclass(name = "PysqlxDBError", extends = PyTypeError)]
#[derive(Error, Clone, Debug)]
pub struct PysqlxDBError {
    code: String,
    error: String,
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
    fn from(err: PysqlxDBError) -> Self {
        PyErr::new::<PysqlxDBError, _>((err.code, err.error, err.type_))
    }
}

#[pymethods]
impl PysqlxDBError {
    #[new]
    pub fn py_new(code: String, error: String, type_: String) -> PysqlxDBError {
        PysqlxDBError { code, error, type_ }
    }

    fn __str__(&self, py: Python) -> PyObject {
        format!(
            "PysqlxDBError(code='{}', message='{}')",
            self.code, self.error
        )
        .into_py(py)
    }

    fn __repr__(&self, py: Python) -> PyObject {
        self.__str__(py)
    }

    #[getter]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    #[getter]
    pub fn error(&self) -> String {
        self.error.clone()
    }

    #[getter]
    pub fn type_(&self) -> String {
        self.type_.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pysqlxdberror() {
        let err = PysqlxDBError {
            code: String::from("0"),
            error: String::from("test"),
            type_: String::from("test"),
        };

        assert_eq!(err.code(), "0");
        assert_eq!(err.error(), "test");
        assert_eq!(err.type_(), "test");
    }

    #[test]
    fn test_db_error_from() {
        let err = DBError::RawQuery(String::from("0"), String::from("test"));
        let pysqlxdberr: PysqlxDBError = err.into();
        assert_eq!(pysqlxdberr.code(), "0");
        assert_eq!(pysqlxdberr.error(), "test");
        assert_eq!(pysqlxdberr.type_(), "RawQuery");
    }
}
