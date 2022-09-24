use std::collections::HashMap;
//use std::option::Option;
use super::base::error::ConversionFailure;
use crate::{base::row::PysqlxValue, value::to_value};
use quaint::connector::ResultSet;

pub fn try_convert(
    result_set: ResultSet,
) -> Result<Vec<HashMap<String, PysqlxValue>>, ConversionFailure> {
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let mut new_rows: Vec<HashMap<String, PysqlxValue>> = Vec::new();

    if let Some(row) = result_set.into_iter().next() {
        let mut new_row: HashMap<String, PysqlxValue> = HashMap::new();
        for (i, val) in row.into_iter().enumerate() {
            let value = to_value(val)?;
            new_row.insert(columns[i].clone(), value);
        }
        new_rows.push(new_row);
    }
    Ok(new_rows)
}
