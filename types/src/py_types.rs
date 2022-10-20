use pyo3::types::PyString;
use pyo3::{prelude::*, types::PyByteArray};

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

impl PyValue {}
