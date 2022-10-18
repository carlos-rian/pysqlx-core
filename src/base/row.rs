use super::error::DBError;
use super::types::{PysqlxListValue, PysqlxResult};
use bigdecimal::{BigDecimal, FromPrimitive};
use serde::de::Unexpected;
use serde::{ser::Serializer, Deserialize, Deserializer, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(untagged)]
pub enum PysqlxValue {
    String(String),
    Boolean(bool),
    Enum(String),

    Int(i64),
    #[serde(serialize_with = "serialize_bigint")]
    BigInt(i64),

    Uuid(Uuid),
    List(PysqlxListValue),
    Json(String),
    Xml(String),

    // when return a 00:00:00 time, it will be a string
    Time(String),
    // when return a 2020-01-01 date, it will be a string
    Date(String),
    // when return a 2020-01-01T00:00:01 dateTIME, it will be a string
    DateTime(String),

    #[serde(serialize_with = "serialize_null")]
    Null,

    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    Float(BigDecimal),

    #[serde(serialize_with = "serialize_bytes")]
    Bytes(Vec<u8>),
}

pub fn encode_bytes(bytes: &[u8]) -> String {
    base64::encode(bytes)
}

pub fn decode_bytes(s: &str) -> PysqlxResult<Vec<u8>> {
    base64::decode(s)
        .map_err(|_| DBError::ConversionError("base64 encoded bytes", "PysqlxValue::Bytes"))
}

fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    encode_bytes(bytes).serialize(serializer)
}

fn serialize_null<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Option::<u8>::None.serialize(serializer)
}

fn serialize_bigint<S>(int: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    int.to_string().serialize(serializer)
}

fn serialize_decimal<S>(decimal: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    decimal
        .to_string()
        .parse::<f64>()
        .unwrap()
        .serialize(serializer)
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_f64(BigDecimalVisitor)
}

struct BigDecimalVisitor;

