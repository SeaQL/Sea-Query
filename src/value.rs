//! Container for all SQL value types.
use std::fmt::Write;

#[cfg(feature = "with-json")]
use serde_json::Value as Json;
#[cfg(feature = "with-json")]
use std::str::from_utf8;

#[cfg(feature = "with-chrono")]
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

#[cfg(feature = "with-rust_decimal")]
use rust_decimal::Decimal;

#[cfg(feature = "with-bigdecimal")]
use bigdecimal::BigDecimal;

#[cfg(feature = "with-uuid")]
use uuid::Uuid;

/// Value variants
///
/// We want Value to be exactly 1 pointer sized, so anything larger should be boxed.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(Option<bool>),
    TinyInt(Option<i8>),
    SmallInt(Option<i16>),
    Int(Option<i32>),
    BigInt(Option<i64>),
    TinyUnsigned(Option<u8>),
    SmallUnsigned(Option<u16>),
    Unsigned(Option<u32>),
    BigUnsigned(Option<u64>),
    Float(Option<f32>),
    Double(Option<f64>),
    String(Option<Box<String>>),

    #[allow(clippy::box_vec)]
    Bytes(Option<Box<Vec<u8>>>),

    #[cfg(feature = "with-json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-json")))]
    Json(Option<Box<Json>>),

    #[cfg(feature = "with-chrono")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
    Date(Option<Box<NaiveDate>>),

    #[cfg(feature = "with-chrono")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
    Time(Option<Box<NaiveTime>>),

    #[cfg(feature = "with-chrono")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
    DateTime(Option<Box<NaiveDateTime>>),

    #[cfg(feature = "with-chrono")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
    DateTimeWithTimeZone(Option<Box<DateTime<FixedOffset>>>),

    #[cfg(feature = "with-uuid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-uuid")))]
    Uuid(Option<Box<Uuid>>),

    #[cfg(feature = "with-rust_decimal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-rust_decimal")))]
    Decimal(Option<Box<Decimal>>),

    #[cfg(feature = "with-bigdecimal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "with-bigdecimal")))]
    BigDecimal(Option<Box<BigDecimal>>),
}

pub trait ValueType: ValueTypeDefault {
    fn unwrap(v: Value) -> Self;

    fn type_name() -> &'static str;
}

pub trait ValueTypeDefault {
    fn default() -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Values(pub Vec<Value>);

#[derive(Debug, PartialEq)]
pub enum ValueTuple {
    One(Value),
    Two(Value, Value),
    Three(Value, Value, Value),
}

pub trait IntoValueTuple {
    fn into_value_tuple(self) -> ValueTuple;
}

pub trait Nullable {
    fn null() -> Value;
}

impl Value {
    pub fn unwrap<T>(self) -> T
    where
        T: ValueType,
    {
        T::unwrap(self)
    }
}

macro_rules! type_to_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(x))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }

        impl ValueType for $type {
            fn unwrap(v: Value) -> Self {
                match v {
                    Value::$name(Some(x)) => x,
                    _ => panic!("type error"),
                }
            }

            fn type_name() -> &'static str {
                stringify!($type)
            }
        }

        impl ValueTypeDefault for $type
        where
            $type: Default,
        {
            fn default() -> Self {
                Default::default()
            }
        }
    };
}

macro_rules! type_to_box_value {
    ( $type: ty, $name: ident ) => {
        impl From<$type> for Value {
            fn from(x: $type) -> Value {
                Value::$name(Some(Box::new(x)))
            }
        }

        impl Nullable for $type {
            fn null() -> Value {
                Value::$name(None)
            }
        }

        impl ValueType for $type {
            fn unwrap(v: Value) -> Self {
                match v {
                    Value::$name(Some(x)) => *x,
                    _ => panic!("type error"),
                }
            }

            fn type_name() -> &'static str {
                stringify!($type)
            }
        }
    };
}

macro_rules! impl_value_type_default {
    ( $type: ty ) => {
        impl ValueTypeDefault for $type {
            fn default() -> Self {
                Default::default()
            }
        }
    };
}

type_to_value!(bool, Bool);
type_to_value!(i8, TinyInt);
type_to_value!(i16, SmallInt);
type_to_value!(i32, Int);
type_to_value!(i64, BigInt);
type_to_value!(u8, TinyUnsigned);
type_to_value!(u16, SmallUnsigned);
type_to_value!(u32, Unsigned);
type_to_value!(u64, BigUnsigned);
type_to_value!(f32, Float);
type_to_value!(f64, Double);

