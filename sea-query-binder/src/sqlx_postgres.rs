#[cfg(feature = "with-bigdecimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "with-chrono")]
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
#[cfg(feature = "with-ipnetwork")]
use ipnetwork::IpNetwork;
#[cfg(feature = "with-mac_address")]
use mac_address::MacAddress;
#[cfg(feature = "with-rust_decimal")]
use rust_decimal::Decimal;
#[cfg(feature = "with-json")]
use serde_json::Value as Json;
#[cfg(feature = "with-uuid")]
use uuid::Uuid;

use sea_query::{ArrayType, Value};

use crate::SqlxValues;

impl<'q> sqlx::IntoArguments<'q, sqlx::postgres::Postgres> for SqlxValues {
    fn into_arguments(self) -> sqlx::postgres::PgArguments {
        let mut args = sqlx::postgres::PgArguments::default();
        for arg in self.0.into_iter() {
            use sqlx::Arguments;
            match arg {
                Value::Bool(b) => {
                    args.add(b);
                }
                Value::TinyInt(i) => {
                    args.add(i);
                }
                Value::SmallInt(i) => {
                    args.add(i);
                }
                Value::Int(i) => {
                    args.add(i);
                }
                Value::BigInt(i) => {
                    args.add(i);
                }
                Value::TinyUnsigned(i) => {
                    args.add(i.map(|i| i as i16));
                }
                Value::SmallUnsigned(i) => {
                    args.add(i.map(|i| i as i32));
                }
                Value::Unsigned(i) => {
                    args.add(i.map(|i| i as i64));
                }
                Value::BigUnsigned(i) => {
                    args.add(i.map(|i| <i64 as TryFrom<u64>>::try_from(i).unwrap()));
                }
                Value::Float(f) => {
                    args.add(f);
                }
                Value::Double(d) => {
                    args.add(d);
                }
                Value::String(s) => {
                    args.add(s.as_deref());
                }
                Value::Char(c) => {
                    args.add(c.map(|c| c.to_string()));
                }
                Value::Bytes(b) => {
                    args.add(b.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoDate(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeUtc(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeLocal(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeWithTimeZone(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-time")]
                Value::TimeDate(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-time")]
                Value::TimeTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-time")]
                Value::TimeDateTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-time")]
                Value::TimeDateTimeWithTimeZone(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(feature = "with-uuid")]
                Value::Uuid(uuid) => {
                    args.add(uuid.as_deref());
                }
                #[cfg(feature = "with-rust_decimal")]
                Value::Decimal(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(feature = "with-bigdecimal")]
                Value::BigDecimal(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(feature = "with-json")]
                Value::Json(j) => {
                    args.add(j.as_deref());
                }
                #[cfg(feature = "with-ipnetwork")]
                Value::IpNetwork(ip) => {
                    args.add(ip.as_deref());
                }
                #[cfg(feature = "with-mac_address")]
                Value::MacAddress(mac) => {
                    args.add(mac.as_deref());
                }
                Value::CustomSqlx(value) => {
                    value.0.add_postgres(&mut args);
                }
                #[cfg(feature = "postgres-array")]
                Value::Array(ty, v) => match ty {
                    ArrayType::Bool => {
                        let value: Option<Vec<bool>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Bool");
                        args.add(value)
                    }
                    ArrayType::TinyInt => {
                        let value: Option<Vec<i8>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::TinyInt");
                        args.add(value)
                    }
                    ArrayType::SmallInt => {
                        let value: Option<Vec<i16>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::SmallInt");
                        args.add(value)
                    }
                    ArrayType::Int => {
                        let value: Option<Vec<i32>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Int");
                        args.add(value)
                    }
                    ArrayType::BigInt => {
                        let value: Option<Vec<i64>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::BigInt");
                        args.add(value)
                    }
                    ArrayType::TinyUnsigned => {
                        let value: Option<Vec<u8>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::TinyUnsigned");
                        let value: Option<Vec<i16>> =
                            value.map(|vec| vec.into_iter().map(|i| i as i16).collect());
                        args.add(value)
                    }
                    ArrayType::SmallUnsigned => {
                        let value: Option<Vec<u16>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::SmallUnsigned");
                        let value: Option<Vec<i32>> =
                            value.map(|vec| vec.into_iter().map(|i| i as i32).collect());
                        args.add(value)
                    }
                    ArrayType::Unsigned => {
                        let value: Option<Vec<u32>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Unsigned");
                        let value: Option<Vec<i64>> =
                            value.map(|vec| vec.into_iter().map(|i| i as i64).collect());
                        args.add(value)
                    }
                    ArrayType::BigUnsigned => {
                        let value: Option<Vec<u64>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::BigUnsigned");
                        let value: Option<Vec<i64>> = value.map(|vec| {
                            vec.into_iter()
                                .map(|i| <i64 as TryFrom<u64>>::try_from(i).unwrap())
                                .collect()
                        });
                        args.add(value)
                    }
                    ArrayType::Float => {
                        let value: Option<Vec<f32>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Float");
                        args.add(value)
                    }
                    ArrayType::Double => {
                        let value: Option<Vec<f64>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Double");
                        args.add(value)
                    }
                    ArrayType::String => {
                        let value: Option<Vec<String>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::String");
                        args.add(value)
                    }
                    ArrayType::Char => {
                        let value: Option<Vec<char>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Char");
                        let value: Option<Vec<String>> =
                            value.map(|vec| vec.into_iter().map(|c| c.to_string()).collect());
                        args.add(value)
                    }
                    ArrayType::Bytes => {
                        let value: Option<Vec<Vec<u8>>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Bytes");
                        args.add(value)
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoDate => {
                        let value: Option<Vec<NaiveDate>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::ChronoDate");
                        args.add(value);
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoTime => {
                        let value: Option<Vec<NaiveTime>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::ChronoTime");
                        args.add(value);
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoDateTime => {
                        let value: Option<Vec<NaiveDateTime>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::ChronoDateTime");
                        args.add(value);
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoDateTimeUtc => {
                        let value: Option<Vec<DateTime<Utc>>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::ChronoDateTimeUtc");
                        args.add(value);
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoDateTimeLocal => {
                        let value: Option<Vec<DateTime<Local>>> = Value::Array(ty, v).expect(
                            "This Value::Array should consist of Value::ChronoDateTimeLocal",
                        );
                        args.add(value);
                    }
                    #[cfg(feature = "with-chrono")]
                    ArrayType::ChronoDateTimeWithTimeZone => {
                        let value: Option<Vec<DateTime<Local>>> = Value::Array(ty, v).expect(
                            "This Value::Array should consist of Value::ChronoDateTimeWithTimeZone",
                        );
                        args.add(value);
                    }
                    #[cfg(feature = "with-time")]
                    ArrayType::TimeDate => {
                        let value: Option<Vec<time::Date>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::TimeDate");
                        args.add(value);
                    }
                    #[cfg(feature = "with-time")]
                    ArrayType::TimeTime => {
                        let value: Option<Vec<time::Time>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::TimeTime");
                        args.add(value);
                    }
                    #[cfg(feature = "with-time")]
                    ArrayType::TimeDateTime => {
                        let value: Option<Vec<time::PrimitiveDateTime>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::TimeDateTime");
                        args.add(value);
                    }
                    #[cfg(feature = "with-time")]
                    ArrayType::TimeDateTimeWithTimeZone => {
                        let value: Option<Vec<time::OffsetDateTime>> = Value::Array(ty, v).expect(
                            "This Value::Array should consist of Value::TimeDateTimeWithTimeZone",
                        );
                        args.add(value);
                    }
                    #[cfg(feature = "with-uuid")]
                    ArrayType::Uuid => {
                        let value: Option<Vec<Uuid>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Uuid");
                        args.add(value);
                    }
                    #[cfg(feature = "with-rust_decimal")]
                    ArrayType::Decimal => {
                        let value: Option<Vec<Decimal>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Decimal");
                        args.add(value);
                    }
                    #[cfg(feature = "with-bigdecimal")]
                    ArrayType::BigDecimal => {
                        let value: Option<Vec<BigDecimal>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::BigDecimal");
                        args.add(value);
                    }
                    #[cfg(feature = "with-json")]
                    ArrayType::Json => {
                        let value: Option<Vec<Json>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::Json");
                        args.add(value);
                    }
                    #[cfg(feature = "with-ipnetwork")]
                    ArrayType::IpNetwork => {
                        let value: Option<Vec<IpNetwork>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::IpNetwork");
                        args.add(value);
                    }
                    #[cfg(feature = "with-mac_address")]
                    ArrayType::MacAddress => {
                        let value: Option<Vec<MacAddress>> = Value::Array(ty, v)
                            .expect("This Value::Array should consist of Value::MacAddress");
                        args.add(value);
                    }
                },
            }
        }
        args
    }
}
