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
            "PySQLXResult(rows: [...], column_types: {:?})",
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
    pub fn get_types(&self, py: Python) -> PyObject {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_py_sqlx_result() {
        let mut result = PySQLXResult::default();
        let mut row = HashMap::new();
        row.insert("id".to_string(), PyValue::Int(1));
        row.insert("name".to_string(), PyValue::String("John".to_string()));
        result.push(row);
        let mut row = HashMap::new();
        row.insert("id".to_string(), PyValue::Int(2));
        row.insert("name".to_string(), PyValue::String("Jane".to_string()));
        result.push(row);
        let mut row = HashMap::new();
        row.insert("id".to_string(), PyValue::Int(3));
        row.insert("name".to_string(), PyValue::String("Jack".to_string()));
        result.push(row);
        let mut column_types = HashMap::new();
        column_types.insert("id".to_string(), "int".to_string());
        column_types.insert("name".to_string(), "str".to_string());
        result.set_column_types(column_types);

        assert!(result.rows().len() == 3);
        assert!(result.types().len() == 2);

        assert!(result.__len__() == 3);
    }
}
