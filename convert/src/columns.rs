use std::collections::HashMap;

use py_types::PyColumnTypes;
use quaint::prelude::ResultSet;
use quaint::Value;

fn get_type(value: &Value) -> String {
    match value {
        Value::Boolean(_) => "bool".to_string(),
        Value::Enum(_) => "str".to_string(),
        Value::Text(_) => "str".to_string(),
        Value::Char(_) => "str".to_string(),
        Value::Int32(_) => "int".to_string(),
        Value::Int64(_) => "int".to_string(),
        Value::Array(v) => match v {
            Some(v) => {
                if v.is_empty() {
                    "array".to_string()
                } else {
                    format!("array_{}", get_type(&v[0]))
                }
            }
            None => "array".to_string(),
        },
        Value::Json(_) => "json".to_string(),
        Value::Xml(_) => "str".to_string(),
        Value::Uuid(_) => "uuid".to_string(),
        Value::Time(_) => "time".to_string(),
        Value::Date(_) => "date".to_string(),
        Value::DateTime(_) => "datetime".to_string(),
        Value::Float(_) => "float".to_string(),
        Value::Double(_) => "float".to_string(),
        Value::Bytes(_) => "bytes".to_string(),
        Value::Numeric(_) => "decimal".to_string(),
    }
}

pub fn get_column_types(columns: &Vec<String>, row: &ResultSet) -> PyColumnTypes {
    let mut data: PyColumnTypes = HashMap::new();
    if let Some(first) = row.first() {
        for column in columns {
            if let Some(value) = first.get(column.as_str()) {
                data.insert(column.clone(), get_type(value));
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
