use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;

use crate::types::PyValue;

pub type PyRow = HashMap<String, PyValue>;
pub type PyRows = Vec<PyRow>;
pub type PyColumnTypes = HashMap<String, String>;

#[pyclass]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PySQLXResult {
    pub rows: PyRows,
    pub column_types: PyColumnTypes,
}

impl PySQLXResult {
    pub fn push(&mut self, row: PyRow) {
        self.rows.push(row);
    }

    pub fn types(&self) -> &PyColumnTypes {
        &self.column_types
    }

    pub fn rows(&self) -> &PyRows {
        &self.rows
    }

    pub fn set_column_types(&mut self, column_types: PyColumnTypes) {
        self.column_types = column_types;
    }
}

impl Display for PySQLXResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PySQLXResult(rows: [...], column_types: {:#?})",
            self.column_types
        )
    }
}

impl Default for PySQLXResult {
    fn default() -> Self {
        let rows: PyRows = Vec::new();
        let column_types: PyColumnTypes = HashMap::new();
        Self { rows, column_types }
    }
}

#[pymethods]
impl PySQLXResult {
    pub fn get_model(&self, py: Python) -> PyObject {
        self.types().to_object(py)
    }

    pub fn get_all(&self, py: Python) -> PyObject {
        self.rows().to_object(py)
    }

    pub fn get_first(&self, py: Python) -> PyObject {
        let first_row = self.rows().get(0);
        match first_row {
            Some(row) => row.to_object(py),
            None => PyDict::new(py).to_object(py),
        }
    }

    pub fn __len__(&self) -> usize {
        self.rows().len()
    }

    pub fn __str__(&self) -> String {
        self.to_string()
    }

    pub fn __repr__(&self) -> String {
        self.to_string()
    }
}
