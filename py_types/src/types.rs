use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyAnyMethods, PyBytes, PyDict, PyModule, PyTuple, PyType, PyTypeMethods};
use pyo3::{intern, pyclass, Bound, PyObject, PyResult, Python, ToPyObject};
use quaint::ast::EnumVariant;
use quaint::{Value, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::borrow::Cow;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::PySQLxInvalidParamError;
// this type is a placeholder for the actual type
type PyValueArray = Vec<PySQLxValue>;

fn get_python_type_name(value: &Bound<'_, PyAny>) -> String {
    value.get_type().qualname().unwrap().to_string()
}

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
                PyTuple::new_bound(py, &list).to_object(py)
            }
            PySQLxValue::Int(i) => i.to_object(py),
            PySQLxValue::Array(l) => {
                let mut list = Vec::new();
                for item in l {
                    list.push(item.to_object(py));
                }
                PyTuple::new_bound(py, &list).to_object(py)
            }
            PySQLxValue::Json(s) => PySQLxStatement::json_value_to_pyobject(py, s).unwrap(),
            PySQLxValue::Xml(s) => s.to_object(py),
            PySQLxValue::Uuid(s) => PySQLxStatement::convert_to_py_uuid(py, s.to_string()).unwrap(),
            PySQLxValue::Time(s) => s.to_object(py),
            PySQLxValue::Date(s) => s.to_object(py),
            PySQLxValue::DateTime(s) => s.to_object(py),
            PySQLxValue::Float(f) => f.to_object(py),
            PySQLxValue::Bytes(b) => PyBytes::new_bound(py, b).to_object(py),
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

#[derive(Debug, Clone)]
#[pyclass]
pub struct PySQLxStatement {
    pub sql: String,
    pub params: Vec<PySQLxValue>,
}

