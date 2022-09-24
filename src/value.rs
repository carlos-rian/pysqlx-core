use super::base::error::ConversionFailure;
use super::base::row::PysqlxValue;
use super::base::types::PysqlxResult;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, NaiveDate, Utc};
use quaint::Value;

pub fn to_value(quaint_value: Value<'_>) -> PysqlxResult<PysqlxValue> {
    let val: PysqlxValue = match quaint_value {
        Value::Int32(i) => i
            .map(|i| PysqlxValue::Int(i as i64))
            .unwrap_or(PysqlxValue::Null),
        Value::Int64(i) => i.map(PysqlxValue::Int).unwrap_or(PysqlxValue::Null),
        Value::Float(Some(f)) => match f {
            f if f.is_nan() => return Err(ConversionFailure::new("NaN", "BigDecimal").into()),
            f if f.is_infinite() => {
                return Err(ConversionFailure::new("Infinity", "BigDecimal").into())
            }
            _ => PysqlxValue::Float(BigDecimal::from_f32(f).unwrap().normalized()),
        },

        Value::Float(None) => PysqlxValue::Null,

        Value::Double(Some(f)) => match f {
            f if f.is_nan() => return Err(ConversionFailure::new("NaN", "BigDecimal").into()),
            f if f.is_infinite() => {
                return Err(ConversionFailure::new("Infinity", "BigDecimal").into())
            }
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

        Value::Date(d) => d
            .map(|d| {
                let dt = DateTime::<Utc>::from_utc(d.and_hms(0, 0, 0), Utc);
                PysqlxValue::DateTime(dt.into())
            })
            .unwrap_or(PysqlxValue::Null),

        Value::Time(t) => t
            .map(|t| {
                let d = NaiveDate::from_ymd(1970, 1, 1);
                let dt = DateTime::<Utc>::from_utc(d.and_time(t), Utc);
                PysqlxValue::DateTime(dt.into())
            })
            .unwrap_or(PysqlxValue::Null),

        Value::DateTime(dt) => dt
            .map(|dt| PysqlxValue::DateTime(dt.into()))
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
