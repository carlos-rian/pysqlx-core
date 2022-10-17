use super::base::error::DBError;

use crate::base::row::PysqlxValue;
use crate::base::types::PysqlxRows;
use crate::value::to_value;

use quaint::connector::ResultSet;
use quaint::prelude::ResultRow;

use std::collections::HashMap;

pub fn try_convert(result_set: ResultSet) -> Result<PysqlxRows, DBError> {
    let columns: Vec<String> = result_set.columns().iter().map(|c| c.to_string()).collect();
    let mut rows = PysqlxRows::new();

    for quaint_row in result_set.into_iter() {
        rows.push(try_convert_row(&columns, quaint_row)?);
    }
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

#[cfg(test)]
mod tests {
    use quaint::{prelude::ResultSet, Value};
    use std::borrow::Cow;

    #[test]
    fn test_try_convert_row() {
        let val: Cow<'_, str> = Cow::Owned("test".to_string());
        let cols = vec!["id".to_string(), "name".to_string()];
        let rows = vec![vec![Value::Int32(Some(1)), Value::Text(Some(val))]];
        let result = ResultSet::new(cols, rows);

        let py_result = super::try_convert(result).unwrap();

        assert_eq!(py_result._rows().len(), 1);
    }
}