impl PySQLxStatement {
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
                let dict = PyDict::new_bound(py);
                for (key, value) in map {
                    dict.set_item(key, Self::json_value_to_pyobject(py, value)?)?;
                }
                Ok(dict.to_object(py))
            }
            JsonValue::Array(vec) => {
                let list: Vec<PyObject> = vec
                    .into_iter()
                    .map(|v| Self::json_value_to_pyobject(py, v).unwrap())
                    .collect();
                Ok(list.to_object(py))
            }
        }
    }

    fn convert_to_py_uuid(py: Python, r_uuid: String) -> PyResult<PyObject> {
        let uuid_module = PyModule::import_bound(py, "uuid")?;
        let py_uuid = uuid_module
            .getattr("UUID")
            .unwrap()
            .call1((r_uuid,))
            .unwrap();
        Ok(py_uuid.to_object(py))
    }

    fn convert_to_rs_uuid(value: &Bound<'_, PyAny>) -> Uuid {
        let py_uuid = value.to_string();
        Uuid::parse_str(&py_uuid).unwrap()
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

        log::debug!(
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
            ))
        }
    }

    fn convert_pyobject_to_pysqlx_value(
        py: Python,
        kind: PySQLxParamKind,
        value: &Bound<'_, PyAny>,
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
                let list = value.extract::<Vec<Bound<PyAny>>>().unwrap();
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
                        match Self::convert_pyobject_to_pysqlx_value(py, kind.clone(), &item) {
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
            PySQLxParamKind::Null => Ok(PySQLxValue::Null),
            PySQLxParamKind::UnsupportedType(t) => Err(PySQLxInvalidParamError::py_new(
                t,
                "str|int|float|etc".to_string(),
                "Unsupported type, check the documentation".to_string(),
            )),
        }
    }

    fn convert_to_pysqlx_value(
        py: Python,
        values: &HashMap<String, Bound<PyAny>>,
    ) -> Result<HashMap<String, PySQLxValue>, PySQLxInvalidParamError> {
        let mut params = HashMap::new();
        for (key, value) in values {
            let kind = PySQLxParamKind::from(py, value);
            match Self::convert_pyobject_to_pysqlx_value(py, kind, value) {
                Ok(v) => {
                    params.insert(key.clone(), v);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(params)
    }

    fn find_sql_param_position_based_on_key(
        sql: &String,
        param_keys: Vec<String>,
    ) -> Vec<(i8, String)> {
        // Find the position of the parameters in the SQL query
        // i8 is the sequence of the parameter in the query
        // for example, if the query is "SELECT * FROM table WHERE id = :x AND name = :y"
        // the position of (0, "x") and (1, "y")
        // if the param repeated in the query, the position will be different
        // for example, if the query is "SELECT * FROM table WHERE id = :x AND name = :x"
        // the position of (0, "x") and (1, "x")
        // if the param is repeated and the query is "SELECT * FROM table WHERE id = :x AND name = :y AND id = :x"
        // the position of (0, "x"), (1, "y"), and (2, "x")
        let mut param_positions: Vec<(i8, String)> = Vec::new();
        // start, end and key
        let mut position: Vec<(usize, usize, String)> = Vec::new();
        for key in param_keys {
            let k = format!(":{}", key.as_str());
            let matches = sql.match_indices(k.as_str());
            for (start, _) in matches {
                let end = start + key.len();
                position.push((start, end, key.clone()));
            }
        }
        position.sort_by(|a: &(usize, usize, String), b: &(usize, usize, String)| a.0.cmp(&b.0));
        for (idx, value) in position.iter().enumerate() {
            param_positions.push((idx as i8, value.2.clone()));
        }
        param_positions
    }

    fn provider_param(param: &String, idx: i8) -> String {
        match param.as_str() {
            "postgresql" => format!("${}", idx + 1),
            "sqlserver" => format!("@P{}", idx + 1),
            _ => "?".to_string(), // "sqlite" | "mysql"
        }
    }

    pub fn prepare_sql_typed<'a>(
        py: Python<'a>,
        sql: &String,
        params: &HashMap<String, Bound<PyAny>>,
        provider: &'a String,
    ) -> Result<(String, Vec<PySQLxValue>), PySQLxInvalidParamError> {
        if params.is_empty() {
            return Ok((sql.to_string(), Vec::new()));
        }
        let mut new_sql = sql.clone();
        let mut new_params = Vec::new();

        let converted_params = match Self::convert_to_pysqlx_value(py, params) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let param_positions =
            Self::find_sql_param_position_based_on_key(sql, params.keys().cloned().collect());

        for (idx, key) in param_positions {
            let value = converted_params.get(&key).unwrap();
            new_sql = new_sql.replace(
                &format!(":{}", key),
                Self::provider_param(provider, idx).as_str(),
            );
            new_params.push(value.clone());
        }
        Ok((new_sql, new_params))
    }

    pub fn get_sql(&self) -> String {
        self.sql.clone()
    }

    pub fn get_params(&self) -> Vec<Value> {
        self.params.iter().map(|v| v.clone().to_value()).collect()
    }
}

#[pymethods]
impl PySQLxStatement {
    #[new]
    #[pyo3(signature = (sql, provider, params = None))]
    fn py_new(
        py: Python,
        sql: String,
        provider: String,
        params: Option<HashMap<String, Bound<PyAny>>>,
    ) -> PyResult<Self> {
        let (new_sql, new_params) =
            match Self::prepare_sql_typed(py, &sql, &params.unwrap_or(HashMap::new()), &provider) {
                Ok((sql, p)) => (sql, p),
                Err(e) => return Err(e.to_pyerr()),
            };

        Ok(PySQLxStatement {
            sql: new_sql,
            params: new_params,
        })
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

    fn validate_tuple_is_same_type(tuple: &Vec<Bound<'_, PyAny>>) -> (bool, String) {
        let kind = get_python_type_name(&tuple[0]);
        for (idx, item) in tuple.iter().enumerate().skip(1) {
            let item_kind = get_python_type_name(&item);
            if kind != item_kind {
                //return (false, format!("the tuple must have the same type, the first item is a {} and the current item is a {}", kind, item_kind));
                return (false, format!("The tuple must have the same type, the first item is a {} and the current item position {} is a {}", kind, idx, item_kind));
            }
        }
        (true, String::new())
    }

    fn from(py: Python, value: &Bound<'_, PyAny>) -> Self {
        // kind string is python class Type name
        match get_python_type_name(value).as_str() {
            "bool" => PySQLxParamKind::Boolean,
            "str" => PySQLxParamKind::String,
            "int" => PySQLxParamKind::Int,
            "tuple" => {
                // check if the tuple is empty
                let tuple = value.extract::<Vec<Bound<PyAny>>>().unwrap();
                if tuple.is_empty() {
                    return PySQLxParamKind::Array;
                }

                // check if the tuple has the same type
                let (is_same_type, msg) = Self::validate_tuple_is_same_type(&tuple);
                if !is_same_type {
                    return PySQLxParamKind::UnsupportedType(msg);
                }

                if Self::is_enum_instance(py, &tuple[0]) {
                    return PySQLxParamKind::EnumArray;
                }

                PySQLxParamKind::Array
            }
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
            "enum" => PySQLxParamKind::Enum,
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
