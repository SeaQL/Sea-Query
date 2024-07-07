use crate::{expr::ExprTrait, SimpleExpr};

use super::SqliteBinOper;

pub trait SqliteExpr: ExprTrait {
    /// Express an sqlite `GLOB` operator.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{extension::sqlite::SqliteExpr, tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Font::Name)
    ///     .from(Font::Table)
    ///     .and_where(Expr::col(Font::Name).glob("a"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "name" FROM "font" WHERE "name" GLOB 'a'"#
    /// );
    /// ```
    fn glob<T>(self, right: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        self.binary(SqliteBinOper::Glob, right)
    }

    /// Express an sqlite `MATCH` operator.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{extension::sqlite::SqliteExpr, tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Font::Name)
    ///     .from(Font::Table)
    ///     .and_where(Expr::col(Font::Name).matches("a"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "name" FROM "font" WHERE "name" MATCH 'a'"#
    /// );
    /// ```
    fn matches<T>(self, right: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        self.binary(SqliteBinOper::Match, right)
    }

    /// Express an sqlite retrieves JSON field as JSON value (`->`).
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{extension::sqlite::SqliteExpr, tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Font::Variant)
    ///     .from(Font::Table)
    ///     .and_where(Expr::col(Font::Variant).get_json_field("a"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "variant" FROM "font" WHERE "variant" -> 'a'"#
    /// );
    /// ```
    fn get_json_field<T>(self, right: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        self.binary(SqliteBinOper::GetJsonField, right)
    }

    /// Express an sqlite retrieves JSON field and casts it to an appropriate SQL type (`->>`).
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{extension::sqlite::SqliteExpr, tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Font::Variant)
    ///     .from(Font::Table)
    ///     .and_where(Expr::col(Font::Variant).cast_json_field("a"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "variant" FROM "font" WHERE "variant" ->> 'a'"#
    /// );
    /// ```
    fn cast_json_field<T>(self, right: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        self.binary(SqliteBinOper::CastJsonField, right)
    }
}

impl<T> SqliteExpr for T where T: ExprTrait {}
