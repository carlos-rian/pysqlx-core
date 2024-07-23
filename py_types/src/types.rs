use core::panic;
use std::borrow::Cow;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pyo3::types::{PyBytes, PyTuple};
use pyo3::{pyclass, pyfunction, FromPyObject, PyAny, PyObject, PyResult, Python, ToPyObject};
use quaint::{Value, ValueType};
use serde::Deserialize;

// this type is a placeholder for the actual type
type PyValueArray = Vec<PySQLxValue>;

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum PySQLxValue {
    // true, false
    Boolean(bool),
    // text
    String(String),
    // red, green, blue
    Enum(String),
    // 1.0
    Int(i64),
    // Vec<String>,
    Array(PyValueArray),
    // { "name": "foo", "age": 42 }
    Json(String),
    // <body>...</body>
    Xml(String),
    // 00000000-0000-0000-0000-000000000000
    Uuid(String),
    // 00:00:00
    Time(NaiveTime),
    // 2020-01-01
    Date(NaiveDate),
    // 2020-01-01T00:00:01
    DateTime(DateTime<Utc>),
    // 18373737.8274
    Float(f64),
    // Vec<u8>
    Bytes(Vec<u8>),
    // None
    Null,
}

impl<'a> From<Value<'a>> for PySQLxValue {
    fn from(value: Value) -> Self {
        match value.typed {
            ValueType::Boolean(Some(b)) => PySQLxValue::Boolean(b),
            ValueType::Enum(s, _) => s
                .map(|s| PySQLxValue::Enum(s.into_owned()))
                .unwrap_or(PySQLxValue::Null),
            ValueType::EnumArray(s, _) => s
                .map(|v| {
                    let mut list = Vec::new();
                    for item in v {
                        list.push(PySQLxValue::Enum(item.into_owned()));
                    }
                    PySQLxValue::Array(list)
                })
                .unwrap_or(PySQLxValue::Null),
            ValueType::Int32(Some(i)) => PySQLxValue::Int(i as i64),
            ValueType::Int64(Some(i)) => PySQLxValue::Int(i),
            ValueType::Array(Some(l)) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(PySQLxValue::from(item));
                }
                PySQLxValue::Array(list)
            }
            ValueType::Json(Some(s)) => PySQLxValue::Json(s.to_string()),
            ValueType::Xml(Some(s)) => PySQLxValue::Xml(s.to_string()),
            ValueType::Uuid(Some(s)) => PySQLxValue::Uuid(s.to_string()),
            ValueType::Time(Some(s)) => PySQLxValue::Time(s),
            ValueType::Date(Some(s)) => PySQLxValue::Date(s),
            ValueType::DateTime(Some(s)) => PySQLxValue::DateTime(s),
            ValueType::Float(Some(s)) => PySQLxValue::Float(s as f64),
            ValueType::Double(Some(s)) => PySQLxValue::Float(s),
            ValueType::Bytes(Some(b)) => PySQLxValue::Bytes(b.into_owned()),
            ValueType::Text(Some(s)) => PySQLxValue::String(s.to_string()),
            ValueType::Char(Some(s)) => PySQLxValue::String(s.to_string()),
            ValueType::Numeric(Some(s)) => PySQLxValue::String(s.to_string()),
            _ => PySQLxValue::Null,
        }
    }
}

impl<'a> ToPyObject for PySQLxValue {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            PySQLxValue::Boolean(b) => b.to_object(py),
            PySQLxValue::String(s) => s.to_object(py),
            PySQLxValue::Enum(s) => s.to_object(py),
            PySQLxValue::Int(i) => i.to_object(py),
            PySQLxValue::Array(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_object(py));
                }
                // convert to tuple python.
                PyTuple::new(py, &list).to_object(py)
                //list.to_object(py)
            }
            PySQLxValue::Json(s) => s.to_object(py),
            PySQLxValue::Xml(s) => s.to_object(py),
            PySQLxValue::Uuid(s) => s.to_object(py),
            PySQLxValue::Time(s) => s.to_object(py),
            PySQLxValue::Date(s) => s.to_object(py),
            PySQLxValue::DateTime(s) => s.to_object(py),
            PySQLxValue::Float(f) => f.to_object(py),
            PySQLxValue::Bytes(b) => PyBytes::new(py, b).to_object(py),
            PySQLxValue::Null => py.None(),
        }
    }
}