impl<'a> From<&'a [u8]> for Value {
    fn from(x: &'a [u8]) -> Value {
        Value::Bytes(Some(Box::<Vec<u8>>::new(x.into())))
    }
}

impl<'a> From<&'a str> for Value {
    fn from(x: &'a str) -> Value {
        let string: String = x.into();
        Value::String(Some(Box::new(string)))
    }
}

impl<'a> Nullable for &'a str {
    fn null() -> Value {
        Value::String(None)
    }
}

impl<T> From<Option<T>> for Value
where T: Into<Value> + Nullable {
    fn from(x: Option<T>) -> Value {
        match x {
            Some(v) => v.into(),
            None => T::null(),
        }
    }
}

impl<T> ValueType for Option<T>
where T: ValueType + Nullable {
    fn unwrap(v: Value) -> Self {
        if v == T::null() {
            None
        }
        else {
            Some(v.unwrap())
        }
    }

    fn type_name() -> &'static str {
        //concat!("Option<", T::type_name(), ">")
        T::type_name()
    }
}

impl<T> ValueTypeDefault for Option<T>
where T: ValueType + Nullable {
    fn default() -> Self {
        Default::default()
    }
}

type_to_box_value!(Vec<u8>, Bytes);
impl_value_type_default!(Vec<u8>);
type_to_box_value!(String, String);
impl_value_type_default!(String);

#[cfg(feature = "with-json")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-json")))]
mod with_json {
    use super::*;

    type_to_box_value!(Json, Json);
    impl_value_type_default!(Json);
}

#[cfg(feature = "with-chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-chrono")))]
mod with_chrono {
    use super::*;
    use chrono::{Offset, TimeZone};

    type_to_box_value!(NaiveDate, Date);
    type_to_box_value!(NaiveTime, Time);
    type_to_box_value!(NaiveDateTime, DateTime);

    impl ValueTypeDefault for NaiveDate {
        fn default() -> Self {
            NaiveDate::from_num_days_from_ce(0)
        }
    }

    impl ValueTypeDefault for NaiveTime {
        fn default() -> Self {
            NaiveTime::from_hms(0, 0, 0)
        }
    }

    impl ValueTypeDefault for NaiveDateTime {
        fn default() -> Self {
            NaiveDateTime::from_timestamp(0, 0)
        }
    }

    impl ValueTypeDefault for DateTime<FixedOffset> {
        fn default() -> Self {
            DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), FixedOffset::east(0))
        }
    }

    impl<Tz> From<DateTime<Tz>> for Value
    where
        Tz: TimeZone,
    {
        fn from(x: DateTime<Tz>) -> Value {
            let v = DateTime::<FixedOffset>::from_utc(x.naive_utc(), x.offset().fix());
            Value::DateTimeWithTimeZone(Some(Box::new(v)))
        }
    }

    impl Nullable for DateTime<FixedOffset> {
        fn null() -> Value {
            Value::DateTimeWithTimeZone(None)
        }
    }

    impl ValueType for DateTime<FixedOffset> {
        fn unwrap(v: Value) -> Self {
            match v {
                Value::DateTimeWithTimeZone(Some(x)) => *x,
                _ => panic!("type error"),
            }
        }

        fn type_name() -> &'static str {
            stringify!(DateTime<FixedOffset>)
        }
    }
}

#[cfg(feature = "with-rust_decimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-rust_decimal")))]
mod with_rust_decimal {
    use super::*;

    type_to_box_value!(Decimal, Decimal);
    impl_value_type_default!(Decimal);
}

#[cfg(feature = "with-bigdecimal")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-bigdecimal")))]
mod with_bigdecimal {
    use super::*;

    type_to_box_value!(BigDecimal, BigDecimal);
    impl_value_type_default!(BigDecimal);
}

#[cfg(feature = "with-uuid")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-uuid")))]
mod with_uuid {
    use super::*;

    type_to_box_value!(Uuid, Uuid);
    impl_value_type_default!(Uuid);
}

