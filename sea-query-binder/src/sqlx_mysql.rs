use crate::SqlxValues;
use sea_query::Value;

impl<'q> sqlx::IntoArguments<'q, sqlx::mysql::MySql> for SqlxValues {
    fn into_arguments(self) -> sqlx::mysql::MySqlArguments {
        let mut args = sqlx::mysql::MySqlArguments::default();
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
                    args.add(i);
                }
                Value::SmallUnsigned(i) => {
                    args.add(i);
                }
                Value::Unsigned(i) => {
                    args.add(i);
                }
                Value::BigUnsigned(i) => {
                    args.add(i);
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
                #[cfg(feature = "postgres-array")]
                Value::BoolArray(_)
                | Value::TinyIntArray(_)
                | Value::SmallIntArray(_)
                | Value::IntArray(_)
                | Value::BigIntArray(_)
                | Value::SmallUnsignedArray(_)
                | Value::UnsignedArray(_)
                | Value::BigUnsignedArray(_)
                | Value::FloatArray(_)
                | Value::DoubleArray(_)
                | Value::StringArray(_)
                | Value::CharArray(_) => panic!("Mysql doesn't support array arguments"),

                #[cfg(feature = "with-chrono")]
                Value::ChronoDate(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoDateArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-chrono")]
                Value::ChronoTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoTimeArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoDateTimeArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeUtc(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoDateTimeUtcArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeLocal(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoDateTimeLocalArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-chrono")]
                Value::ChronoDateTimeWithTimeZone(t) => {
                    args.add(Value::ChronoDateTimeWithTimeZone(t).chrono_as_naive_utc_in_string());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::ChronoDateTimeWithTimeZoneArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-time")]
                Value::TimeDate(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::TimeDateArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-time")]
                Value::TimeTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::TimeTimeArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-time")]
                Value::TimeDateTime(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::TimeDateTimeArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-time")]
                Value::TimeDateTimeWithTimeZone(t) => {
                    args.add(t.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::TimeDateTimeWithTimeZoneArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-uuid")]
                Value::Uuid(uuid) => {
                    args.add(uuid.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::UuidArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-rust_decimal")]
                Value::Decimal(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::DecimalArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-bigdecimal")]
                Value::BigDecimal(d) => {
                    args.add(d.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::BigDecimalArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-json")]
                Value::Json(j) => {
                    args.add(j.as_deref());
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::JsonArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-ipnetwork")]
                Value::IpNetwork(_) => {
                    panic!("Mysql doesn't support IpNetwork arguments");
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::IpNetworkArray(_) => panic!("Mysql doesn't support array"),
                #[cfg(feature = "with-mac_address")]
                Value::MacAddress(_) => {
                    panic!("Mysql doesn't support MacAddress arguments");
                }
                #[cfg(all(feature = "with-chrono", feature = "postgres-array"))]
                Value::MacAddressArray(_) => panic!("Mysql doesn't support array"),
            }
        }
        args
    }
}
