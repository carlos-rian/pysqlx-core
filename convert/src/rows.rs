use quaint::{connector::ResultSet, prelude::ResultRow};

use std::collections::HashMap;

use crate::columns::{check_column_name, get_column_types};
use py_types::{PySQLxColumnTypes, PySQLxResult, PySQLxRow, PySQLxRows, PySQLxValue};

pub fn convert_result_set(result_set: ResultSet) -> PySQLxResult {
    let mut py_result = PySQLxResult::default();
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let column_types: PySQLxColumnTypes = get_column_types(&columns, &result_set);
    py_result.set_column_types(column_types);
    py_result.last_insert_id = result_set.last_insert_id();

    for row in result_set.into_iter() {
        py_result.push(convert_row(&columns, row));
    }

    py_result
}

pub fn convert_result_set_as_list(result_set: ResultSet) -> PySQLxRows {
    let mut py_result: PySQLxRows = Vec::new();
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();

    for row in result_set.into_iter() {
        py_result.push(convert_row(&columns, row));
    }
    py_result
}

fn convert_row(columns: &Vec<String>, row: ResultRow) -> PySQLxRow {
    let mut data: PySQLxRow = HashMap::new();
    let mut count: i32 = 1;

    for (index, value) in row.into_iter().enumerate() {
        let column = columns[index].clone();

        let mut new_column = check_column_name(&column, index);

        if data.contains_key(new_column.as_str()) {
            new_column = format!("{}_{}", new_column, count);
            count += 1;
        }
        data.insert(new_column, PySQLxValue::from(value));
    }
    data
}

fn _find_sql_param_position_based_on_key(
    sql: String,
    param_keys: Vec<String>,
) -> Vec<(i8, String)> {
    // Find the position of the parameters in the SQL query
    // i8 is the sequence of the parameter in the query
    // for example, if the query is "SELECT * FROM table WHERE id = :x AND name = :y"
    // the position of (0, "x") and (1, "y")
    // if the param repeated in the query, the position will be different
    // for example, if the query is "SELECT * FROM table WHERE id = :x AND name = :x"
    // the position of (0, "x") and (1, "x")
    // if the param is repeated and the query is "SELECT * FROM table WHERE id = :x AND name = :y AND id = :x"
    // the position of (0, "x"), (1, "y"), and (2, "x")
    let mut param_positions: Vec<(i8, String)> = Vec::new();
    // start, end and key
    let mut position: Vec<(usize, usize, String)> = Vec::new();
    for key in param_keys {
        let matches = sql.match_indices(key.as_str());
        for (start, _) in matches {
            let end = start + key.len();
            position.push((start, end, key.clone()));
        }
    }
    println!("unsorted: {:?}", position);
    position.sort_by(|a: &(usize, usize, String), b: &(usize, usize, String)| a.0.cmp(&b.0));
    println!("sorted: {:?}", position);
    for (idx, value) in position.iter().enumerate() {
        param_positions.push((idx as i8, value.2.clone()));
    }
    param_positions
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
        assert_eq!(row[0].get("id").unwrap(), &PySQLxValue::Int(1));
        assert_eq!(
            row[0].get("name").unwrap(),
            &PySQLxValue::String("hello".to_string())
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
        assert_eq!(py_result[0].get("id").unwrap(), &PySQLxValue::Int(1));
        assert_eq!(
            py_result[0].get("name").unwrap(),
            &PySQLxValue::String("hello".to_string())
        );
    }
}