impl Value {
    pub fn is_json(&self) -> bool {
        #[cfg(feature = "with-json")]
        return matches!(self, Self::Json(_));
        #[cfg(not(feature = "with-json"))]
        return false;
    }
    #[cfg(feature = "with-json")]
    pub fn as_ref_json(&self) -> &Json {
        match self {
            Self::Json(Some(v)) => v.as_ref(),
            _ => panic!("not Value::Json"),
        }
    }
    #[cfg(not(feature = "with-json"))]
    pub fn as_ref_json(&self) -> &bool {
        panic!("not Value::Json")
    }
}

impl Value {
    pub fn is_date(&self) -> bool {
        #[cfg(feature = "with-chrono")]
        return matches!(self, Self::Date(_));
        #[cfg(not(feature = "with-chrono"))]
        return false;
    }
    #[cfg(feature = "with-chrono")]
    pub fn as_ref_date(&self) -> &NaiveDate {
        match self {
            Self::Date(Some(v)) => v.as_ref(),
            _ => panic!("not Value::Date"),
        }
    }
    #[cfg(not(feature = "with-chrono"))]
    pub fn as_ref_date(&self) -> &bool {
        panic!("not Value::Date")
    }
}

impl Value {
    pub fn is_time(&self) -> bool {
        #[cfg(feature = "with-chrono")]
        return matches!(self, Self::Time(_));
        #[cfg(not(feature = "with-chrono"))]
        return false;
    }
    #[cfg(feature = "with-chrono")]
    pub fn as_ref_time(&self) -> &NaiveTime {
        match self {
            Self::Time(Some(v)) => v.as_ref(),
            _ => panic!("not Value::Time"),
        }
    }
    #[cfg(not(feature = "with-chrono"))]
    pub fn as_ref_time(&self) -> &bool {
        panic!("not Value::Time")
    }
}

impl Value {
    pub fn is_date_time(&self) -> bool {
        #[cfg(feature = "with-chrono")]
        return matches!(self, Self::DateTime(_));
        #[cfg(not(feature = "with-chrono"))]
        return false;
    }
    #[cfg(feature = "with-chrono")]
    pub fn as_ref_date_time(&self) -> &NaiveDateTime {
        match self {
            Self::DateTime(Some(v)) => v.as_ref(),
            _ => panic!("not Value::DateTime"),
        }
    }
    #[cfg(not(feature = "with-chrono"))]
    pub fn as_ref_date_time(&self) -> &bool {
        panic!("not Value::DateTime")
    }
}

impl Value {
    pub fn is_date_time_with_time_zone(&self) -> bool {
        #[cfg(feature = "with-chrono")]
        return matches!(self, Self::DateTimeWithTimeZone(_));
        #[cfg(not(feature = "with-chrono"))]
        return false;
    }
    #[cfg(feature = "with-chrono")]
    pub fn as_ref_date_time_with_time_zone(&self) -> &DateTime<FixedOffset> {
        match self {
            Self::DateTimeWithTimeZone(Some(v)) => v.as_ref(),
            _ => panic!("not Value::DateTimeWithTimeZone"),
        }
    }
    #[cfg(not(feature = "with-chrono"))]
    pub fn as_ref_date_time_with_time_zone(&self) -> &bool {
        panic!("not Value::DateTimeWithTimeZone")
    }
}

impl Value {
    pub fn is_decimal(&self) -> bool {
        #[cfg(feature = "with-rust_decimal")]
        return matches!(self, Self::Decimal(_));
        #[cfg(not(feature = "with-rust_decimal"))]
        return false;
    }
    #[cfg(feature = "with-rust_decimal")]
    pub fn as_ref_decimal(&self) -> &Decimal {
        match self {
            Self::Decimal(Some(v)) => v.as_ref(),
            _ => panic!("not Value::Decimal"),
        }
    }
    #[cfg(feature = "with-rust_decimal")]
    pub fn decimal_to_f64(&self) -> f64 {
        use rust_decimal::prelude::ToPrimitive;
        self.as_ref_decimal().to_f64().unwrap()
    }
    #[cfg(not(feature = "with-rust_decimal"))]
    pub fn as_ref_decimal(&self) -> &bool {
        panic!("not Value::Decimal")
    }
    #[cfg(not(feature = "with-rust_decimal"))]
    pub fn decimal_to_f64(&self) -> f64 {
        0.0
    }
}

