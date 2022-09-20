use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::fmt;
use std::error;

#[pyclass(extends=PyException)]
pub struct DefaultError {
    message: String,
}

impl fmt::Debug for DefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DefaultError({:?})", self.message)
    }
}

impl fmt::Display for DefaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for DefaultError {}