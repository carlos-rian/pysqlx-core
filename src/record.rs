use super::base::error::DBError;
use crate::{
    base::{row::PysqlxValue, types::PysqlxRows},
    value::to_value,
};
use quaint::{connector::ResultSet, prelude::ResultRow};
use std::collections::HashMap;

pub fn try_convert(result_set: ResultSet) -> Result<PysqlxRows, DBError> {
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let mut rows = PysqlxRows::new();

    for quaint_row in result_set.into_iter() {
        rows.push(try_convert_row(&columns, quaint_row)?);
    }
    rows.load_types();
    Ok(rows)
}

pub fn try_convert_row(
    columns: &Vec<String>,
    quaint_row: ResultRow,
) -> Result<HashMap<String, PysqlxValue>, DBError> {
    let mut row: HashMap<String, PysqlxValue> = HashMap::new();
    for (index, val) in quaint_row.into_iter().enumerate() {
        let value = to_value(val)?;
        row.insert(columns[index].clone(), value);
    }
    Ok(row)
}