impl Value {
    pub fn is_big_decimal(&self) -> bool {
        #[cfg(feature = "with-bigdecimal")]
        return matches!(self, Self::BigDecimal(_));
        #[cfg(not(feature = "with-bigdecimal"))]
        return false;
    }
    #[cfg(feature = "with-bigdecimal")]
    pub fn as_ref_big_decimal(&self) -> &BigDecimal {
        match self {
            Self::BigDecimal(Some(v)) => v.as_ref(),
            _ => panic!("not Value::BigDecimal"),
        }
    }
    #[cfg(feature = "with-bigdecimal")]
    pub fn big_decimal_to_f64(&self) -> f64 {
        use bigdecimal::ToPrimitive;
        self.as_ref_big_decimal().to_f64().unwrap()
    }
    #[cfg(not(feature = "with-bigdecimal"))]
    pub fn as_ref_big_decimal(&self) -> &bool {
        panic!("not Value::BigDecimal")
    }
    #[cfg(not(feature = "with-bigdecimal"))]
    pub fn big_decimal_to_f64(&self) -> f64 {
        0.0
    }
}

impl Value {
    pub fn is_uuid(&self) -> bool {
        #[cfg(feature = "with-uuid")]
        return matches!(self, Self::Uuid(_));
        #[cfg(not(feature = "with-uuid"))]
        return false;
    }
    #[cfg(feature = "with-uuid")]
    pub fn as_ref_uuid(&self) -> &Uuid {
        match self {
            Self::Uuid(Some(v)) => v.as_ref(),
            _ => panic!("not Value::Uuid"),
        }
    }
    #[cfg(not(feature = "with-uuid"))]
    pub fn as_ref_uuid(&self) -> &bool {
        panic!("not Value::Uuid")
    }
}

impl IntoIterator for ValueTuple {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ValueTuple::One(v) => vec![v].into_iter(),
            ValueTuple::Two(v, w) => vec![v, w].into_iter(),
            ValueTuple::Three(u, v, w) => vec![u, v, w].into_iter(),
        }
    }
}

impl<V> IntoValueTuple for V
where
    V: Into<Value>,
{
    fn into_value_tuple(self) -> ValueTuple {
        ValueTuple::One(self.into())
    }
}

impl<V, W> IntoValueTuple for (V, W)
where
    V: Into<Value>,
    W: Into<Value>,
{
    fn into_value_tuple(self) -> ValueTuple {
        ValueTuple::Two(self.0.into(), self.1.into())
    }
}

impl<U, V, W> IntoValueTuple for (U, V, W)
where
    U: Into<Value>,
    V: Into<Value>,
    W: Into<Value>,
{
    fn into_value_tuple(self) -> ValueTuple {
        ValueTuple::Three(self.0.into(), self.1.into(), self.2.into())
    }
}

/// Escape a SQL string literal
pub fn escape_string(string: &str) -> String {
    string
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("'", "\\'")
        .replace("\0", "\\0")
        .replace("\x08", "\\b")
        .replace("\x09", "\\t")
        .replace("\x1a", "\\z")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
}

/// Unescape a SQL string literal
pub fn unescape_string(input: &str) -> String {
    let mut escape = false;
    let mut output = String::new();
    for c in input.chars() {
        if !escape && c == '\\' {
            escape = true;
        } else if escape {
            write!(
                output,
                "{}",
                match c {
                    '0' => '\0',
                    'b' => '\x08',
                    't' => '\x09',
                    'z' => '\x1a',
                    'n' => '\n',
                    'r' => '\r',
                    c => c,
                }
            )
            .unwrap();
            escape = false;
        } else {
            write!(output, "{}", c).unwrap();
        }
    }
    output
}

