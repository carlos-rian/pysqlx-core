use super::converter::{Converters, SQLPosition};
use super::errors::PySQLxInvalidParamError;
use super::param::Params;
use super::value::PySQLxValue;
use log::{debug, info};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBytes, PyTuple};
use pyo3::{pyclass, Bound, PyObject, PyResult, Python, ToPyObject};
use quaint::ast::EnumVariant;
use quaint::{Value, ValueType};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::borrow::Cow;
use std::collections::HashMap;

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
            ValueType::Numeric(Some(s)) => PySQLxValue::Numeric(s),
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
            PySQLxValue::Json(s) => Converters::convert_json_value_to_pyobject(py, s).unwrap(),
            PySQLxValue::Xml(s) => s.to_object(py),
            PySQLxValue::Uuid(s) => Converters::convert_to_py_uuid(py, s.to_string()).unwrap(),
            PySQLxValue::Time(s) => s.to_object(py),
            PySQLxValue::Date(s) => s.to_object(py),
            PySQLxValue::DateTime(s) => s.to_object(py),
            PySQLxValue::Float(f) => f.to_object(py),
            PySQLxValue::Bytes(b) => PyBytes::new_bound(py, b).to_object(py),
            PySQLxValue::Numeric(n) => Converters::convert_to_py_decimal(py, n.clone()).unwrap(),
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
            PySQLxValue::Numeric(n) => Value::from(ValueType::Numeric(Some(n))),
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
    fn generate_random_string(length: usize, exist_keys: &Vec<String>, sql: &String) -> String {
        // generate random string with length to replace the parameter in the query
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        // check if the random string is already exist in the keys
        if exist_keys.contains(&rand_string) || sql.contains(&rand_string) {
            // if exist, generate again with length + 1
            return PySQLxStatement::generate_random_string(length + 1, &exist_keys, &sql);
        }
        format!(":{}", rand_string.to_lowercase())
    }

    fn mapped_sql(sql: &str, mut param_keys: Vec<String>) -> (String, Vec<(i8, SQLPosition)>) {
        let mut param_positions: Vec<(i8, SQLPosition)> = Vec::new();
        param_keys.sort_by(|a, b| b.len().cmp(&a.len()));

        let mut positions: Vec<(usize, String, String, String)> = Vec::new();
        let mut exist_keys: Vec<String> = Vec::new();
        let mut new_sql = sql.to_string();

        for key in param_keys {
            let old_key = format!(":{}", key.as_str());
            let temp = new_sql.clone();
            let matches = temp.match_indices(old_key.as_str());
            for (start, mat) in matches {
                let new_key = PySQLxStatement::generate_random_string(7, &exist_keys, &new_sql);

                exist_keys.push(new_key.clone());

                new_sql = new_sql.replacen(&old_key, &new_key, 1);

                let position = (start, key.clone(), old_key.clone(), new_key);
                debug!(
                    "replacing old_key: {} -> new_key: {} -> start: {} -> end: {}",
                    position.2,
                    position.3,
                    position.0,
                    position.0 + mat.len()
                );
                positions.push(position);
            }
        }
        // remap the position based on the new key
        // change the start position based on the new key
        positions = positions
            .iter()
            .map(|v| {
                let new_start = new_sql.find(&v.3).unwrap();
                (new_start, v.1.clone(), v.2.clone(), v.3.clone())
            })
            .collect();

        positions.sort_by(|a, b| a.0.cmp(&b.0));
        for (idx, (_a, key, old, new)) in positions.iter().enumerate() {
            param_positions.push((
                idx as i8,
                SQLPosition {
                    idx: idx as i8,
                    key: key.clone(),
                    old_key: old.clone(),
                    new_key: new.clone(),
                },
            ));
        }
        info!(
            "generated new sql: \noriginal keys -> {}\nrandom keys -> {}",
            sql, new_sql
        );
        (new_sql, param_positions)
    }

    fn provider_param(param: &String, idx: i8) -> String {
        match param.as_str() {
            "postgresql" => format!("${}", idx + 1),
            "sqlserver" => format!("@P{}", idx + 1),
            "sqlite" => format!("?"),
            "mysql" => format!("?"),
            _ => panic!("Unsupported provider"),
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

        let converted_params =
            match Converters::convert_to_pysqlx_value(py, params, provider.as_str()) {
                Ok(v) => v,
                Err(e) => return Err(e),
            };
        let (mut new_sql, param_positions) =
            Self::mapped_sql(sql.as_str(), params.keys().cloned().collect());
        let mut new_params = Vec::new();

        for (idx, sql_pos) in param_positions {
            let value = converted_params.get(&sql_pos.key).unwrap();
            let arg = Self::provider_param(provider, idx);
            new_sql = new_sql.replace(sql_pos.new_key.as_str(), arg.as_str());
            new_params.push(value.clone());
            debug!(
                "replacing new_key: {} -> arg: {} -> old_key: {}{}-> value: {}",
                sql_pos.new_key,
                arg,
                sql_pos.old_key,
                " ".repeat(30 - sql_pos.old_key.len()),
                value.clone().to_value(),
            );
        }
        info!(
            "db.statement = {}, db.params = {}",
            new_sql,
            Params(&new_params.as_slice())
        );
        Ok((new_sql, new_params))
    }

    pub fn get_sql(&self) -> String {
        self.sql.clone()
    }

    pub fn get_params(&self) -> Vec<Value> {
        self.params.iter().map(|v| v.clone().to_value()).collect()
    }

    pub fn prepared_sql(&self) -> (String, Vec<Value>) {
        let params = self.get_params();
        (self.get_sql(), params)
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

    pub fn __repr__(&self) -> String {
        format!(
            "PySQLxStatement(sql={}, params={})",
            self.sql,
            Params(self.params.as_slice())
        )
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn sql(&self) -> String {
        self.sql.clone()
    }

    pub fn params(&self, py: Python) -> Py<PyAny> {
        self.params.as_slice().to_object(py)
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

    #[test] // this test is not working because of the Python::with_gil
    fn test_pyobject_to_pysqlx_value() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let value = py
                .eval_bound("True", None, None)
                .unwrap()
                .extract()
                .unwrap();
            let mut hash_map = HashMap::new();
            hash_map.insert("value".to_string(), value);
            let stmt = PySQLxStatement::py_new(
                py,
                "SELECT * FROM table WHERE column = :value".to_string(),
                "sqlite".to_string(),
                Some(hash_map),
            )
            .unwrap();
            assert_eq!(stmt.get_params(), vec!(Value::boolean(true)));
        })
    }
}
