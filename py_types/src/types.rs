use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use pyo3::types::PyDict;
use pyo3::types::{PyBytes, PyModule, PyTuple};
use pyo3::{intern, PyObject, PyResult, Python, ToPyObject};
use quaint::ast::EnumVariant;
use quaint::{Value, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::PySQLxInvalidParamType;
// this type is a placeholder for the actual type
type PyValueArray = Vec<PySQLxValue>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PySQLxValue {
    // true, false
    Boolean(bool),
    // text
    String(String),
    // red, green, blue
    Enum(String),
    // [red, green, blue]
    EnumArray(Vec<String>),
    // 1.0
    Int(i64),
    // Vec<String>,
    Array(PyValueArray),
    // { "name": "foo", "age": 42 }
    Json(JsonValue),
    // <body>...</body>
    Xml(String),
    // 00000000-0000-0000-0000-000000000000
    Uuid(Uuid),
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
            // numbers
            ValueType::Int32(Some(i)) => PySQLxValue::Int(i as i64),
            ValueType::Int64(Some(i)) => PySQLxValue::Int(i),
            ValueType::Float(Some(s)) => PySQLxValue::Float(s as f64),
            ValueType::Double(Some(s)) => PySQLxValue::Float(s),
            // String value.
            ValueType::Text(Some(s)) => PySQLxValue::String(s.to_string()),
            // enums
            ValueType::Enum(Some(s), _) => PySQLxValue::Enum(s.into_owned()),
            ValueType::EnumArray(Some(l), _) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.into_owned());
                }
                PySQLxValue::EnumArray(list)
            }
            // bytes
            ValueType::Bytes(Some(b)) => PySQLxValue::Bytes(b.into_owned()),
            // boolean
            ValueType::Boolean(Some(b)) => PySQLxValue::Boolean(b),
            // char
            ValueType::Char(Some(s)) => PySQLxValue::String(s.to_string()),
            // array of values (Postgres arrays)
            ValueType::Array(Some(l)) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(PySQLxValue::from(item));
                }
                PySQLxValue::Array(list)
            }
            // Numeric
            ValueType::Numeric(Some(s)) => PySQLxValue::String(s.to_string()),
            // Json
            ValueType::Json(Some(s)) => PySQLxValue::Json(s),
            // Xml
            ValueType::Xml(Some(s)) => PySQLxValue::Xml(s.to_string()),
            // Uuid
            ValueType::Uuid(Some(s)) => PySQLxValue::Uuid(s),
            // date, time, datetime
            ValueType::Time(Some(s)) => PySQLxValue::Time(s),
            ValueType::DateTime(Some(s)) => PySQLxValue::DateTime(s),
            ValueType::Date(Some(s)) => PySQLxValue::Date(s),
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
            PySQLxValue::EnumArray(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_object(py));
                }
                PyTuple::new(py, &list).to_object(py)
            }
            PySQLxValue::Int(i) => i.to_object(py),
            PySQLxValue::Array(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_object(py));
                }
                PyTuple::new(py, &list).to_object(py)
            }
            PySQLxValue::Json(s) => json_value_to_pyobject(py, s).unwrap(),
            PySQLxValue::Xml(s) => s.to_object(py),
            PySQLxValue::Uuid(s) => convert_to_py_uuid(py, s.to_string()).unwrap(),
            PySQLxValue::Time(s) => s.to_object(py),
            PySQLxValue::Date(s) => s.to_object(py),
            PySQLxValue::DateTime(s) => s.to_object(py),
            PySQLxValue::Float(f) => f.to_object(py),
            PySQLxValue::Bytes(b) => PyBytes::new(py, b).to_object(py),
            PySQLxValue::Null => py.None(),
        }
    }
}

