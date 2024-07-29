//use std::{collections::HashMap, hash::Hash};

use pyo3::{PyObject, Python};
use quaint::Value;
use std::collections::HashMap;

use crate::{convert_to_quaint_values, errors::PySQLxInvalidParamType};

fn _find_sql_param_position_based_on_key(
    sql: &String,
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
        let k = format!(":{}", key.as_str());
        let matches = sql.match_indices(k.as_str());
        for (start, _) in matches {
            let end = start + key.len();
            position.push((start, end, key.clone()));
        }
    }
    position.sort_by(|a: &(usize, usize, String), b: &(usize, usize, String)| a.0.cmp(&b.0));
    for (idx, value) in position.iter().enumerate() {
        param_positions.push((idx as i8, value.2.clone()));
    }
    param_positions
}

fn provider_param(param: &String, idx: i8) -> String {
    match param.as_str() {
        "postgres" => format!("${}", idx + 1),
        "mssql" => format!("@P{}", idx + 1),
        _ => "?".to_string(), // "sqlite" | "mysql"
    }
}

pub fn prepare_sql_typed<'a>(
    py: Python<'a>,
    sql: &String,
    params: &HashMap<String, PyObject>,
    provider: &'a String,
) -> Result<(String, Vec<Value<'static>>), PySQLxInvalidParamType> {
    if params.is_empty() {
        return Ok((sql.to_string(), Vec::new()));
    }
    let mut new_sql = sql.clone();
    let mut new_params = Vec::new();

    let converted_params = match convert_to_quaint_values(py, &params) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let param_positions =
        _find_sql_param_position_based_on_key(sql, params.keys().cloned().collect());

    for (idx, key) in param_positions {
        let value = converted_params.get(&key).unwrap();
        new_sql = new_sql.replace(&format!(":{}", key), provider_param(provider, idx).as_str());
        new_params.push(value.clone());
    }
    Ok((new_sql, new_params))
}
