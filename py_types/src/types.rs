use chrono::SecondsFormat;
use pyo3::types::{PyBytes, PyTuple};
use pyo3::{PyObject, Python, ToPyObject};
use quaint::{Value, ValueType};
use serde::Deserialize;

// this type is a placeholder for the actual type
type PyValueList = Vec<PyValue>;

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
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
    Json(String),
    // <body>...</body>
    Xml(String),
    // 00000000-0000-0000-0000-000000000000
    Uuid(String),
    // 00:00:00
    Time(String),
    // 2020-01-01
    Date(String),
    // 2020-01-01T00:00:01
    DateTime(String),
    // 18373737.8274
    Float(f64),
    // Vec<u8>
    Bytes(Vec<u8>),
    // None
    Null,
}

impl<'a> From<Value<'a>> for PyValue {
    fn from(value: Value) -> Self {
        match value.typed {
            ValueType::Boolean(Some(b)) => PyValue::Boolean(b),
            ValueType::Enum(s, _) => s
                .map(|s| PyValue::Enum(s.into_owned()))
                .unwrap_or(PyValue::Null),
            ValueType::EnumArray(s, _) => s
                .map(|v| {
                    let mut list = Vec::new();
                    for item in v {
                        list.push(PyValue::Enum(item.into_owned()));
                    }
                    PyValue::List(list)
                })
                .unwrap_or(PyValue::Null),
            ValueType::Int32(Some(i)) => PyValue::Int(i as i64),
            ValueType::Int64(Some(i)) => PyValue::Int(i),
            ValueType::Array(Some(l)) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(PyValue::from(item));
                }
                PyValue::List(list)
            }
            ValueType::Json(Some(s)) => PyValue::Json(s.to_string()),
            ValueType::Xml(Some(s)) => PyValue::Xml(s.to_string()),
            ValueType::Uuid(Some(s)) => PyValue::Uuid(s.to_string()),
            ValueType::Time(Some(s)) => PyValue::Time(s.to_string()),
            ValueType::Date(Some(s)) => PyValue::Date(s.to_string()),
            ValueType::DateTime(Some(s)) => {
                PyValue::DateTime(s.to_rfc3339_opts(SecondsFormat::Millis, true))
            }
            ValueType::Float(Some(s)) => PyValue::Float(s as f64),
            ValueType::Double(Some(s)) => PyValue::Float(s),
            ValueType::Bytes(Some(b)) => PyValue::Bytes(b.into_owned()),
            ValueType::Text(Some(s)) => PyValue::String(s.to_string()),
            ValueType::Char(Some(s)) => PyValue::String(s.to_string()),
            ValueType::Numeric(Some(s)) => PyValue::String(s.to_string()),
            _ => PyValue::Null,
        }
    }
}

impl<'a> ToPyObject for PyValue {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            PyValue::Boolean(b) => b.to_object(py),
            PyValue::String(s) => s.to_object(py),
            PyValue::Enum(s) => s.to_object(py),
            PyValue::Int(i) => i.to_object(py),
            PyValue::List(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_object(py));
                }
                // convert to tuple python.
                PyTuple::new(py, &list).to_object(py)
                //list.to_object(py)
            }
            PyValue::Json(s) => s.to_object(py),
            PyValue::Xml(s) => s.to_object(py),
            PyValue::Uuid(s) => s.to_object(py),
            PyValue::Time(s) => s.to_object(py),
            PyValue::Date(s) => s.to_object(py),
            PyValue::DateTime(s) => s.to_object(py),
            PyValue::Float(f) => f.to_object(py),
            PyValue::Bytes(b) => PyBytes::new(py, b).to_object(py),
            PyValue::Null => py.None(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, str::FromStr};

    use super::*;
    use bigdecimal::BigDecimal;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use quaint::ast::{EnumName, EnumVariant};
    use quaint::Value;
    use serde_json::json;
    use uuid::{uuid, Uuid};

    #[test]
    fn test_pyvalue_from_value() {
        let value = Value::from(ValueType::Boolean(Some(true)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Boolean(true));

        let value = Value::from(ValueType::Enum(
            Some(EnumVariant::new("red")),
            Some(EnumName::new("xpto", Some("foo"))),
        ));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Enum("red".to_string()));

        let value = Value::from(ValueType::Array(Some(vec![Value::from(ValueType::Enum(
            Some(EnumVariant::new("red")),
            Some(EnumName::new("xpto", Some("foo"))),
        ))])));
        let pyvalue = PyValue::from(value);
        assert_eq!(
            pyvalue,
            PyValue::List(vec![PyValue::Enum("red".to_string())])
        );

        let value = Value::from(ValueType::Int32(Some(1)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Int(1));

        let value = Value::from(ValueType::Int64(Some(1)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Int(1));

        let value = Value::from(ValueType::Array(Some(vec![Value::from(ValueType::Int32(
            Some(1),
        ))])));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::List(vec![PyValue::Int(1)]));

        let value = Value::from(ValueType::Json(Some(json!({"name": "foo"}))));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Json(r#"{"name":"foo"}"#.to_string()));

        let value = Value::from(ValueType::Xml(Some(Cow::from(
            "<body>foo</body>".to_string(),
        ))));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Xml("<body>foo</body>".to_string()));

        let id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
        let value = Value::from(ValueType::Uuid(Some(id)));
        let pyvalue = PyValue::from(value);
        assert_eq!(
            pyvalue,
            PyValue::Uuid("00000000-0000-0000-0000-000000000000".to_string())
        );

        let value = Value::from(ValueType::Time(Some(
            NaiveTime::from_str("12:01:02").unwrap(),
        )));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Time("12:01:02".to_string()));

        let value = Value::from(ValueType::Date(Some(
            NaiveDate::from_str("2022-01-01").unwrap(),
        )));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Date("2022-01-01".to_string()));

        let v = Utc::now();
        let value = Value::from(ValueType::DateTime(Some(v)));
        let pyvalue = PyValue::from(value);
        assert_eq!(
            pyvalue,
            PyValue::DateTime(v.to_rfc3339_opts(SecondsFormat::Millis, true))
        );

        let value = Value::from(ValueType::Float(Some(1.0)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Float(1.0));

        let value = Value::from(ValueType::Double(Some(1.0)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Float(1.0));

        let v: Cow<'_, [u8]> = Cow::from(vec![1, 2, 3]);

        let value = Value::from(ValueType::Bytes(Some(v)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Bytes(vec![1, 2, 3]));

        let v: Cow<'_, str> = Cow::from("foo");

        let value = Value::from(ValueType::Text(Some(v)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::String("foo".to_string()));

        let value = Value::from(ValueType::Char(Some('a')));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::String("a".to_string()));

        let v = BigDecimal::from_str("1.0").unwrap();

        let value = Value::from(ValueType::Numeric(Some(v)));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::String("1.0".to_string()));

        let v: Option<Cow<'_, str>> = None;

        let value = Value::from(ValueType::Text(v));
        let pyvalue = PyValue::from(value);
        assert_eq!(pyvalue, PyValue::Null);
    }
}