// convert PySQLxValue to quaint::Value
impl PySQLxValue {
    pub fn to_value(self) -> Value<'static> {
        match self {
            PySQLxValue::Boolean(b) => Value::from(ValueType::Boolean(Some(b))),
            PySQLxValue::String(s) => Value::from(ValueType::Text(Some(Cow::from(s)))),
            PySQLxValue::Enum(s) => Value::from(ValueType::Enum(Some(EnumVariant::new(s)), None)),
            PySQLxValue::EnumArray(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(EnumVariant::new(item));
                }
                Value::from(ValueType::EnumArray(Some(list), None))
            }
            PySQLxValue::Int(i) => Value::from(ValueType::Int64(Some(i))),
            PySQLxValue::Array(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_value());
                }
                Value::from(ValueType::Array(Some(list)))
            }
            PySQLxValue::Json(s) => Value::from(ValueType::Json(Some(s))),
            PySQLxValue::Xml(s) => Value::from(ValueType::Xml(Some(Cow::from(s)))),
            PySQLxValue::Uuid(s) => Value::from(ValueType::Uuid(Some(s))),
            PySQLxValue::Time(s) => Value::from(ValueType::Time(Some(s))),
            PySQLxValue::Date(s) => Value::from(ValueType::Date(Some(s))),
            PySQLxValue::DateTime(s) => Value::from(ValueType::DateTime(Some(s))),
            PySQLxValue::Float(f) => Value::from(ValueType::Float(Some(f as f32))),
            PySQLxValue::Bytes(b) => Value::from(ValueType::Bytes(Some(Cow::from(b)))),
            PySQLxValue::Null => Value::from(ValueType::Text(None)),
        }
    }
}

// convert serde_json::Value to PyObject
fn json_value_to_pyobject(py: Python, value: &JsonValue) -> PyResult<PyObject> {
    match value {
        JsonValue::String(s) => Ok(s.to_object(py)),
        JsonValue::Number(n) => {
            if n.is_f64() {
                Ok(n.as_f64().unwrap().to_object(py))
            } else if n.is_i64() {
                Ok(n.as_i64().unwrap().to_object(py))
            } else {
                Ok(n.as_u64().unwrap().to_object(py))
            }
        }
        JsonValue::Bool(b) => Ok(b.to_object(py)),
        JsonValue::Null => Ok(py.None()),
        JsonValue::Object(map) => {
            let dict = PyDict::new(py);
            for (key, value) in map {
                dict.set_item(key, json_value_to_pyobject(py, value)?)?;
            }
            Ok(dict.to_object(py))
        }
        JsonValue::Array(vec) => {
            let list: Vec<PyObject> = vec
                .into_iter()
                .map(|v| json_value_to_pyobject(py, v).unwrap())
                .collect();
            Ok(list.to_object(py))
        }
    }
}

fn convert_to_py_uuid(py: Python, r_uuid: String) -> PyResult<PyObject> {
    let uuid_module = PyModule::import(py, "uuid")?;
    let py_uuid = uuid_module.getattr("UUID")?.call1((r_uuid,))?;
    Ok(py_uuid.to_object(py))
}

fn convert_to_rs_uuid(py: Python, value: PyObject) -> Uuid {
    let py_uuid = &value.extract::<String>(py).unwrap();
    Uuid::parse_str(&py_uuid).unwrap()
}

fn convert_json_pyobject_to_serde_value(
    py: Python,
    value: PyObject,
) -> Result<JsonValue, PySQLxInvalidParamType> {
    // the could be a PyDict, PyList, bool, int, float, str or None
    match value.as_ref(py).get_type().name().unwrap() {
        "dict" => {
            let dict = value.extract::<HashMap<String, PyObject>>(py).unwrap();
            let mut map = serde_json::Map::new();
            for (key, value) in dict {
                let v = convert_json_pyobject_to_serde_value(py, value).unwrap();
                map.insert(key, v);
            }
            Ok(JsonValue::Object(map))
        }
        "list" | "tuple" => {
            let list = value.extract::<Vec<PyObject>>(py).unwrap();
            let mut vec = Vec::new();
            for item in list {
                vec.push(convert_json_pyobject_to_serde_value(py, item).unwrap());
            }
            Ok(JsonValue::Array(vec))
        }
        "bool" => Ok(JsonValue::Bool(value.extract::<bool>(py).unwrap())),
        "int" => Ok(JsonValue::Number(serde_json::Number::from(
            value.extract::<i64>(py).unwrap(),
        ))),
        "float" => Ok(JsonValue::Number(
            serde_json::Number::from_f64(value.extract::<f64>(py).unwrap()).unwrap(),
        )),
        "str" => Ok(JsonValue::String(value.extract::<String>(py).unwrap())),
        "date" => {
            let date: NaiveDate = value.extract::<NaiveDate>(py).unwrap();
            Ok(JsonValue::String(date.to_string()))
        }
        "time" => {
            let time: NaiveTime = value.extract::<NaiveTime>(py).unwrap();
            Ok(JsonValue::String(time.to_string()))
        }
        "datetime" => {
            let datetime: DateTime<Utc> = convert_to_datetime(py, value);
            Ok(JsonValue::String(datetime.to_rfc3339()))
        }
        "uuid" => {
            let rs_uuid = convert_to_rs_uuid(py, value);
            Ok(JsonValue::String(rs_uuid.to_string()))
        }
        "bytes" => {
            let bytes = value.extract::<Vec<u8>>(py).unwrap();
            Ok(JsonValue::String(base64::encode(bytes)))
        }
        "decimal" => {
            let decimal = value.extract::<String>(py).unwrap();
            Ok(JsonValue::String(decimal))
        }
        "enum" => {
            let enum_value = value
                .getattr(py, intern!(py, "value"))
                .unwrap()
                .extract::<String>(py)
                .unwrap();
            Ok(JsonValue::String(enum_value))
        }
        "NoneType" => Ok(JsonValue::Null),
        value_type => Err(PySQLxInvalidParamType::py_new(
            value_type.to_string(),
            "json".to_string(),
            "Unsupported type".to_string(),
        )),
    }
}

