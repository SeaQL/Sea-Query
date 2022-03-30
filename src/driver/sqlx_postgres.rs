#[macro_export]
macro_rules! bind_params_sqlx_postgres {
    ( $query:expr, $params:expr ) => {{
        let mut query = $query;
        for value in $params.iter() {
            macro_rules! bind {
                ( $v: expr, $ty: ty ) => {
                    match $v {
                        Some(v) => query.bind((*v as $ty)),
                        None => query.bind(None::<$ty>),
                    }
                };
            }
            macro_rules! bind_box {
                ( $v: expr, $ty: ty ) => {
                    match $v {
                        Some(v) => query.bind(v.as_ref()),
                        None => query.bind(None::<$ty>),
                    }
                };
            }
            query = match value {
                Value::Bool(v) => bind!(v, bool),
                Value::TinyInt(v) => bind!(v, i8),
                Value::SmallInt(v) => bind!(v, i16),
                Value::Int(v) => bind!(v, i32),
                Value::BigInt(v) => bind!(v, i64),
                Value::TinyUnsigned(v) => bind!(v, u32),
                Value::SmallUnsigned(v) => bind!(v, u32),
                Value::Unsigned(v) => bind!(v, u32),
                Value::BigUnsigned(v) => bind!(v, i64),
                Value::Float(v) => bind!(v, f32),
                Value::Double(v) => bind!(v, f64),
                Value::String(v) => bind_box!(v, String),
                Value::Bytes(v) => bind_box!(v, Vec<u8>),
                #[cfg(feature = "with-json")]
                Value::Json(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDate(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoTime(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTime(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeUtc(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeLocal(v) => query.bind(v),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeWithTimeZone(v) => query.bind(v),
                #[cfg(feature = "with-time")]
                Value::TimeDate(v) => query.bind(v),
                #[cfg(feature = "with-time")]
                Value::TimeTime(v) => query.bind(v),
                #[cfg(feature = "with-time")]
                Value::TimeDateTime(v) => query.bind(v),
                #[cfg(feature = "with-time")]
                Value::TimeDateTimeWithTimeZone(v) => query.bind(v),
                #[cfg(feature = "with-uuid")]
                Value::Uuid(v) => query.bind(v),
                #[cfg(feature = "with-rust_decimal")]
                Value::Decimal(v) => query.bind(v),
                #[cfg(feature = "with-bigdecimal")]
                Value::BigDecimal(v) => query.bind(v),
                #[cfg(feature = "postgres-array")]
                Value::Array(_) => unimplemented!("SQLx array is not supported"),
            };
        }
        query
    }};
}

#[macro_export]
macro_rules! sea_query_driver_postgres {
    () => {
        mod sea_query_driver_postgres {
            use sqlx::{postgres::PgArguments, query::Query, query::QueryAs, Postgres};
            use $crate::{Value, Values};

            type SqlxQuery<'a> = sqlx::query::Query<'a, Postgres, PgArguments>;
            type SqlxQueryAs<'a, T> = sqlx::query::QueryAs<'a, Postgres, T, PgArguments>;

            pub fn bind_query<'a>(query: SqlxQuery<'a>, params: &'a Values) -> SqlxQuery<'a> {
                $crate::bind_params_sqlx_postgres!(query, params.0)
            }

            pub fn bind_query_as<'a, T>(
                query: SqlxQueryAs<'a, T>,
                params: &'a Values,
            ) -> SqlxQueryAs<'a, T> {
                $crate::bind_params_sqlx_postgres!(query, params.0)
            }
        }
    };
}
