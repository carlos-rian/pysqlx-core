use super::error::ConversionFailure;
use super::types::{PysqlxListValue, PysqlxResult};
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::prelude::*;
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
    Uuid(Uuid),
    List(PysqlxListValue),
    Json(String),
    Xml(String),

    /// A collections of key-value pairs constituting an object.
    Object(Vec<(String, PysqlxValue)>),

    #[serde(serialize_with = "serialize_null")]
    Null,

    #[serde(serialize_with = "serialize_date")]
    DateTime(DateTime<FixedOffset>),

    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    Float(BigDecimal),

    #[serde(serialize_with = "serialize_bigint")]
    BigInt(i64),

    #[serde(serialize_with = "serialize_bytes")]
    Bytes(Vec<u8>),
}

/// Stringify a date to the following format
/// 1999-05-01T00:00:00.000Z
pub fn stringify_date(date: &DateTime<FixedOffset>) -> String {
    // Warning: Be careful if you plan on changing the code below
    // The findUnique batch optimization expects date inputs to have exactly the same format as date outputs
    // This works today because clients always send date inputs in the same format as the serialized format below
    // Updating this without transforming date inputs to the same format WILL break the findUnique batch optimization
    date.to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub fn encode_bytes(bytes: &[u8]) -> String {
    base64::encode(bytes)
}

pub fn decode_bytes(s: &str) -> PysqlxResult<Vec<u8>> {
    base64::decode(s)
        .map_err(|_| ConversionFailure::new("base64 encoded bytes", "PysqlxValue::Bytes"))
}

fn serialize_date<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    stringify_date(date).serialize(serializer)
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

    pub fn new_datetime(datetime: &str) -> PysqlxValue {
        PysqlxValue::DateTime(DateTime::parse_from_rfc3339(datetime).unwrap())
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
            PysqlxValue::Bytes(b) => encode_bytes(b).fmt(f),
            PysqlxValue::Object(pairs) => {
                let joined = pairs
                    .iter()
                    .map(|(key, value)| format!(r#""{}": {}"#, key, value))
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "{{ {} }}", joined)
            }
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
    type Error = ConversionFailure;

    fn try_from(f: f64) -> PysqlxResult<PysqlxValue> {
        BigDecimal::from_f64(f)
            .map(PysqlxValue::Float)
            .ok_or_else(|| ConversionFailure::new("f64", "Decimal"))
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
    type Error = ConversionFailure;

    fn try_from(value: PysqlxValue) -> PysqlxResult<i64> {
        match value {
            PysqlxValue::Int(i) => Ok(i),
            _ => Err(ConversionFailure::new("PysqlxValue", "i64")),
        }
    }
}

impl TryFrom<PysqlxValue> for String {
    type Error = ConversionFailure;

    fn try_from(pv: PysqlxValue) -> PysqlxResult<String> {
        match pv {
            PysqlxValue::String(s) => Ok(s),
            _ => Err(ConversionFailure::new("PysqlxValue", "String")),
        }
    }
}