fn convert_to_datetime(py: Python, value: PyObject) -> DateTime<Utc> {
    match value.extract::<DateTime<Utc>>(py) {
        //datetime with timezone
        Ok(v) => v,
        Err(_) => {
            let naive_dt = value.extract::<NaiveDateTime>(py).unwrap();
            //datetime without timezone
            DateTime::<Utc>::from_utc(naive_dt, Utc)
        }
    }
}

pub fn convert_to_pysqlx_value(
    py: Python,
    kind: PySQLxParamKind,
    value: PyObject,
) -> Result<PySQLxValue, PySQLxInvalidParamType> {
    match kind {
        PySQLxParamKind::Boolean => Ok(PySQLxValue::Boolean(value.extract::<bool>(py).unwrap())),
        PySQLxParamKind::String => Ok(PySQLxValue::String(value.extract::<String>(py).unwrap())),
        PySQLxParamKind::Enum => Ok(PySQLxValue::Enum(value.extract::<String>(py).unwrap())),
        PySQLxParamKind::EnumArray => {
            let list = value.extract::<Vec<String>>(py).unwrap();
            Ok(PySQLxValue::EnumArray(list))
        }
        PySQLxParamKind::Int => Ok(PySQLxValue::Int(value.extract::<i64>(py).unwrap())),
        PySQLxParamKind::Array => {
            let list = value.extract::<Vec<PyObject>>(py).unwrap();
            let mut pysqlx_list = Vec::new();
            for item in list {
                pysqlx_list.push(match convert_to_pysqlx_value(py, kind.clone(), item) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                });
            }
            Ok(PySQLxValue::Array(pysqlx_list))
        }
        PySQLxParamKind::Json => Ok(PySQLxValue::Json(
            match convert_json_pyobject_to_serde_value(py, value) {
                Ok(v) => v,
                Err(e) => return Err(e),
            },
        )),
        PySQLxParamKind::Xml => Ok(PySQLxValue::Xml(value.extract::<String>(py).unwrap())),
        PySQLxParamKind::Uuid => {
            let rs_uuid = convert_to_rs_uuid(py, value);
            Ok(PySQLxValue::Uuid(rs_uuid))
        }
        PySQLxParamKind::Time => Ok(PySQLxValue::Time(value.extract::<NaiveTime>(py).unwrap())),
        PySQLxParamKind::Date => Ok(PySQLxValue::Date(value.extract::<NaiveDate>(py).unwrap())),
        PySQLxParamKind::DateTime => Ok(PySQLxValue::DateTime(convert_to_datetime(py, value))),
        PySQLxParamKind::Float => Ok(PySQLxValue::Float(value.extract::<f64>(py).unwrap())),
        PySQLxParamKind::Bytes => Ok(PySQLxValue::Bytes(value.extract::<Vec<u8>>(py).unwrap())),
        PySQLxParamKind::Null => Ok(PySQLxValue::Null),
        PySQLxParamKind::UnsupportedType(t) => Err(PySQLxInvalidParamType::py_new(
            t,
            "str|int|float|etc".to_string(),
            "Unsupported type, check the documentation".to_string(),
        )),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum PySQLxParamKind {
    Boolean,
    String,
    Enum,
    EnumArray,
    Int,
    Array,
    Json,
    Xml,
    Uuid,
    Time,
    Date,
    DateTime,
    Float,
    Bytes,
    Null,
    UnsupportedType(String),
}

impl From<String> for PySQLxParamKind {
    fn from(kind: String) -> Self {
        // kind string is python class Type name
        match kind.to_lowercase().as_str() {
            "bool" => PySQLxParamKind::Boolean,
            "str" => PySQLxParamKind::String,
            "enum" => PySQLxParamKind::Enum,
            "enumarray" => PySQLxParamKind::EnumArray,
            "int" => PySQLxParamKind::Int,
            "tuple" => PySQLxParamKind::Array,
            "dict" | "list" => PySQLxParamKind::Json,
            "xml" => PySQLxParamKind::Xml,
            "uuid" => PySQLxParamKind::Uuid,
            "time" => PySQLxParamKind::Time,
            "date" => PySQLxParamKind::Date,
            "datetime" => PySQLxParamKind::DateTime,
            "float" => PySQLxParamKind::Float,
            "bytes" => PySQLxParamKind::Bytes,
            "decimal" => PySQLxParamKind::String,
            "none" => PySQLxParamKind::Null,
            t => PySQLxParamKind::UnsupportedType(t.to_string()),
        }
    }
}

pub fn convert_to_quaint_values(
    py: Python,
    values: &HashMap<String, PyObject>,
) -> Result<HashMap<String, Value<'static>>, PySQLxInvalidParamType> {
    let mut params = HashMap::new();
    for (key, value) in values {
        let value_type = value.as_ref(py).get_type().name().unwrap().to_string();
        let kind = PySQLxParamKind::from(value_type);
        match convert_to_pysqlx_value(py, kind, value.clone_ref(py)) {
            Ok(v) => {
                params.insert(key.clone(), v.to_value());
            }
            Err(e) => return Err(e),
        }
    }

    Ok(params)
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
        assert_eq!(
            pyvalue,
            PySQLxValue::Json(serde_json::json!({"name":"foo"}))
        );

        let value = Value::from(ValueType::Xml(Some(Cow::from(
            "<body>foo</body>".to_string(),
        ))));
        let pyvalue = PySQLxValue::from(value);
        assert_eq!(pyvalue, PySQLxValue::Xml("<body>foo</body>".to_string()));

        let id: Uuid = uuid!("00000000-0000-0000-0000-000000000000");
        let value = Value::from(ValueType::Uuid(Some(id)));
        let pyvalue = PySQLxValue::from(value);
        assert!(matches!(pyvalue, PySQLxValue::Uuid(_)));

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

    /*#[test] // this test is not working because of the Python::with_gil
    fn test_pyobject_to_pysqlx_value() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let value = true.to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Boolean".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Boolean(true));

            let value = "foo".to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "String".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::String("foo".to_string()));

            let value = "red".to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Enum".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Enum("red".to_string()));

            let value = vec!["red", "green"].to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "EnumArray".to_string(), value);
            assert_eq!(
                pyvalue,
                PySQLxValue::EnumArray(vec!["red".to_string(), "green".to_string()])
            );

            let value = 1.to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Int".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Int(1));

            let value = vec![1, 2].to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Array".to_string(), value);
            assert_eq!(
                pyvalue,
                PySQLxValue::Array(vec![PySQLxValue::Int(1), PySQLxValue::Int(2)])
            );

            let value = json!({"name": "foo"}).to_string().to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Json".to_string(), value);
            assert_eq!(
                pyvalue,
                PySQLxValue::Json(serde_json::json!({"name":"foo"}))
            );

            let value = "<body>foo</body>".to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Xml".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Xml("<body>foo</body>".to_string()));

            let value = Uuid::new_v4().to_string().to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Uuid".to_string(), value);
            assert!(matches!(pyvalue, PySQLxValue::Uuid(_)));

            let value = NaiveTime::from_hms_opt(12, 1, 2).to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Time".to_string(), value);
            assert_eq!(
                pyvalue,
                PySQLxValue::Time(NaiveTime::from_hms_opt(12, 1, 2).expect("invalid"))
            );

            let value = NaiveDate::from_ymd_opt(2022, 1, 1).to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Date".to_string(), value);
            assert_eq!(
                pyvalue,
                PySQLxValue::Date(NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid"))
            );

            let value = Utc::now().to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "DateTime".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::DateTime(Utc::now()));

            let value = 1.0.to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Float".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Float(1.0));

            let value = vec![1, 2, 3].to_object(py);
            let pyvalue = convert_to_pysqlx_value(py, "Bytes".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Bytes(vec![1, 2, 3]));

            let value = py.None();
            let pyvalue = convert_to_pysqlx_value(py, "Null".to_string(), value);
            assert_eq!(pyvalue, PySQLxValue::Null);
        })
    }*/
}
