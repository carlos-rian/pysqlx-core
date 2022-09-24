use super::{error::ConversionFailure, row::PysqlxValue};
use std::fmt::Display;
//use quaint::single::Quaint;

pub type PysqlxListValue = Vec<PysqlxValue>;
pub type PysqlxResult<T> = std::result::Result<T, ConversionFailure>;

#[derive(Clone)]
pub struct PysqlxRow {
    column: String,
    value: PysqlxValue,
}

impl PysqlxRow {
    pub fn new(column: String, value: PysqlxValue) -> Self {
        Self { column, value }
    }

    pub fn column(&self) -> &str {
        self.column.as_ref()
    }

    pub fn set_column(&mut self, column: String) {
        self.column = column;
    }

    pub fn value(&self) -> &PysqlxValue {
        &self.value
    }

    pub fn set_value(&mut self, value: PysqlxValue) {
        self.value = value;
    }
}

impl Display for PysqlxRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PysqlxRow(column={}, value={})", self.column, self.value)
    }
}


