use pyo3::types::PyString;
use pyo3::{prelude::*, types::PyByteArray};
use quaint::Value;
type PyValueList = Vec<PyValue>;

pub enum PyValue {
    // true, false
    Boolean(bool),
    // text
    String(String),
    // red, green, blue
    Enum(String),
    // 1.0
    Int(i64),
    // Vec<String>,
    List(PyValueList),
    // { "name": "foo", "age": 42 }
    Json(PyString),
    // <body>...</body>
    Xml(PyString),
    // 00000000-0000-0000-0000-000000000000
    Uuid(PyString),
    // 00:00:00
    Time(PyString),
    // 2020-01-01
    Date(PyString),
    // 2020-01-01T00:00:01
    DateTime(PyString),
    // 18373737.8274
    Float(PyString),
    // Vec<u8>
    Bytes(PyByteArray),
    // None
    Null,
}

impl From<Value> for PyValue {
    fn from(value: Value) -> Self {
        let py = Python::with_gil(|py| py);
        match value {
            Value::Boolean(Some(b)) => PyValue::Boolean(b),
            Value::Boolean(None) => PyValue::Null,
            Value::String(s) => PyValue::String(s),
            Value::Enum(s) => PyValue::Enum(s),
            Value::Enum(s) => s
                .map(|s| PyValue::Enum(s.into_owned()))
                .unwrap_or(PyValue::Null),
            Value::Int(i) => PyValue::Int(i),
            Value::List(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(PyValue::from(item));
                }
                PyValue::List(list)
            }
            Value::Json(s) => PyValue::Json(PyString::new(py, s)),
            Value::Xml(s) => PyValue::Xml(PyString::new(py, s)),
            Value::Uuid(s) => PyValue::Uuid(PyString::new(py, s)),
            Value::Time(s) => PyValue::Time(PyString::new(py, s)),
            Value::Date(s) => PyValue::Date(PyString::new(py, s)),
            Value::DateTime(s) => PyValue::DateTime(PyString::new(py, s)),
            Value::Float(s) => PyValue::Float(PyString::new(py, s)),
            Value::Bytes(b) => PyValue::Bytes(PyByteArray::new(py, b)),
            Value::Null => PyValue::Null,
        }
    }
}
