use std::collections::HashMap;

use crate::py_types::PySQLxColumnTypes;
use quaint::prelude::ResultSet;
use quaint::{Value, ValueType};

/*
use pyo3::prelude::*;
fn get_type_2(py: Python, value: &Value) -> Bound<'_, PyAny> {
    match value.typed.clone() {
        ValueType::Boolean(_) => py.eval_bound("bool", None, None).unwrap(),
        ValueType::Enum(_, _) => py.eval_bound("str", None, None).unwrap(),
        ValueType::EnumArray(_, _) => py
            .eval_bound(
                r#"
            def sequence():
                from typing import Sequence, Optional
                return Optional[Sequence[str]]
            sequence()
            "#,
                None,
                None,
            )
            .unwrap(),
        ValueType::Text(_) | ValueType::Char(_) | ValueType::Xml(_) => {
            py.eval_bound("str", None, None).unwrap()
        }
        ValueType::Int32(_) | ValueType::Int64(_) => py.eval_bound("int", None, None).unwrap(),
        ValueType::Array(v) => {
            let sub_type = get_type_2(py, &v[0]);
            py.eval_bound(
                r#"
            def sequence():
                from typing import Sequence, Optional
                return Optional[Sequence[sub_type]]
            sequence()
            "#,
                None,
                None,
            )
            .unwrap()
        }
        ValueType::Json(_) => py
            .eval_bound(
                r#"
            def json():
                from typing import Union, Dict, List
                return Union[Dict, List]
            json()
            "#,
                None,
                None,
            )
            .unwrap(),
        ValueType::Uuid(_) => "uuid".to_string(),
        ValueType::Time(_) => "time".to_string(),
        ValueType::Date(_) => "date".to_string(),
        ValueType::DateTime(_) => "datetime".to_string(),
        ValueType::Float(_) => "float".to_string(),
        ValueType::Double(_) => "float".to_string(),
        ValueType::Bytes(_) => "bytes".to_string(),
        ValueType::Numeric(_) => "decimal".to_string(),
    }
}
*/

fn get_type(value: &Value) -> String {
    match value.typed.clone() {
        ValueType::Boolean(_) => "bool".to_string(),
        ValueType::Enum(_, _) => "str".to_string(),
        ValueType::EnumArray(v, _) => match v {
            Some(v) => {
                if v.is_empty() {
                    "array".to_string()
                } else {
                    "array_str".to_string()
                }
            }
            None => "array".to_string(),
        },
        ValueType::Text(_) => "str".to_string(),
        ValueType::Char(_) => "str".to_string(),
        ValueType::Int32(_) => "int".to_string(),
        ValueType::Int64(_) => "int".to_string(),
        ValueType::Array(v) => match v {
            Some(v) => {
                if v.is_empty() {
                    "array".to_string()
                } else {
                    format!("array_{}", get_type(&v[0]))
                }
            }
            None => "array".to_string(),
        },
        ValueType::Json(_) => "json".to_string(),
        ValueType::Xml(_) => "str".to_string(),
        ValueType::Uuid(_) => "uuid".to_string(),
        ValueType::Time(_) => "time".to_string(),
        ValueType::Date(_) => "date".to_string(),
        ValueType::DateTime(_) => "datetime".to_string(),
        ValueType::Float(_) => "float".to_string(),
        ValueType::Double(_) => "float".to_string(),
        ValueType::Bytes(_) => "bytes".to_string(),
        ValueType::Numeric(_) => "decimal".to_string(),
    }
}

fn check_column_is_number(column: &String) -> bool {
    column.parse::<i64>().is_ok() || column.parse::<f64>().is_ok()
}

pub fn check_column_name(column: &String, index: usize) -> String {
    if column.len() == 0 || column == "" || column == "?column?" || column == "?" {
        format!("col_{}", index)
    } else if check_column_is_number(column) {
        format!("col_{}", column.replace("-", "_").replace(".", "_")).replace("__", "_")
    } else {
        column.clone()
    }
}

pub fn get_column_types(columns: &Vec<String>, row: &ResultSet) -> PySQLxColumnTypes {
    let mut data: PySQLxColumnTypes = HashMap::new();
    let mut count: i32 = 1;

    if let Some(first) = row.first() {
        for (index, column) in columns.into_iter().enumerate() {
            let mut new_column = check_column_name(column, index);

            if let Some(value) = first.get(column.as_str()) {
                if data.contains_key(new_column.as_str()) {
                    new_column = format!("{}_{}", new_column, count);
                    count += 1;
                }
                data.insert(new_column, get_type(value));
            }
        }
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
        let columns = vec!["id".to_string(), "name".to_string()];
        let types = get_column_types(&columns, &result);
        assert_eq!(types.get("id").unwrap(), "int");
        assert_eq!(types.get("name").unwrap(), "str");
    }
}
