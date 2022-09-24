//use std::option::Option;
use super::base::error::ConversionFailure;
use crate::{base::types::PysqlxRow, value::to_value};
use quaint::{connector::ResultSet, prelude::ResultRow};

pub fn try_convert(result_set: ResultSet) -> Result<Vec<Vec<PysqlxRow>>, ConversionFailure> {
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let mut new_rows: Vec<Vec<PysqlxRow>> = Vec::new();

    for row in result_set.into_iter() {
        new_rows.push(try_convert_row(&columns, row)?);
    }
    Ok(new_rows)
}

pub fn try_convert_row(
    columns: &Vec<String>,
    row: ResultRow,
) -> Result<Vec<PysqlxRow>, ConversionFailure> {
    let mut values: Vec<PysqlxRow> = Vec::new();
    for (index, val) in row.into_iter().enumerate() {
        let value = to_value(val)?;
        values.push(PysqlxRow::new(columns[index].clone(), value));
    }
    Ok(values)
}
