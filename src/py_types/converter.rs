use super::errors::PySQLxInvalidParamError;
use super::value::PySQLxValue;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use log::info;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyAnyMethods, PyDict, PyModule, PyTuple, PyType, PyTypeMethods};
use pyo3::{intern, Bound, PyObject, PyResult, Python, ToPyObject};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

pub fn get_python_type_name(value: &Bound<'_, PyAny>) -> String {
    value.get_type().qualname().unwrap().to_string()
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
    Numeric,
    Null,
    UnsupportedType(String),
}

impl PySQLxParamKind {
    fn is_enum_instance(py: Python, obj: &Bound<'_, PyAny>) -> bool {
        let enum_mod = PyModule::import_bound(py, "enum").unwrap();
        let enum_class = enum_mod.getattr("Enum").unwrap();

        if let Ok(enum_class) = enum_class.downcast::<PyType>() {
            match obj.as_ref().is_instance(enum_class) {
                Ok(is_instance) => is_instance,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    fn validate_tuple_is_same_type(py: Python, tuple: &Bound<PyTuple>) -> (bool, String, bool) {
        let first_item = tuple.get_item(0).unwrap();
        let kind = get_python_type_name(&first_item);
        for (idx, item) in tuple.iter().enumerate().skip(1) {
            let item_kind = get_python_type_name(&item);
            if kind != item_kind {
                //return (false, format!("the tuple must have the same type, the first item is a {} and the current item is a {}", kind, item_kind));
                return (false, format!("The tuple (array) must have the same type, the first item is a {} and the current item position {} is a {}", kind, idx, item_kind), false);
            }
        }
        (true, String::new(), Self::is_enum_instance(py, &first_item))
    }

    fn from(py: Python, value: &Bound<'_, PyAny>, provider: &str) -> Self {
        // kind string is python class Type name
        info!("{:?}", value);
        match get_python_type_name(value).as_str() {
            "bool" => PySQLxParamKind::Boolean,
            "str" => PySQLxParamKind::String,
            "int" => PySQLxParamKind::Int,
            "tuple" => {
                if provider != "postgresql" {
                    return PySQLxParamKind::UnsupportedType(
                        "The tuple (array) is only supported in PostgreSQL".to_string(),
                    );
                }
                // check if the tuple is empty
                let tuple = value.extract::<Bound<PyTuple>>().unwrap();
                if tuple.is_empty() {
                    return PySQLxParamKind::Array;
                }

                // check if the tuple has the same type
                let (is_same_type, msg, is_enum) = Self::validate_tuple_is_same_type(py, &tuple);

                if !is_same_type {
                    return PySQLxParamKind::UnsupportedType(msg);
                }

                if is_enum {
                    return PySQLxParamKind::EnumArray;
                }

                PySQLxParamKind::Array
            }
            "dict" | "list" => PySQLxParamKind::Json,
            "xml" => PySQLxParamKind::Xml,
            "time" => PySQLxParamKind::Time,
            "date" => PySQLxParamKind::Date,
            "datetime" => PySQLxParamKind::DateTime,
            "float" => PySQLxParamKind::Float,
            "bytes" => PySQLxParamKind::Bytes,
            "UUID" => PySQLxParamKind::Uuid,
            "Decimal" => PySQLxParamKind::Numeric,
            "NoneType" => PySQLxParamKind::Null,
            "Enum" => PySQLxParamKind::Enum,
            t => {
                if Self::is_enum_instance(py, &value) {
                    PySQLxParamKind::Enum
                } else {
                    PySQLxParamKind::UnsupportedType(t.to_string())
                }
            }
        }
    }
}

pub struct Converters;
impl Converters {
    pub fn convert_json_value_to_pyobject(py: Python, value: &JsonValue) -> PyResult<PyObject> {
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
                let dict = PyDict::new_bound(py);
                for (key, value) in map {
                    dict.set_item(key, Self::convert_json_value_to_pyobject(py, value)?)?;
                }
                Ok(dict.to_object(py))
            }
            JsonValue::Array(vec) => {
                let list: Vec<PyObject> = vec
                    .into_iter()
                    .map(|v| Self::convert_json_value_to_pyobject(py, v).unwrap())
                    .collect();
                Ok(list.to_object(py))
            }
        }
    }

    pub fn convert_to_py_uuid(py: Python, r_uuid: String) -> PyResult<PyObject> {
        let uuid_module = PyModule::import_bound(py, "uuid")?;
        let py_uuid = uuid_module
            .getattr("UUID")
            .unwrap()
            .call1((r_uuid,))
            .unwrap();
        Ok(py_uuid.to_object(py))
    }

    pub fn convert_to_rs_uuid(value: &Bound<'_, PyAny>) -> Uuid {
        let py_uuid = value.to_string();
        Uuid::parse_str(&py_uuid).unwrap()
    }

    pub fn convert_to_py_decimal(py: Python, r_decimal: BigDecimal) -> PyResult<PyObject> {
        let decimal_module = PyModule::import_bound(py, "decimal")?;
        let py_decimal = decimal_module
            .getattr("Decimal")
            .unwrap()
            .call1((r_decimal.to_string(),))?;
        Ok(py_decimal.to_object(py))
    }

    pub fn convert_to_rs_decimal(value: &Bound<'_, PyAny>) -> BigDecimal {
        let py_decimal = value.to_string();
        <BigDecimal as std::str::FromStr>::from_str(&py_decimal).unwrap()
    }

    fn convert_json_pyobject_to_serde_value(
        py: Python,
        value: &Bound<'_, PyAny>,
    ) -> Result<JsonValue, PySQLxInvalidParamError> {
        // the could be a PyDict, PyList, bool, int, float, str or None
        match get_python_type_name(value).as_str() {
            "dict" => {
                let dict = value.extract::<HashMap<String, Bound<PyAny>>>().unwrap();
                let mut map = serde_json::Map::new();
                for (key, value) in dict {
                    let v = Self::convert_json_pyobject_to_serde_value(py, &value).unwrap();
                    map.insert(key, v);
                }
                Ok(JsonValue::Object(map))
            }
            "list" | "tuple" => {
                let list = value.extract::<Vec<Bound<PyAny>>>().unwrap();
                let mut vec = Vec::new();
                for item in list {
                    vec.push(Self::convert_json_pyobject_to_serde_value(py, &item).unwrap());
                }
                Ok(JsonValue::Array(vec))
            }
            "bool" => Ok(JsonValue::Bool(value.extract::<bool>().unwrap())),
            "int" => Ok(JsonValue::Number(serde_json::Number::from(
                value.extract::<i64>().unwrap(),
            ))),
            "float" => Ok(JsonValue::Number(
                serde_json::Number::from_f64(value.extract::<f64>().unwrap()).unwrap(),
            )),
            "str" => Ok(JsonValue::String(value.extract::<String>().unwrap())),
            "date" => {
                let date: NaiveDate = value.extract::<NaiveDate>().unwrap();
                Ok(JsonValue::String(date.to_string()))
            }
            "time" => {
                let time: NaiveTime = value.extract::<NaiveTime>().unwrap();
                Ok(JsonValue::String(time.to_string()))
            }
            "datetime" => {
                let datetime: DateTime<Utc> = Self::convert_to_datetime(value);
                Ok(JsonValue::String(datetime.to_rfc3339()))
            }
            "uuid" => {
                let rs_uuid = Self::convert_to_rs_uuid(value);
                Ok(JsonValue::String(rs_uuid.to_string()))
            }
            "bytes" => {
                let bytes = value.extract::<Vec<u8>>().unwrap();
                Ok(JsonValue::String(base64::encode(bytes)))
            }
            "decimal" => {
                let decimal = value.extract::<String>().unwrap();
                Ok(JsonValue::String(decimal))
            }
            "enum" => {
                let enum_value = value
                    .getattr(intern!(py, "value"))
                    .unwrap()
                    .extract::<String>()
                    .unwrap();
                Ok(JsonValue::String(enum_value))
            }
            "NoneType" => Ok(JsonValue::Null),
            value_type => Err(PySQLxInvalidParamError::py_new(
                value_type.to_string(),
                "json".to_string(),
                "Unsupported type".to_string(),
                None,
            )),
        }
    }

    fn convert_to_datetime(value: &Bound<'_, PyAny>) -> DateTime<Utc> {
        match value.extract::<DateTime<Utc>>() {
            //datetime with timezone
            Ok(v) => v,
            Err(_) => {
                let naive_dt = value.extract::<NaiveDateTime>().unwrap();
                //datetime without timezone
                DateTime::<Utc>::from_utc(naive_dt, Utc)
            }
        }
    }

    fn is_number_instance(value: &Bound<'_, PyAny>) -> bool {
        match get_python_type_name(value).as_str() {
            "int" | "float" => true,
            _ => false,
        }
    }

    fn convert_enum_to_string(
        py: Python,
        value: &Bound<'_, PyAny>,
    ) -> Result<String, PySQLxInvalidParamError> {
        let enum_name = value.as_ref().getattr(intern!(py, "name")).unwrap();
        let enum_value = value.as_ref().getattr(intern!(py, "value")).unwrap();

        info!(
            "converting Enum(name={}({:?}), value={}({:?})",
            enum_name.get_type().name().unwrap(),
            enum_name,
            enum_value.get_type().name().unwrap(),
            enum_value,
        );

        if enum_value.get_type().name().unwrap() == "str" {
            Ok(enum_value.to_string())
        } else if Self::is_number_instance(&enum_value) {
            Ok(enum_name.to_string())
        } else {
            Err(PySQLxInvalidParamError::py_new(
                "enum".to_string(),
                "str".to_string(),
                r#"
                    Unsupported enum type. 
                    The postgres enum should be a `string`. 
                    If the python enum.value is a string (str), we will use the enum.value.
                    If the python enum.value is a number (int, float), we will use the enum.name.
                    Otherwise, an error will be raised."#
                    .to_string(),
                None,
            ))
        }
    }

    fn convert_pyobject_to_pysqlx_value(
        py: Python,
        kind: PySQLxParamKind,
        value: &Bound<'_, PyAny>,
        provider: &str,
    ) -> Result<PySQLxValue, PySQLxInvalidParamError> {
        match kind {
            PySQLxParamKind::Boolean => Ok(PySQLxValue::Boolean(value.extract::<bool>().unwrap())),
            PySQLxParamKind::String => Ok(PySQLxValue::String(value.extract::<String>().unwrap())),
            PySQLxParamKind::Enum => Ok(PySQLxValue::Enum(
                match Self::convert_enum_to_string(py, value) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                },
            )),
            PySQLxParamKind::EnumArray => {
                let list = value.extract::<Bound<PyTuple>>().unwrap();
                let mut enum_list = Vec::new();
                for item in list {
                    enum_list.push(match Self::convert_enum_to_string(py, &item) {
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    });
                }

                Ok(PySQLxValue::EnumArray(enum_list))
            }
            PySQLxParamKind::Int => Ok(PySQLxValue::Int(value.extract::<i64>().unwrap())),
            PySQLxParamKind::Array => {
                let list = value.extract::<Vec<Bound<PyAny>>>().unwrap();
                let mut pysqlx_list = Vec::new();
                for item in list {
                    pysqlx_list.push(
                        match Self::convert_pyobject_to_pysqlx_value(
                            py,
                            PySQLxParamKind::from(py, &item, provider),
                            &item,
                            provider,
                        ) {
                            Ok(v) => v,
                            Err(e) => return Err(e),
                        },
                    );
                }
                Ok(PySQLxValue::Array(pysqlx_list))
            }
            PySQLxParamKind::Json => Ok(PySQLxValue::Json(
                match Self::convert_json_pyobject_to_serde_value(py, value) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                },
            )),
            PySQLxParamKind::Xml => Ok(PySQLxValue::Xml(value.extract::<String>().unwrap())),
            PySQLxParamKind::Uuid => {
                let rs_uuid = Self::convert_to_rs_uuid(value);
                Ok(PySQLxValue::Uuid(rs_uuid))
            }
            PySQLxParamKind::Time => Ok(PySQLxValue::Time(value.extract::<NaiveTime>().unwrap())),
            PySQLxParamKind::Date => Ok(PySQLxValue::Date(value.extract::<NaiveDate>().unwrap())),
            PySQLxParamKind::DateTime => {
                Ok(PySQLxValue::DateTime(Self::convert_to_datetime(value)))
            }
            PySQLxParamKind::Float => Ok(PySQLxValue::Float(value.extract::<f64>().unwrap())),
            PySQLxParamKind::Bytes => Ok(PySQLxValue::Bytes(value.extract::<Vec<u8>>().unwrap())),
            PySQLxParamKind::Numeric => {
                Ok(PySQLxValue::Numeric(Self::convert_to_rs_decimal(value)))
            }
            PySQLxParamKind::Null => Ok(PySQLxValue::Null),
            PySQLxParamKind::UnsupportedType(t) => Err(PySQLxInvalidParamError::py_new(
                t,
                "str|int|float|etc".to_string(),
                "Unsupported type, check the documentation".to_string(),
                None,
            )),
        }
    }

    pub fn convert_to_pysqlx_value(
        py: Python,
        values: &HashMap<String, Bound<PyAny>>,
        provider: &str,
    ) -> Result<HashMap<String, PySQLxValue>, PySQLxInvalidParamError> {
        let mut params = HashMap::new();
        for (key, value) in values {
            let kind = PySQLxParamKind::from(py, value, provider);
            match Self::convert_pyobject_to_pysqlx_value(py, kind, value, provider) {
                Ok(v) => {
                    params.insert(key.clone(), v);
                }
                Err(mut e) => {
                    e.set_field(Some(key.clone()));
                    return Err(e);
                }
            }
        }

        Ok(params)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SQLPosition {
    pub idx: i8,
    pub key: String,
    pub new_key: String,
    pub old_key: String,
}
