use quaint::{connector::ResultSet, prelude::ResultRow};

use std::collections::HashMap;

use crate::columns::get_column_types;
use py_types::{PyColumnTypes, PyRow, PyRows, PySQLXResult, PyValue};

pub fn convert_result_set(result_set: ResultSet) -> PySQLXResult {
    let mut py_result = PySQLXResult::default();
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let column_types: PyColumnTypes = get_column_types(&columns, &result_set);
    py_result.set_column_types(column_types);

    for row in result_set.into_iter() {
        py_result.push(convert_row(&columns, row));
    }

    py_result
}

pub fn convert_result_set_as_list(result_set: ResultSet) -> PyRows {
    let mut py_result: PyRows = Vec::new();
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();

    for row in result_set.into_iter() {
        py_result.push(convert_row(&columns, row));
    }
    py_result
}

fn convert_row(columns: &Vec<String>, row: ResultRow) -> PyRow {
    let mut data: PyRow = HashMap::new();
    for (index, value) in row.into_iter().enumerate() {
        data.insert(columns[index].clone(), PyValue::from(value));
    }
    data
}