impl FromPyObject<'_> for PySQLxValue {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        if let Ok(b) = ob.extract::<bool>() {
            return Ok(PySQLxValue::Boolean(b));
        }
        if let Ok(s) = ob.extract::<String>() {
            return Ok(PySQLxValue::String(s));
        }
        if let Ok(s) = ob.extract::<i64>() {
            return Ok(PySQLxValue::Int(s));
        }
        if let Ok(s) = ob.extract::<Vec<PySQLxValue>>() {
            return Ok(PySQLxValue::Array(s));
        }
        if let Ok(s) = ob.extract::<PyObject>() {
            return Ok(PySQLxValue::Json(s.to_string()));
        }
        if let Ok(s) = ob.extract::<PyObject>() {
            return Ok(PySQLxValue::Xml(s.to_string()));
        }
        if let Ok(s) = ob.extract::<PyObject>() {
            return Ok(PySQLxValue::Uuid(s.to_string()));
        }
        if let Ok(s) = ob.extract::<NaiveTime>() {
            return Ok(PySQLxValue::Time(s));
        }
        if let Ok(s) = ob.extract::<NaiveDate>() {
            return Ok(PySQLxValue::Date(s));
        }
        if let Ok(s) = ob.extract::<DateTime<Utc>>() {
            return Ok(PySQLxValue::DateTime(s));
        }
        if let Ok(s) = ob.extract::<f64>() {
            return Ok(PySQLxValue::Float(s));
        }
        if let Ok(s) = ob.extract::<Vec<u8>>() {
            return Ok(PySQLxValue::Bytes(s));
        }
        if ob.is_none() {
            return Ok(PySQLxValue::Null);
        }

        Err(pyo3::exceptions::PyTypeError::new_err(
            "Invalid type, expected a valid SQLx value",
        )) //todo implement error
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
#[serde(untagged)]
pub enum PySQLxType {
    // true, false
    Boolean,
    // text
    String,
    // red, green, blue
    Enum,
    // 1.0
    Int,
    // Vec<String>,
    List,
    // { "name": "foo", "age": 42 }
    Json,
    // <body>...</body>
    Xml,
    // 00000000-0000-0000-0000-000000000000
    Uuid,
    // 00:00:00
    Time,
    // 2020-01-01
    Date,
    // 2020-01-01T00:00:01
    DateTime,
    // 18373737.8274
    Float,
    // Vec<u8>
    Bytes,
    // None
    Null,
}

