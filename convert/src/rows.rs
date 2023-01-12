use quaint::{connector::ResultSet, prelude::ResultRow};

use std::collections::HashMap;

use crate::columns::{check_column_name, get_column_types};
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
    let mut count: i32 = 1;

    for (index, value) in row.into_iter().enumerate() {
        let column = columns[index].clone();

        let mut new_column = check_column_name(&column, index);

        if data.contains_key(new_column.as_str()) {
            new_column = format!("{}_{}", new_column, count);
            count += 1;
        }
        data.insert(new_column, PyValue::from(value));
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use quaint::prelude::Queryable;
    use quaint::single::Quaint;

    #[tokio::test]
    async fn test_get_column_types() {
        let url = "file:///tmp/db.db";
        let quaint = Quaint::new(url).await.unwrap();
        let result = quaint
            .query_raw("SELECT 1 as id, 'hello' as name", &[])
            .await
            .unwrap();
        let py_result = convert_result_set(result);
        assert_eq!(py_result.types().get("id").unwrap(), "int");
        assert_eq!(py_result.types().get("name").unwrap(), "str");
    }

    #[tokio::test]
    async fn test_get_py_sqlx_result() {
        let url = "file:///tmp/db.db";
        let quaint = Quaint::new(url).await.unwrap();
        let result = quaint
            .query_raw("SELECT 1 as id, 'hello' as name", &[])
            .await
            .unwrap();
        let py_result = convert_result_set(result);
        assert_eq!(py_result.__len__() as usize, 1);
        let row = py_result.rows();
        assert_eq!(row[0].get("id").unwrap(), &PyValue::Int(1));
        assert_eq!(
            row[0].get("name").unwrap(),
            &PyValue::String("hello".to_string())
        );
        assert_eq!(row.len(), 1);
    }

    #[tokio::test]
    async fn test_get_raw_as_list() {
        let url = "file:///tmp/db.db";
        let quaint = Quaint::new(url).await.unwrap();
        let result = quaint
            .query_raw("SELECT 1 as id, 'hello' as name", &[])
            .await
            .unwrap();
        let py_result = convert_result_set_as_list(result);
        assert_eq!(py_result.len(), 1);
        assert_eq!(py_result[0].get("id").unwrap(), &PyValue::Int(1));
        assert_eq!(
            py_result[0].get("name").unwrap(),
            &PyValue::String("hello".to_string())
        );
    }
}