/// Convert value to json value
#[allow(clippy::many_single_char_names)]
#[cfg(feature = "with-json")]
#[cfg_attr(docsrs, doc(cfg(feature = "with-json")))]
pub fn sea_value_to_json_value(value: &Value) -> Json {
    use crate::{CommonSqlQueryBuilder, QueryBuilder};

    match value {
        Value::Bool(None)
        | Value::TinyInt(None)
        | Value::SmallInt(None)
        | Value::Int(None)
        | Value::BigInt(None)
        | Value::TinyUnsigned(None)
        | Value::SmallUnsigned(None)
        | Value::Unsigned(None)
        | Value::BigUnsigned(None)
        | Value::Float(None)
        | Value::Double(None)
        | Value::String(None)
        | Value::Bytes(None)
        | Value::Json(None) => Json::Null,
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(None) => Json::Null,
        #[cfg(feature = "with-bigdecimal")]
        Value::BigDecimal(None) => Json::Null,
        #[cfg(feature = "with-uuid")]
        Value::Uuid(None) => Json::Null,
        Value::Bool(Some(b)) => Json::Bool(*b),
        Value::TinyInt(Some(v)) => (*v).into(),
        Value::SmallInt(Some(v)) => (*v).into(),
        Value::Int(Some(v)) => (*v).into(),
        Value::BigInt(Some(v)) => (*v).into(),
        Value::TinyUnsigned(Some(v)) => (*v).into(),
        Value::SmallUnsigned(Some(v)) => (*v).into(),
        Value::Unsigned(Some(v)) => (*v).into(),
        Value::BigUnsigned(Some(v)) => (*v).into(),
        Value::Float(Some(v)) => (*v).into(),
        Value::Double(Some(v)) => (*v).into(),
        Value::String(Some(s)) => Json::String(s.as_ref().clone()),
        Value::Bytes(Some(s)) => Json::String(from_utf8(s).unwrap().to_string()),
        Value::Json(Some(v)) => v.as_ref().clone(),
        #[cfg(feature = "with-chrono")]
        Value::Date(_) => CommonSqlQueryBuilder.value_to_string(value).into(),
        #[cfg(feature = "with-chrono")]
        Value::Time(_) => CommonSqlQueryBuilder.value_to_string(value).into(),
        #[cfg(feature = "with-chrono")]
        Value::DateTime(_) => CommonSqlQueryBuilder.value_to_string(value).into(),
        #[cfg(feature = "with-chrono")]
        Value::DateTimeWithTimeZone(_) => CommonSqlQueryBuilder.value_to_string(value).into(),
        #[cfg(feature = "with-rust_decimal")]
        Value::Decimal(Some(v)) => {
            use rust_decimal::prelude::ToPrimitive;
            v.as_ref().to_f64().unwrap().into()
        }
        #[cfg(feature = "with-bigdecimal")]
        Value::BigDecimal(Some(v)) => {
            use bigdecimal::ToPrimitive;
            v.as_ref().to_f64().unwrap().into()
        }
        #[cfg(feature = "with-uuid")]
        Value::Uuid(Some(v)) => Json::String(v.to_string()),
    }
}

