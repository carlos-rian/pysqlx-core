use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;

use super::value::PySQLxValue;

pub type PySQLxRow = HashMap<String, PySQLxValue>;
pub type PySQLxRows = Vec<PySQLxRow>;
pub type PySQLxColumnTypes = HashMap<String, String>;

#[pyclass]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PySQLxResponse {
    pub rows: PySQLxRows,
    pub column_types: PySQLxColumnTypes,
    pub last_insert_id: Option<u64>,
}

impl PySQLxResponse {
    pub fn push(&mut self, row: PySQLxRow) {
        self.rows.push(row);
    }

    pub fn types(&self) -> &PySQLxColumnTypes {
        &self.column_types
    }

    pub fn rows(&self) -> &PySQLxRows {
        &self.rows
    }

    pub fn set_column_types(&mut self, column_types: PySQLxColumnTypes) {
        self.column_types = column_types;
    }
}

impl Display for PySQLxResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PySQLXResult(rows: [...], column_types: {:?}, last_insert_id: {:?})",
            self.column_types, self.last_insert_id
        )
    }
}

impl Default for PySQLxResponse {
    fn default() -> Self {
        let rows: PySQLxRows = Vec::new();
        let column_types: PySQLxColumnTypes = HashMap::new();
        Self {
            rows,
            column_types,
            last_insert_id: None,
        }
    }
}

#[pymethods]
impl PySQLxResponse {
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
            None => PyDict::new_bound(py).to_object(py),
        }
    }

    pub fn get_last_insert_id(&self) -> Option<u64> {
        self.last_insert_id
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
        let mut result = PySQLxResponse::default();
        let mut row = HashMap::new();
        row.insert("id".to_string(), PySQLxValue::Int(1));
        row.insert("name".to_string(), PySQLxValue::String("John".to_string()));
        result.push(row);
        let mut row = HashMap::new();
        row.insert("id".to_string(), PySQLxValue::Int(2));
        row.insert("name".to_string(), PySQLxValue::String("Jane".to_string()));
        result.push(row);
        let mut row = HashMap::new();
        row.insert("id".to_string(), PySQLxValue::Int(3));
        row.insert("name".to_string(), PySQLxValue::String("Jack".to_string()));
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