fn get_sqlx_type(value: &PySQLxValue) -> PySQLxType {
    match value {
        PySQLxValue::Boolean(_) => PySQLxType::Boolean,
        PySQLxValue::String(_) => PySQLxType::String,
        PySQLxValue::Enum(_) => PySQLxType::Enum,
        PySQLxValue::Int(_) => PySQLxType::Int,
        PySQLxValue::Array(_) => PySQLxType::List,
        PySQLxValue::Json(_) => PySQLxType::Json,
        PySQLxValue::Xml(_) => PySQLxType::Xml,
        PySQLxValue::Uuid(_) => PySQLxType::Uuid,
        PySQLxValue::Time(_) => PySQLxType::Time,
        PySQLxValue::Date(_) => PySQLxType::Date,
        PySQLxValue::DateTime(_) => PySQLxType::DateTime,
        PySQLxValue::Float(_) => PySQLxType::Float,
        PySQLxValue::Bytes(_) => PySQLxType::Bytes,
        PySQLxValue::Null => PySQLxType::Null,
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct PySQLxValueIn {
    pub value: PySQLxValue,
    pub py_type: PySQLxType,
}

// convert PySQLxValue to quaint::Value
impl From<PySQLxValueIn> for Value<'_> {
    fn from(value: PySQLxValueIn) -> Self {
        match value.py_type {
            PySQLxType::Boolean => Value::from(ValueType::Boolean(Some(match value.value {
                PySQLxValue::Boolean(b) => b,
                _ => false,
            }))),
            PySQLxType::Enum => Value::from(ValueType::Enum(
                Some(match value.value {
                    PySQLxValue::Enum(s) => s.into(),
                    _ => panic!("Invalid Enum string"),
                }),
                None,
            )),
            PySQLxType::Int => Value::from(ValueType::Int64(Some(match value.value {
                PySQLxValue::Int(i) => i,
                _ => panic!("Invalid int"),
            }))),
            PySQLxType::List => Value::from(ValueType::Array(Some(match value.value {
                PySQLxValue::Array(l) => {
                    let mut list = Vec::new();
                    for item in l {
                        list.push(Value::from(PySQLxValueIn::from(PySQLxValueIn {
                            value: item.clone(),
                            py_type: get_sqlx_type(&item),
                        })));
                    }
                    list
                }
                _ => panic!("Invalid list"),
            }))),
            PySQLxType::Json => Value::from(ValueType::Json(Some(match value.value {
                PySQLxValue::Json(s) => serde_json::from_str(&s).unwrap(),
                _ => panic!("Invalid JSON string"),
            }))),
            PySQLxType::Xml => Value::from(ValueType::Xml(Some(match value.value {
                PySQLxValue::Xml(s) => Cow::from(s),
                _ => panic!("Invalid XML string"),
            }))),
            PySQLxType::Uuid => Value::from(ValueType::Uuid(Some(match value.value {
                PySQLxValue::Uuid(s) => {
                    uuid::Uuid::parse_str(&s).expect(format!("Invalid UUID: {}", s).as_str())
                }
                _ => panic!("Invalid UUID"),
            }))),
            PySQLxType::Time => Value::from(ValueType::Time(Some(match value.value {
                PySQLxValue::Time(s) => s,
                _ => panic!("Invalid time"),
            }))),
            PySQLxType::Date => Value::from(ValueType::Date(Some(match value.value {
                PySQLxValue::Date(s) => s,
                _ => panic!("Invalid date"),
            }))),
            PySQLxType::DateTime => Value::from(ValueType::DateTime(Some(match value.value {
                PySQLxValue::DateTime(s) => s,
                _ => panic!("Invalid datetime"),
            }))),
            PySQLxType::Float => Value::from(ValueType::Float(Some(match value.value {
                PySQLxValue::Float(f) => f as f32,
                _ => panic!("Invalid float"),
            }))),
            PySQLxType::Bytes => Value::from(ValueType::Bytes(Some(match value.value {
                PySQLxValue::Bytes(b) => Cow::from(b),
                _ => panic!("Invalid bytes"),
            }))),
            PySQLxType::String => Value::from(ValueType::Text(Some(match value.value {
                PySQLxValue::String(s) => Cow::from(s),
                _ => panic!("Invalid string"),
            }))),
            PySQLxType::Null => Value::from(ValueType::Text(None)),
        }
    }
}

#[pyfunction]
pub fn convert_pyobject_to_value_in(py: Python, obj: PyObject) -> PySQLxValueIn {
    let value = obj.extract::<PySQLxValue>(py);
    match value {
        Ok(val) => PySQLxValueIn {
            py_type: get_sqlx_type(&val),
            value: val,
        },
        Err(_) => todo!("Handle error, generate a python exception"),
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
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Boolean(true));

        let value = Value::from(ValueType::Enum(
            Some(EnumVariant::new("red")),
            Some(EnumName::new("xpto", Some("foo"))),
        ));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Enum("red".to_string()));

        let value = Value::from(ValueType::Array(Some(vec![Value::from(ValueType::Enum(
            Some(EnumVariant::new("red")),
            Some(EnumName::new("xpto", Some("foo"))),
        ))])));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(
            pyvalue,
            PySQLxValue::Array(vec![PySQLxValue::Enum("red".to_string())])
        );

        let value = Value::from(ValueType::Int32(Some(1)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Int(1));

        let value = Value::from(ValueType::Int64(Some(1)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Int(1));

        let value = Value::from(ValueType::Array(Some(vec![Value::from(ValueType::Int32(
            Some(1),
        ))])));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Array(vec![PySQLxValue::Int(1)]));

        let value = Value::from(ValueType::Json(Some(json!({"name": "foo"}))));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Json(r#"{"name":"foo"}"#.to_string()));

        let value = Value::from(ValueType::Xml(Some(Cow::from(
            "<body>foo</body>".to_string(),
        ))));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Xml("<body>foo</body>".to_string()));

        let id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
        let value = Value::from(ValueType::Uuid(Some(id)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(
            pyvalue,
            PySQLxValue::Uuid("00000000-0000-0000-0000-000000000000".to_string())
        );

        let value = Value::from(ValueType::Time(Some(
            NaiveTime::from_str("12:01:02").unwrap(),
        )));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(
            pyvalue,
            PySQLxValue::Time(NaiveTime::from_hms_opt(12, 1, 2).expect("invalid"))
        );

        let value = Value::from(ValueType::Date(Some(
            NaiveDate::from_str("2022-01-01").expect("invalid"),
        )));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(
            pyvalue,
            PySQLxValue::Date(NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid"))
        );

        let v = Utc::now();
        let value = Value::from(ValueType::DateTime(Some(v)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::DateTime(v));

        let value = Value::from(ValueType::Float(Some(1.0)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Float(1.0));

        let value = Value::from(ValueType::Double(Some(1.0)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Float(1.0));

        let v: Cow<'_, [u8]> = Cow::from(vec![1, 2, 3]);

        let value = Value::from(ValueType::Bytes(Some(v)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Bytes(vec![1, 2, 3]));

        let v: Cow<'_, str> = Cow::from("foo");

        let value = Value::from(ValueType::Text(Some(v)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::String("foo".to_string()));

        let value = Value::from(ValueType::Char(Some('a')));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::String("a".to_string()));

        let v = BigDecimal::from_str("1.0").unwrap();

        let value = Value::from(ValueType::Numeric(Some(v)));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::String("1.0".to_string()));

        let v: Option<Cow<'_, str>> = None;

        let value = Value::from(ValueType::Text(v));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Null);
    }

    #[test]
    fn test_value_in_to_value() {
        let value = PySQLxValueIn {
            value: PySQLxValue::Boolean(true),
            py_type: PySQLxType::Boolean,
        };
        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::Boolean(Some(true))));

        let value = PySQLxValueIn {
            value: PySQLxValue::Enum("red".to_string()),
            py_type: PySQLxType::Enum,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Enum(Some(EnumVariant::new("red")), None))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Int(1),
            py_type: PySQLxType::Int,
        };
        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::Int64(Some(1))));

        let value = PySQLxValueIn {
            value: PySQLxValue::Array(vec![PySQLxValue::Int(1)]),
            py_type: PySQLxType::List,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Array(Some(vec![Value::from(ValueType::Int64(
                Some(1)
            ))])))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Json(r#"{"name":"foo"}"#.to_string()),
            py_type: PySQLxType::Json,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Json(Some(json!({"name": "foo"}))))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Xml("<body>foo</body>".to_string()),
            py_type: PySQLxType::Xml,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Xml(Some(Cow::from("<body>foo</body>"))))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Uuid("00000000-0000-0000-0000-000000000000".to_string()),
            py_type: PySQLxType::Uuid,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Uuid(Some(uuid!(
                "00000000-0000-0000-0000-000000000000"
            ))))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Time(NaiveTime::from_hms_opt(12, 1, 2).expect("invalid")),
            py_type: PySQLxType::Time,
        };
        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Time(Some(
                NaiveTime::from_hms_opt(12, 1, 2).expect("invalid")
            )))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::Date(NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid")),
            py_type: PySQLxType::Date,
        };

        let v: Value = value.into();
        assert_eq!(
            v,
            Value::from(ValueType::Date(Some(
                NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid")
            )))
        );

        let now = Utc::now();
        let value = PySQLxValueIn {
            value: PySQLxValue::DateTime(now),
            py_type: PySQLxType::DateTime,
        };
        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::DateTime(Some(now))));

        let value = PySQLxValueIn {
            value: PySQLxValue::Float(1.0),
            py_type: PySQLxType::Float,
        };

        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::Float(Some(1.0))));

        let value = PySQLxValueIn {
            value: PySQLxValue::Bytes(vec![1, 2, 3]),
            py_type: PySQLxType::Bytes,
        };

        let v: Value = value.into();

        assert_eq!(
            v,
            Value::from(ValueType::Bytes(Some(Cow::from(vec![1, 2, 3]))))
        );

        let value = PySQLxValueIn {
            value: PySQLxValue::String("foo".to_string()),
            py_type: PySQLxType::String,
        };

        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::Text(Some(Cow::from("foo")))));

        let value = PySQLxValueIn {
            value: PySQLxValue::Null,
            py_type: PySQLxType::Null,
        };

        let v: Value = value.into();
        assert_eq!(v, Value::from(ValueType::Text(None)));
    }
}