impl Values {
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_1() {
        let test = r#" "abc" "#;
        assert_eq!(escape_string(test), r#" \"abc\" "#.to_owned());
        assert_eq!(unescape_string(escape_string(test).as_str()), test);
    }

    #[test]
    fn test_escape_2() {
        let test = "a\nb\tc";
        assert_eq!(escape_string(test), "a\\nb\\tc".to_owned());
        assert_eq!(unescape_string(escape_string(test).as_str()), test);
    }

    #[test]
    fn test_escape_3() {
        let test = "a\\b";
        assert_eq!(escape_string(test), "a\\\\b".to_owned());
        assert_eq!(unescape_string(escape_string(test).as_str()), test);
    }

    #[test]
    fn test_escape_4() {
        let test = "a\"b";
        assert_eq!(escape_string(test), "a\\\"b".to_owned());
        assert_eq!(unescape_string(escape_string(test).as_str()), test);
    }

    #[test]
    fn test_value() {
        macro_rules! test_value {
            ( $type: ty, $val: literal ) => {
                let val: $type = $val;
                let v: Value = val.into();
                let out: $type = v.unwrap();
                assert_eq!(out, val);
            };
        }

        test_value!(u8, 255);
        test_value!(u16, 65535);
        test_value!(i8, 127);
        test_value!(i16, 32767);
        test_value!(i32, 1073741824);
        test_value!(i64, 8589934592);
    }

    #[test]
    fn test_option_value() {
        macro_rules! test_some_value {
            ( $type: ty, $val: literal ) => {
                let val: Option<$type> = Some($val);
                let v: Value = val.into();
                let out: $type = v.unwrap();
                assert_eq!(out, val.unwrap());
            };
        }

        macro_rules! test_none {
            ( $type: ty, $name: ident ) => {
                let val: Option<$type> = None;
                let v: Value = val.into();
                assert_eq!(v, Value::$name(None));
            };
        }

        test_some_value!(u8, 255);
        test_some_value!(u16, 65535);
        test_some_value!(i8, 127);
        test_some_value!(i16, 32767);
        test_some_value!(i32, 1073741824);
        test_some_value!(i64, 8589934592);

        test_none!(u8, TinyUnsigned);
        test_none!(u16, SmallUnsigned);
        test_none!(i8, TinyInt);
        test_none!(i16, SmallInt);
        test_none!(i32, Int);
        test_none!(i64, BigInt);
    }

    #[test]
    fn test_box_value() {
        let val: String = "hello".to_owned();
        let v: Value = val.clone().into();
        let out: String = v.unwrap();
        assert_eq!(out, val);
    }

    #[test]
    fn test_value_tuple() {
        assert_eq!(
            1i32.into_value_tuple(),
            ValueTuple::One(Value::Int(Some(1)))
        );
        assert_eq!(
            "b".into_value_tuple(),
            ValueTuple::One(Value::String(Some(Box::new("b".to_owned()))))
        );
        assert_eq!(
            (1i32, "b").into_value_tuple(),
            ValueTuple::Two(
                Value::Int(Some(1)),
                Value::String(Some(Box::new("b".to_owned())))
            )
        );
        assert_eq!(
            (1i32, 2.4f64, "b").into_value_tuple(),
            ValueTuple::Three(
                Value::Int(Some(1)),
                Value::Double(Some(2.4)),
                Value::String(Some(Box::new("b".to_owned())))
            )
        );
    }

    #[test]
    fn test_value_tuple_iter() {
        let mut iter = (1i32).into_value_tuple().into_iter();
        assert_eq!(iter.next().unwrap(), Value::Int(Some(1)));
        assert_eq!(iter.next(), None);

        let mut iter = (1i32, 2.4f64).into_value_tuple().into_iter();
        assert_eq!(iter.next().unwrap(), Value::Int(Some(1)));
        assert_eq!(iter.next().unwrap(), Value::Double(Some(2.4)));
        assert_eq!(iter.next(), None);

        let mut iter = (1i32, 2.4f64, "b").into_value_tuple().into_iter();
        assert_eq!(iter.next().unwrap(), Value::Int(Some(1)));
        assert_eq!(iter.next().unwrap(), Value::Double(Some(2.4)));
        assert_eq!(
            iter.next().unwrap(),
            Value::String(Some(Box::new("b".to_owned())))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[cfg(feature = "with-json")]
    fn test_json_value() {
        let json = serde_json::json! {{
            "a": 25.0,
            "b": "hello",
        }};
        let value: Value = json.clone().into();
        let out: Json = value.unwrap();
        assert_eq!(out, json);
    }

    #[test]
    #[cfg(feature = "with-chrono")]
    fn test_chrono_value() {
        let timestamp = chrono::NaiveDate::from_ymd(2020, 1, 1).and_hms(2, 2, 2);
        let value: Value = timestamp.into();
        let out: NaiveDateTime = value.unwrap();
        assert_eq!(out, timestamp);
    }

    #[test]
    #[cfg(feature = "with-chrono")]
    fn test_chrono_timezone_value() {
        let timestamp = DateTime::parse_from_rfc3339("2020-01-01T02:02:02+08:00").unwrap();
        let value: Value = timestamp.into();
        let out: DateTime<FixedOffset> = value.unwrap();
        assert_eq!(out, timestamp);
    }

    #[test]
    #[cfg(feature = "with-chrono")]
    fn test_chrono_query() {
        use crate::*;

        let string = "2020-01-01T02:02:02+08:00";
        let timestamp = DateTime::parse_from_rfc3339(string).unwrap();

        let query = Query::select().expr(Expr::val(timestamp)).to_owned();

        let formatted = "2020-01-01 02:02:02 +08:00";

        assert_eq!(
            query.to_string(MysqlQueryBuilder),
            format!("SELECT '{}'", formatted)
        );
        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            format!("SELECT '{}'", formatted)
        );
        assert_eq!(
            query.to_string(SqliteQueryBuilder),
            format!("SELECT '{}'", formatted)
        );
    }

    #[test]
    #[cfg(feature = "with-uuid")]
    fn test_uuid_value() {
        let uuid = uuid::Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
        let value: Value = uuid.into();
        let out: uuid::Uuid = value.unwrap();
        assert_eq!(out, uuid);
    }

    #[test]
    #[cfg(feature = "with-rust_decimal")]
    fn test_decimal_value() {
        use std::str::FromStr;

        let num = "2.02";
        let val = Decimal::from_str(num).unwrap();
        let v: Value = val.into();
        let out: Decimal = v.unwrap();
        assert_eq!(out.to_string(), num);
    }
}
