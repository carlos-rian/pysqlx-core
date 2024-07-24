use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Display;

use crate::{convert_to_pysqlx_value, PySQLxParamKind, PySQLxValue};

pub type PySQLxRow = HashMap<String, PySQLxValue>;
pub type PySQLxRows = Vec<PySQLxRow>;
pub type PySQLxColumnTypes = HashMap<String, String>;

#[pyclass]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PySQLxResult {
    pub rows: PySQLxRows,
    pub column_types: PySQLxColumnTypes,
}

impl PySQLxResult {
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

impl Display for PySQLxResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PySQLXResult(rows: [...], column_types: {:?})",
            self.column_types
        )
    }
}

impl Default for PySQLxResult {
    fn default() -> Self {
        let rows: PySQLxRows = Vec::new();
        let column_types: PySQLxColumnTypes = HashMap::new();
        Self { rows, column_types }
    }
}

#[pymethods]
impl PySQLxResult {
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

#[pyclass]
pub struct PySQLxParams {
    pub params: Vec<PySQLxValue>,
}

#[pymethods]
impl PySQLxParams {
    #[new]
    pub fn new(py: Python, values: Vec<PyObject>) -> Self {
        let mut params = Vec::new();
        for value in values {
            let kind = PySQLxParamKind::from(
                value
                    .getattr(py, "__name__")
                    .unwrap()
                    .extract::<String>(py)
                    .unwrap(),
            );
            let param = convert_to_pysqlx_value(py, kind, value);
            params.push(param);
        }

        Self { params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_py_sqlx_result() {
        let mut result = PySQLxResult::default();
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
