
use sqlx::any::AnyRow;
use sqlx::{Decode};
use sqlx;
use serde_json::{ Map, Value };
use sqlx::{ Row, Column };
use sqlx::ValueRef;
use sqlx::TypeInfo;
use super::utils::add_value_to_map;
//use uuid::Uuid;


pub fn row_to_json<'r>(row: AnyRow) -> Value {
    use Value::{Null, Object};

    let columns = row.columns();
    let mut map = Map::new();
    for col in columns {
        let key = col.name().to_string();
        let value: Value = match row.try_get_raw(col.ordinal()) {
            Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                "REAL" | "FLOAT" | "NUMERIC" | "FLOAT4" | "FLOAT8" | "DOUBLE" => {
                    <f64 as Decode<sqlx::any::Any>>::decode(raw_value)
                        .unwrap_or(f64::NAN)
                        .into()
                }
                "INT8" | "BIGINT" => <i64 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "INT" | "INTEGER" | "INT4" => <i32 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "INT2" | "SMALLINT" => <i16 as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                "BOOL" | "BOOLEAN" => <bool as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into(),
                //"JSON" | "JSON[]" | "JSONB" | "JSONB[]" if !raw_value.type_info().to_string().contains("Mssql") => {
                //    <&[u8] as Decode<sqlx::any::Any>>::decode(raw_value)
                //        .and_then(|rv| {
                //            serde_json::from_slice::<Value>(rv).map_err(|e| {
                //                Box::new(e) as Box<dyn std::error::Error + Sync + Send>
                //            })
                //        })
                //        .unwrap_or_default()
                //}
                // Deserialize as a string by default
                "UUID" => {
                    let x = <String as Decode<sqlx::any::Any>>::decode(raw_value).unwrap_or_default().into();
                    println!("v={}", x);
                    x
                }
                _ => {
                    println!("{}", raw_value.type_info());
                    <String as Decode<sqlx::any::Any>>::decode(raw_value)
                    .unwrap_or_default()
                    .into()
                },
            },
            Ok(_null) => {
                Null
            },
            Err(e) => {
                println!("Unable to extract value from row: {:?}", e);
                Null
            }
        };
        map = add_value_to_map(map, (key, value));
    }
    Object(map)
}