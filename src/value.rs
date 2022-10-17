use super::base::error::DBError;
use super::base::row::PysqlxValue;
use super::base::types::PysqlxResult;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::SecondsFormat;
use quaint::Value;

pub fn to_value(quaint_value: Value<'_>) -> PysqlxResult<PysqlxValue> {
    let val: PysqlxValue = match quaint_value {
        Value::Int32(i) => i
            .map(|i| PysqlxValue::Int(i as i64))
            .unwrap_or(PysqlxValue::Null),
        Value::Int64(i) => i.map(PysqlxValue::Int).unwrap_or(PysqlxValue::Null),
        Value::Float(Some(f)) => match f {
            f if f.is_nan() => return Err(DBError::ConversionError("NaN", "BigDecimal")),
            f if f.is_infinite() => return Err(DBError::ConversionError("Infinity", "BigDecimal")),
            _ => PysqlxValue::Float(BigDecimal::from_f32(f).unwrap().normalized()),
        },

        Value::Float(None) => PysqlxValue::Null,

        Value::Double(Some(f)) => match f {
            f if f.is_nan() => return Err(DBError::ConversionError("NaN", "BigDecimal")),
            f if f.is_infinite() => return Err(DBError::ConversionError("Infinity", "BigDecimal")),
            _ => PysqlxValue::Float(BigDecimal::from_f64(f).unwrap().normalized()),
        },

        Value::Double(None) => PysqlxValue::Null,

        Value::Numeric(d) => d
            // chop the trailing zeroes off so javascript doesn't start rounding things wrong
            .map(|d| PysqlxValue::Float(d.normalized()))
            .unwrap_or(PysqlxValue::Null),

        Value::Text(s) => s
            .map(|s| PysqlxValue::String(s.into_owned()))
            .unwrap_or(PysqlxValue::Null),

        Value::Enum(s) => s
            .map(|s| PysqlxValue::Enum(s.into_owned()))
            .unwrap_or(PysqlxValue::Null),

        Value::Boolean(b) => b.map(PysqlxValue::Boolean).unwrap_or(PysqlxValue::Null),

        Value::Array(Some(v)) => {
            let mut res = Vec::with_capacity(v.len());

            for v in v.into_iter() {
                res.push(to_value(v)?);
            }

            PysqlxValue::List(res)
        }

        Value::Array(None) => PysqlxValue::Null,

        Value::Json(val) => val
            .map(|val| PysqlxValue::Json(val.to_string()))
            .unwrap_or(PysqlxValue::Null),

        Value::Uuid(uuid) => uuid.map(PysqlxValue::Uuid).unwrap_or(PysqlxValue::Null),

        Value::Date(d) => match d {
            Some(v) => PysqlxValue::Date(v.to_string()),
            None => PysqlxValue::Null,
        },
        Value::Time(t) => match t {
            Some(t) => PysqlxValue::Time(t.to_string()),
            None => PysqlxValue::Null,
        },
        //date.to_rfc3339_opts(SecondsFormat::Millis, true)
        Value::DateTime(dt) => dt
            .map(|dt| PysqlxValue::DateTime(dt.to_rfc3339_opts(SecondsFormat::Millis, true)))
            .unwrap_or(PysqlxValue::Null),

        Value::Char(c) => c
            .map(|c| PysqlxValue::String(c.to_string()))
            .unwrap_or(PysqlxValue::Null),

        Value::Bytes(bytes) => bytes
            .map(|b| PysqlxValue::Bytes(b.into_owned()))
            .unwrap_or(PysqlxValue::Null),

        Value::Xml(s) => s
            .map(|s| PysqlxValue::Xml(s.into_owned()))
            .unwrap_or(PysqlxValue::Null),
    };

    Ok(val)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::str::FromStr;

    use crate::{base::row::PysqlxValue, value::to_value};
    use bigdecimal::BigDecimal;
    use bigdecimal::FromPrimitive;
    use chrono::SecondsFormat;
    use quaint::Value;
    use serde_json;
    use uuid::Uuid;

    #[test]
    fn test_to_value_numbers() {
        let val = to_value(Value::Int32(Some(1))).unwrap();
        assert_eq!(val, PysqlxValue::Int(1));

        let val = to_value(Value::Int32(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Int64(Some(1))).unwrap();
        assert_eq!(val, PysqlxValue::Int(1));

        let val = to_value(Value::Int64(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Float(Some(1.0))).unwrap();
        assert_eq!(val, PysqlxValue::Float(BigDecimal::from_f32(1.0).unwrap()));

        let val = to_value(Value::Float(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Double(Some(1.0))).unwrap();
        assert_eq!(val, PysqlxValue::Float(BigDecimal::from_f64(1.0).unwrap()));

        let val = to_value(Value::Double(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Numeric(Some(BigDecimal::from_f64(1.0).unwrap()))).unwrap();
        assert_eq!(val, PysqlxValue::Float(BigDecimal::from_f64(1.0).unwrap()));

        let val = to_value(Value::Numeric(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_text() {
        let text: Cow<'_, str> = Cow::Owned("foo".to_string());

        let val = to_value(Value::Text(Some(text.clone()))).unwrap();
        assert_eq!(val, PysqlxValue::String("foo".to_string()));

        let val = to_value(Value::Text(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Enum(Some(text.clone()))).unwrap();
        assert_eq!(val, PysqlxValue::Enum("foo".to_string()));

        let val = to_value(Value::Enum(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);

        let val = to_value(Value::Char(Some('a'))).unwrap();
        assert_eq!(val, PysqlxValue::String("a".to_string()));

        let val = to_value(Value::Char(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_boolean() {
        let val = to_value(Value::Boolean(Some(true))).unwrap();
        assert_eq!(val, PysqlxValue::Boolean(true));

        let val = to_value(Value::Boolean(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_array() {
        let val = to_value(Value::Array(Some(vec![
            Value::Int32(Some(1)),
            Value::Int32(Some(2)),
        ])))
        .unwrap();
        assert_eq!(
            val,
            PysqlxValue::List(vec![PysqlxValue::Int(1), PysqlxValue::Int(2),])
        );

        let val = to_value(Value::Array(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_json() {
        let value = serde_json::from_str(r#"{"foo":"bar"}"#).unwrap();
        let val = to_value(Value::Json(Some(value))).unwrap();
        assert_eq!(val, PysqlxValue::Json(r#"{"foo":"bar"}"#.to_string()));

        let val = to_value(Value::Json(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_uuid() {
        let id = Uuid::parse_str("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8").unwrap();
        let val = to_value(Value::Uuid(Some(id))).unwrap();
        assert_eq!(val, PysqlxValue::Uuid(id));

        let val = to_value(Value::Uuid(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_date() {
        let date = chrono::NaiveDate::from_ymd(2019, 1, 1);
        let val = to_value(Value::Date(Some(date))).unwrap();
        assert_eq!(val, PysqlxValue::Date("2019-01-01".to_string()));

        let val = to_value(Value::Date(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_time() {
        let time = chrono::NaiveTime::from_hms(12, 0, 0);
        let val = to_value(Value::Time(Some(time))).unwrap();
        assert_eq!(val, PysqlxValue::Time("12:00:00".to_string()));

        let val = to_value(Value::Time(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_datetime() {
        let datetime = chrono::DateTime::from_str("2020-04-12T22:10:57+02:00").unwrap();
        let val = to_value(Value::DateTime(Some(datetime))).unwrap();
        assert_eq!(
            val,
            PysqlxValue::DateTime(datetime.to_rfc3339_opts(SecondsFormat::Millis, true))
        );

        let val = to_value(Value::DateTime(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_bytes() {
        let bytes: Cow<'_, [u8]> = Cow::Owned(vec![1, 2, 3]);
        let val = to_value(Value::Bytes(Some(bytes.clone()))).unwrap();
        assert_eq!(val, PysqlxValue::Bytes(bytes.to_vec()));

        let val = to_value(Value::Bytes(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }

    #[test]
    fn test_to_value_xml() {
        let xml: Cow<'_, str> = Cow::Owned("<foo>bar</foo>".to_string());
        let val = to_value(Value::Xml(Some(xml.clone()))).unwrap();
        assert_eq!(val, PysqlxValue::Xml(xml.to_string()));

        let val = to_value(Value::Xml(None)).unwrap();
        assert_eq!(val, PysqlxValue::Null);
    }
    // ...
}