impl<'de> serde::de::Visitor<'de> for BigDecimalVisitor {
    type Value = BigDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a BigDecimal type representing a fixed-point number"
        )
    }

    fn visit_i64<E>(self, value: i64) -> Result<BigDecimal, E>
    where
        E: serde::de::Error,
    {
        match BigDecimal::from_i64(value) {
            Some(s) => Ok(s),
            None => Err(E::invalid_value(Unexpected::Signed(value), &self)),
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<BigDecimal, E>
    where
        E: serde::de::Error,
    {
        match BigDecimal::from_u64(value) {
            Some(s) => Ok(s),
            None => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<BigDecimal, E>
    where
        E: serde::de::Error,
    {
        BigDecimal::from_f64(value).ok_or_else(|| E::invalid_value(Unexpected::Float(value), &self))
    }

    fn visit_str<E>(self, value: &str) -> Result<BigDecimal, E>
    where
        E: serde::de::Error,
    {
        BigDecimal::from_str(value).map_err(|_| E::invalid_value(Unexpected::Str(value), &self))
    }
}

impl PysqlxValue {
    pub fn as_enum_value(&self) -> Option<&str> {
        match self {
            PysqlxValue::Enum(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            PysqlxValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, PysqlxValue::Null)
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            PysqlxValue::String(s) => Some(s),
            PysqlxValue::Enum(ev) => Some(ev),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<PysqlxListValue> {
        match self {
            PysqlxValue::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn new_float(float: f64) -> PysqlxValue {
        PysqlxValue::Float(BigDecimal::from_f64(float).unwrap())
    }

    pub fn as_boolean(&self) -> Option<&bool> {
        match self {
            PysqlxValue::Boolean(bool) => Some(bool),
            _ => None,
        }
    }
}

impl fmt::Display for PysqlxValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PysqlxValue::String(x) => x.fmt(f),
            PysqlxValue::Float(x) => x.fmt(f),
            PysqlxValue::Boolean(x) => x.fmt(f),
            PysqlxValue::DateTime(x) => x.fmt(f),
            PysqlxValue::Enum(x) => x.fmt(f),
            PysqlxValue::Int(x) => x.fmt(f),
            PysqlxValue::Null => "null".fmt(f),
            PysqlxValue::Uuid(x) => x.fmt(f),
            PysqlxValue::Json(x) => x.fmt(f),
            PysqlxValue::Xml(x) => x.fmt(f),
            PysqlxValue::BigInt(x) => x.fmt(f),
            PysqlxValue::List(x) => {
                let as_string = format!("{:?}", x);
                as_string.fmt(f)
            }
            PysqlxValue::Time(x) => x.fmt(f),
            PysqlxValue::Date(x) => x.fmt(f),
            PysqlxValue::Bytes(b) => encode_bytes(b).fmt(f),
        }
    }
}

impl From<&str> for PysqlxValue {
    fn from(s: &str) -> Self {
        PysqlxValue::from(s.to_string())
    }
}

impl From<String> for PysqlxValue {
    fn from(s: String) -> Self {
        PysqlxValue::String(s)
    }
}

impl TryFrom<f64> for PysqlxValue {
    type Error = DBError;

    fn try_from(f: f64) -> PysqlxResult<PysqlxValue> {
        BigDecimal::from_f64(f)
            .map(PysqlxValue::Float)
            .ok_or_else(|| DBError::ConversionError("f64", "Decimal"))
    }
}

impl From<bool> for PysqlxValue {
    fn from(b: bool) -> Self {
        PysqlxValue::Boolean(b)
    }
}

impl From<i32> for PysqlxValue {
    fn from(i: i32) -> Self {
        PysqlxValue::Int(i64::from(i))
    }
}

impl From<i64> for PysqlxValue {
    fn from(i: i64) -> Self {
        PysqlxValue::Int(i)
    }
}

impl From<usize> for PysqlxValue {
    fn from(u: usize) -> Self {
        PysqlxValue::Int(u as i64)
    }
}

impl From<Uuid> for PysqlxValue {
    fn from(s: Uuid) -> Self {
        PysqlxValue::Uuid(s)
    }
}

impl From<PysqlxListValue> for PysqlxValue {
    fn from(s: PysqlxListValue) -> Self {
        PysqlxValue::List(s)
    }
}

impl TryFrom<PysqlxValue> for i64 {
    type Error = DBError;

    fn try_from(value: PysqlxValue) -> PysqlxResult<i64> {
        match value {
            PysqlxValue::Int(i) => Ok(i),
            _ => Err(DBError::ConversionError("PysqlxValue", "i64")),
        }
    }
}

impl TryFrom<PysqlxValue> for String {
    type Error = DBError;

    fn try_from(pv: PysqlxValue) -> PysqlxResult<String> {
        match pv {
            PysqlxValue::String(s) => Ok(s),
            _ => Err(DBError::ConversionError("PysqlxValue", "String")),
        }
    }
}

pub fn get_pysqlx_type(row: PysqlxValue) -> String {
    match row {
        PysqlxValue::Boolean(_) => "bool".to_string(),
        PysqlxValue::Bytes(_) => "bytes".to_string(),
        PysqlxValue::Null => "null".to_string(),
        //string default
        PysqlxValue::Enum(_) => "str".to_string(),
        PysqlxValue::String(_) => "str".to_string(),
        // number
        PysqlxValue::BigInt(_) | PysqlxValue::Int(_) => "int".to_string(),
        PysqlxValue::Float(_) => "float".to_string(),
        // datetime, date, time
        PysqlxValue::Time(_) => "time".to_string(),
        PysqlxValue::Date(_) => "date".to_string(),
        PysqlxValue::DateTime(_) => "datetime".to_string(),
        // id
        PysqlxValue::Uuid(_) => "uuid".to_string(),
        // structs and lists
        PysqlxValue::List(_) => "list".to_string(),
        PysqlxValue::Json(_) => "json".to_string(),
        PysqlxValue::Xml(_) => "xml".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pysqlx_value_from() {
        assert_eq!(
            PysqlxValue::from("test"),
            PysqlxValue::String("test".to_string())
        );
        assert_eq!(
            PysqlxValue::from("test".to_string()),
            PysqlxValue::String("test".to_string())
        );
        assert_eq!(
            PysqlxValue::try_from(1.0 as f64).unwrap(),
            PysqlxValue::Float(BigDecimal::from_f64(1.0).unwrap())
        );
        assert_eq!(PysqlxValue::from(true), PysqlxValue::Boolean(true));
        assert_eq!(PysqlxValue::from(1), PysqlxValue::Int(1));
        assert_eq!(PysqlxValue::from(1 as i64), PysqlxValue::Int(1));
        assert_eq!(PysqlxValue::from(1 as usize), PysqlxValue::Int(1));

        let uuid = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        assert_eq!(PysqlxValue::from(uuid), PysqlxValue::Uuid(uuid));
        assert_eq!(
            PysqlxValue::from(PysqlxListValue::new()),
            PysqlxValue::List(PysqlxListValue::new())
        );
    }

    #[test]
    fn test_get_pysqlx_type() {
        assert_eq!(get_pysqlx_type(PysqlxValue::Boolean(true)), "bool");
        assert_eq!(get_pysqlx_type(PysqlxValue::Bytes(vec![])), "bytes");
        assert_eq!(get_pysqlx_type(PysqlxValue::Null), "null");
        assert_eq!(get_pysqlx_type(PysqlxValue::Enum("".to_string())), "str");
        assert_eq!(get_pysqlx_type(PysqlxValue::String("".to_string())), "str");
        assert_eq!(get_pysqlx_type(PysqlxValue::BigInt(0)), "int");
        assert_eq!(get_pysqlx_type(PysqlxValue::Int(0)), "int");
        assert_eq!(
            get_pysqlx_type(PysqlxValue::Float(BigDecimal::from(0))),
            "float"
        );
        assert_eq!(
            get_pysqlx_type(PysqlxValue::Time("00:00:00".to_string())),
            "time"
        );
        assert_eq!(
            get_pysqlx_type(PysqlxValue::Date("0000-00-00".to_string())),
            "date"
        );
        assert_eq!(
            get_pysqlx_type(PysqlxValue::DateTime("0000-00-00 00:00:00".to_string())),
            "datetime"
        );
        assert_eq!(
            get_pysqlx_type(PysqlxValue::Uuid(
                Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap()
            )),
            "uuid"
        );
        assert_eq!(
            get_pysqlx_type(PysqlxValue::List(PysqlxListValue::new())),
            "list"
        );
        assert_eq!(get_pysqlx_type(PysqlxValue::Json("".to_string())), "json");
        assert_eq!(get_pysqlx_type(PysqlxValue::Xml("".to_string())), "xml");
    }
}
