use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::prelude::*;
use serde::de::Unexpected;
use serde::{ser::Serializer, Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};
//use std::{convert::TryFrom, fmt, str::FromStr};
use uuid::Uuid;

pub type PysqlxListValue = Vec<NewPysqlxValue>;

#[derive(Debug, PartialEq, Clone, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(untagged)]
pub enum NewPysqlxValue {
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

    /// A collections of key-value pairs constituting an object.
    Object(Vec<(String, NewPysqlxValue)>),

    #[serde(serialize_with = "serialize_null")]
    Null,

    #[serde(serialize_with = "serialize_date")]
    DateTime(DateTime<FixedOffset>),

    #[serde(
        serialize_with = "serialize_decimal",
        deserialize_with = "deserialize_decimal"
    )]
    Float(BigDecimal),

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

fn serialize_null<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Option::<u8>::None.serialize(serializer)
}

fn serialize_date<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    stringify_date(date).serialize(serializer)
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

fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    encode_bytes(bytes).serialize(serializer)
}

fn serialize_bigint<S>(int: &i64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    int.to_string().serialize(serializer)
}

pub fn encode_bytes(bytes: &[u8]) -> String {
    base64::encode(bytes)
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
