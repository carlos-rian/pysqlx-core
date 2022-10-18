use super::error::DBError;
use super::row::get_pysqlx_type;
use super::row::PysqlxValue;
use pyo3::conversion::ToPyObject;
use pyo3::prelude::*;
use pythonize::pythonize;
use std::collections::{hash_map::RandomState, HashMap};
use std::option::Option;
use std::result::Result;

pub type PysqlxListValue = Vec<PysqlxValue>;
pub type PysqlxResult<T> = Result<T, DBError>;
pub type PysqlxRow = Result<Option<HashMap<String, PysqlxValue>>, DBError>;

#[pyclass]
pub struct PysqlxRows {
    pub types: HashMap<String, String>,
    pub rows: Vec<HashMap<String, PysqlxValue>>,
}

impl std::ops::Deref for PysqlxRows {
    type Target = Vec<HashMap<String, PysqlxValue>>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl PysqlxRows {
    pub fn new() -> Self {
        let rows: Vec<HashMap<String, PysqlxValue>> = Vec::new();
        let types: HashMap<String, String> = HashMap::new();
        Self { rows, types }
    }

    pub fn push(&mut self, row: HashMap<String, PysqlxValue>) {
        self.rows.push(row);
    }

    pub fn load_types(&mut self) {
        if let Some(first_row) = self.rows.get(0) {
            for (column, value) in first_row {
                self.types
                    .insert(column.clone(), get_pysqlx_type(value.clone()));
            }
        }
    }

    pub fn _rows(&self) -> &[HashMap<String, PysqlxValue, RandomState>] {
        self.rows.as_ref()
    }

    pub fn _first(&self) -> Option<HashMap<String, PysqlxValue>> {
        let first_row = self.rows.get(0);
        match first_row {
            Some(row) => Some(row.clone()),
            None => None,
        }
    }

    pub fn _types(&self) -> &HashMap<String, String> {
        &self.types
    }
}

#[pymethods]
impl PysqlxRows {
    pub fn get_types(&self, py: Python) -> PyObject {
        self._types().clone().to_object(py)
    }

    pub fn get_all(&self, py: Python) -> PyResult<Py<PyAny>> {
        Ok(pythonize(py, &self._rows()).unwrap())
    }

    pub fn get_first(&self, py: Python) -> PyResult<Py<PyAny>> {
        Ok(pythonize(py, &self._first()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pysqlx_rows() {
        let mut rows = PysqlxRows::new();
        let mut row = HashMap::new();
        row.insert("id".to_string(), PysqlxValue::Int(1));
        row.insert("name".to_string(), PysqlxValue::String("test".to_string()));
        rows.push(row);
        rows.load_types();
        assert_eq!(rows._types().len(), 2);
        assert_eq!(
            rows._first().unwrap().get("id").unwrap(),
            &PysqlxValue::Int(1)
        );
        assert_eq!(rows._rows().len(), 1);
    }
}
