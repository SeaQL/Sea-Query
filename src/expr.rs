//! Building blocks of SQL statements.
//!
//! [`Expr`] representing the primitive building block in the expressions.
//!
//! [`SimpleExpr`] is the expression common among select fields, where clauses and many other places.

use crate::{func::*, query::*, types::*, value::*};

/// Helper to build a [`SimpleExpr`].
#[derive(Debug, Clone, Default)]
pub struct Expr {
    pub(crate) left: Option<SimpleExpr>,
    pub(crate) right: Option<SimpleExpr>,
    pub(crate) uopr: Option<UnOper>,
    pub(crate) bopr: Option<BinOper>,
    pub(crate) func: Option<Function>,
    pub(crate) args: Vec<SimpleExpr>,
}

/// Represents a Simple Expression in SQL.
///
/// [`SimpleExpr`] is a node in the expression tree and can represent identifiers, function calls,
/// various operators and sub-queries.
#[derive(Debug, Clone)]
pub enum SimpleExpr {
    Column(ColumnRef),
    Unary(UnOper, Box<SimpleExpr>),
    FunctionCall(Function, Vec<SimpleExpr>),
    Binary(Box<SimpleExpr>, BinOper, Box<SimpleExpr>),
    SubQuery(Box<SelectStatement>),
    Value(Value),
    Values(Vec<Value>),
    Custom(String),
    CustomWithValues(String, Vec<Value>),
    Keyword(Keyword),
}

impl Expr {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn new_with_left(left: SimpleExpr) -> Self {
        Self {
            left: Some(left),
            right: None,
            uopr: None,
            bopr: None,
            func: None,
            args: Vec::new(),
        }
    }

    /// Express the target column without table prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).eq(1))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` = 1"#
    /// );
    /// ```
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).eq(1))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` = 1"#
    /// );
    /// ```
    pub fn col<T>(n: T) -> Self
    where
        T: IntoColumnRef,
    {
        Self::new_with_left(SimpleExpr::Column(n.into_column_ref()))
    }

    /// Express the target column with table prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).eq(1))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` = 1"#
    /// );
    /// ```
    pub fn tbl<T, C>(t: T, c: C) -> Self
    where
        T: IntoIden,
        C: IntoIden,
    {
        Self::col((t.into_iden(), c.into_iden()))
    }

    /// Express a [`Value`], returning a [`Expr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).into())
    ///     .and_where(Expr::val(2.5).into())
    ///     .and_where(Expr::val("3").into())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// ```
    pub fn val<V>(v: V) -> Self
    where
        V: Into<Value>,
    {
        Self::new_with_left(SimpleExpr::Value(v.into()))
    }

    /// Wrap a [`SimpleExpr`] and perform some operation on it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::expr(Expr::col(Char::SizeW).if_null(0)).gt(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE IFNULL(`size_w`, 0) > 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE COALESCE("size_w", 0) > 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE IFNULL(`size_w`, 0) > 2"#
    /// );
    /// ```
    pub fn expr(expr: SimpleExpr) -> Self {
        Self::new_with_left(expr)
    }

    /// Express a [`Value`], returning a [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::value(1).into())
    ///     .and_where(Expr::value(2.5).into())
    ///     .and_where(Expr::value("3").into())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// ```
    pub fn value<V>(v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        SimpleExpr::Value(v.into())
    }

    /// Express any custom expression in [`&str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::cust("1 = 1").into())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 = 1"#
    /// );
    /// ```
    pub fn cust(s: &str) -> SimpleExpr {
        SimpleExpr::Custom(s.to_owned())
    }

    /// Express any custom expression with [`Value`]. Use this if your expression needs variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::Id).eq(1))
    ///     .and_where(Expr::cust_with_values("6 = ? * ?", vec![2, 3]).into())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `id` = 1 AND 6 = 2 * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "id" = 1 AND 6 = 2 * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `id` = 1 AND 6 = 2 * 3"#
    /// );
    /// ```
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("6 = ? * ?", vec![2, 3]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT 6 = 2 * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT 6 = 2 * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT 6 = 2 * 3"#
    /// );
    /// ```
    /// Postgres only: use `??` to escape `?`
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("? ?? ?", vec!["a", "b"]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT 'a' ? 'b'"#
    /// );
    /// ```
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("data @?? (?::JSONPATH)", vec!["hello"]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT data @? ('hello'::JSONPATH)"#
    /// );
    /// ```
    pub fn cust_with_values<V, I>(s: &str, v: I) -> SimpleExpr
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        SimpleExpr::CustomWithValues(s.to_owned(), v.into_iter().map(|v| v.into()).collect())
    }

    /// Express an equal (`=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val("What!").eq("Nothing"))
    ///     .and_where(Expr::col(Char::Id).eq(1))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'What!' = 'Nothing' AND `id` = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'What!' = 'Nothing' AND "id" = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'What!' = 'Nothing' AND `id` = 1"#
    /// );
    /// ```
    pub fn eq<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::Equal, SimpleExpr::Value(v.into()))
    }

    /// Express a not equal (`<>`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val("Morning").ne("Good"))
    ///     .and_where(Expr::col(Char::Id).ne(1))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'Morning' <> 'Good' AND `id` <> 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'Morning' <> 'Good' AND "id" <> 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'Morning' <> 'Good' AND `id` <> 1"#
    /// );
    /// ```
    pub fn ne<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::NotEqual, SimpleExpr::Value(v.into()))
    }

    /// Express a equal expression between two table columns,
    /// you will mainly use this to relate identical value between two table columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`font_id` = `font`.`id`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."font_id" = "font"."id""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`font_id` = `font`.`id`"#
    /// );
    /// ```
    pub fn equals<T, C>(self, t: T, c: C) -> SimpleExpr
    where
        T: IntoIden,
        C: IntoIden,
    {
        self.bin_oper(
            BinOper::Equal,
            SimpleExpr::Column((t.into_iden(), c.into_iden()).into_column_ref()),
        )
    }

    /// Express a greater than (`>`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).gt(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` > 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" > 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` > 2"#
    /// );
    /// ```
    pub fn gt<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::GreaterThan, SimpleExpr::Value(v.into()))
    }

    /// Express a greater than or equal (`>=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).gte(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` >= 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" >= 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` >= 2"#
    /// );
    /// ```
    pub fn gte<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::GreaterThanOrEqual, SimpleExpr::Value(v.into()))
    }

    /// Express a less than (`<`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).lt(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` < 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" < 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` < 2"#
    /// );
    /// ```
    pub fn lt<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::SmallerThan, SimpleExpr::Value(v.into()))
    }

    /// Express a less than or equal (`<=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).lte(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` <= 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" <= 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` <= 2"#
    /// );
    /// ```
    pub fn lte<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::SmallerThanOrEqual, SimpleExpr::Value(v.into()))
    }

    /// Express an arithmetic addition operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).add(1).equals(Expr::value(2)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 + 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 + 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 + 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::Add, SimpleExpr::Value(v.into()))
    }

    /// Express an arithmetic subtraction operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).sub(1).equals(Expr::value(2)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 - 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 - 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 - 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn sub<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::Sub, SimpleExpr::Value(v.into()))
    }

    /// Express an arithmetic multiplication operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).mul(1).equals(Expr::value(2)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 * 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 * 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 * 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn mul<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::Mul, SimpleExpr::Value(v.into()))
    }

    /// Express an arithmetic division operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).div(1).equals(Expr::value(2)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 / 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 / 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 / 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn div<V>(self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(BinOper::Div, SimpleExpr::Value(v.into()))
    }

    /// Express a `BETWEEN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).between(1, 10))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` BETWEEN 1 AND 10"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" BETWEEN 1 AND 10"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` BETWEEN 1 AND 10"#
    /// );
    /// ```
    pub fn between<V>(self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.between_or_not_between(BinOper::Between, a, b)
    }

    /// Express a `NOT BETWEEN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).not_between(1, 10))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` NOT BETWEEN 1 AND 10"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" NOT BETWEEN 1 AND 10"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` NOT BETWEEN 1 AND 10"#
    /// );
    /// ```
    pub fn not_between<V>(self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.between_or_not_between(BinOper::NotBetween, a, b)
    }

    fn between_or_not_between<V>(self, op: BinOper, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        self.bin_oper(
            op,
            SimpleExpr::Binary(
                Box::new(SimpleExpr::Value(a.into())),
                BinOper::And,
                Box::new(SimpleExpr::Value(b.into())),
            ),
        )
    }

    /// Express a `LIKE` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::Character).like("Ours'%"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`character` LIKE 'Ours\'%'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."character" LIKE E'Ours\'%'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`character` LIKE 'Ours\'%'"#
    /// );
    /// ```
    pub fn like(self, v: &str) -> SimpleExpr {
        self.bin_oper(
            BinOper::Like,
            SimpleExpr::Value(Value::String(Box::new(v.to_owned()))),
        )
    }

    pub fn not_like(self, v: &str) -> SimpleExpr {
        self.bin_oper(
            BinOper::NotLike,
            SimpleExpr::Value(Value::String(Box::new(v.to_owned()))),
        )
    }

    /// Express a `IS NULL` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_null())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` IS NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" IS NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` IS NULL"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_null(self) -> SimpleExpr {
        self.bin_oper(BinOper::Is, SimpleExpr::Keyword(Keyword::Null))
    }

    /// Express a `IS NOT NULL` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_not_null())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` IS NOT NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" IS NOT NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`size_w` IS NOT NULL"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_null(self) -> SimpleExpr {
        self.bin_oper(BinOper::IsNot, SimpleExpr::Keyword(Keyword::Null))
    }

    /// Negates an expression with `NOT`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::expr(Expr::tbl(Char::Table, Char::SizeW).is_null()).not())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE NOT `character`.`size_w` IS NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE NOT "character"."size_w" IS NULL"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE NOT `character`.`size_w` IS NULL"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> SimpleExpr {
        self.un_oper(UnOper::Not)
    }

    /// Express a `MAX` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::tbl(Char::Table, Char::SizeW).max())
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MAX(`character`.`size_w`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MAX("character"."size_w") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MAX(`character`.`size_w`) FROM `character`"#
    /// );
    /// ```
    pub fn max(mut self) -> SimpleExpr {
        let left = self.left.take();
        Self::func_with_args(Function::Max, vec![left.unwrap()])
    }

    /// Express a `MIN` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::tbl(Char::Table, Char::SizeW).min())
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MIN(`character`.`size_w`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MIN("character"."size_w") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MIN(`character`.`size_w`) FROM `character`"#
    /// );
    /// ```
    pub fn min(mut self) -> SimpleExpr {
        let left = self.left.take();
        Self::func_with_args(Function::Min, vec![left.unwrap()])
    }

    /// Express a `SUM` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::tbl(Char::Table, Char::SizeW).sum())
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT SUM(`character`.`size_w`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT SUM("character"."size_w") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT SUM(`character`.`size_w`) FROM `character`"#
    /// );
    /// ```
    pub fn sum(mut self) -> SimpleExpr {
        let left = self.left.take();
        Self::func_with_args(Function::Sum, vec![left.unwrap()])
    }

    /// Express a `COUNT` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::tbl(Char::Table, Char::SizeW).count())
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT COUNT(`character`.`size_w`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT COUNT("character"."size_w") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT COUNT(`character`.`size_w`) FROM `character`"#
    /// );
    /// ```
    pub fn count(mut self) -> SimpleExpr {
        let left = self.left.take();
        Self::func_with_args(Function::Count, vec![left.unwrap()])
    }

    /// Express a `IF NULL` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::tbl(Char::Table, Char::SizeW).if_null(0))
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT IFNULL(`character`.`size_w`, 0) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT COALESCE("character"."size_w", 0) FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT IFNULL(`character`.`size_w`, 0) FROM `character`"#
    /// );
    /// ```
    pub fn if_null<V>(mut self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        let left = self.left.take();
        Self::func_with_args(
            Function::IfNull,
            vec![left.unwrap(), SimpleExpr::Value(v.into())],
        )
    }

    /// Express a `IN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_in(vec![1, 2, 3]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `character`.`size_w` IN (1, 2, 3)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE "character"."size_w" IN (1, 2, 3)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `character`.`size_w` IN (1, 2, 3)"#
    /// );
    /// ```
    /// Empty value list
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_in(Vec::<u8>::new()))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_in<V, I>(mut self, v: I) -> SimpleExpr
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        self.bopr = Some(BinOper::In);
        self.right = Some(SimpleExpr::Values(
            v.into_iter().map(|v| v.into()).collect(),
        ));
        self.into()
    }

    /// Express a `NOT IN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_not_in(vec![1, 2, 3]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `character`.`size_w` NOT IN (1, 2, 3)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE "character"."size_w" NOT IN (1, 2, 3)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `character`.`size_w` NOT IN (1, 2, 3)"#
    /// );
    /// ```
    /// Empty value list
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::SizeW).is_not_in(Vec::<u8>::new()))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE 1 = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE 1 = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE 1 = 1"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_in<V, I>(mut self, v: I) -> SimpleExpr
    where
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        self.bopr = Some(BinOper::NotIn);
        self.right = Some(SimpleExpr::Values(
            v.into_iter().map(|v| v.into()).collect(),
        ));
        self.into()
    }

    /// Express a `IN` sub-query expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).in_subquery(
    ///         Query::select()
    ///             .expr(Expr::cust("3 + 2 * 2"))
    ///             .take()
    ///     ))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` IN (SELECT 3 + 2 * 2)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" IN (SELECT 3 + 2 * 2)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` IN (SELECT 3 + 2 * 2)"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn in_subquery(mut self, sel: SelectStatement) -> SimpleExpr {
        self.bopr = Some(BinOper::In);
        self.right = Some(SimpleExpr::SubQuery(Box::new(sel)));
        self.into()
    }

    /// Express a `NOT IN` sub-query expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).not_in_subquery(
    ///         Query::select()
    ///             .expr(Expr::cust("3 + 2 * 2"))
    ///             .take()
    ///     ))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` NOT IN (SELECT 3 + 2 * 2)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" NOT IN (SELECT 3 + 2 * 2)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` NOT IN (SELECT 3 + 2 * 2)"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn not_in_subquery(mut self, sel: SelectStatement) -> SimpleExpr {
        self.bopr = Some(BinOper::NotIn);
        self.right = Some(SimpleExpr::SubQuery(Box::new(sel)));
        self.into()
    }

    pub(crate) fn func(func: Function) -> Self {
        let mut expr = Expr::new();
        expr.func = Some(func);
        expr
    }

    pub fn arg<T>(mut self, arg: T) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
    {
        self.args = vec![arg.into()];
        self.into()
    }

    pub fn args<T, I>(mut self, args: I) -> SimpleExpr
    where
        T: Into<SimpleExpr>,
        I: IntoIterator<Item = T>,
    {
        self.args = args.into_iter().map(|v| v.into()).collect();
        self.into()
    }

    fn func_with_args(func: Function, args: Vec<SimpleExpr>) -> SimpleExpr {
        let mut expr = Expr::new();
        expr.func = Some(func);
        expr.args = args;
        expr.into()
    }

    fn un_oper(mut self, o: UnOper) -> SimpleExpr {
        self.uopr = Some(o);
        self.into()
    }

    fn bin_oper(mut self, o: BinOper, e: SimpleExpr) -> SimpleExpr {
        self.bopr = Some(o);
        self.right = Some(e);
        self.into()
    }

    /// `Into::<SimpleExpr>::into()` when type inference is impossible
    pub fn into_simple_expr(self) -> SimpleExpr {
        self.into()
    }
}

impl Into<SimpleExpr> for Expr {
    /// Convert into SimpleExpr. Will panic if this Expr is missing an operand
    fn into(self) -> SimpleExpr {
        if let Some(uopr) = self.uopr {
            SimpleExpr::Unary(uopr, Box::new(self.left.unwrap()))
        } else if let Some(bopr) = self.bopr {
            SimpleExpr::Binary(
                Box::new(self.left.unwrap()),
                bopr,
                Box::new(self.right.unwrap()),
            )
        } else if let Some(func) = self.func {
            SimpleExpr::FunctionCall(func, self.args)
        } else if let Some(left) = self.left {
            left
        } else {
            panic!("incomplete expression")
        }
    }
}

impl Into<SelectExpr> for Expr {
    fn into(self) -> SelectExpr {
        self.into_simple_expr().into()
    }
}

impl SimpleExpr {
    /// Express a logical `AND` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .or_where(Expr::col(Char::SizeW).eq(1).and(Expr::col(Char::SizeH).eq(2)))
    ///     .or_where(Expr::col(Char::SizeW).eq(3).and(Expr::col(Char::SizeH).eq(4)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE ((`size_w` = 1) AND (`size_h` = 2)) OR ((`size_w` = 3) AND (`size_h` = 4))"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE (("size_w" = 1) AND ("size_h" = 2)) OR (("size_w" = 3) AND ("size_h" = 4))"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE ((`size_w` = 1) AND (`size_h` = 2)) OR ((`size_w` = 3) AND (`size_h` = 4))"#
    /// );
    /// ```
    pub fn and(self, right: SimpleExpr) -> Self {
        self.binary(BinOper::And, right)
    }

    /// Express a logical `OR` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).eq(1).or(Expr::col(Char::SizeH).eq(2)))
    ///     .and_where(Expr::col(Char::SizeW).eq(3).or(Expr::col(Char::SizeH).eq(4)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE ((`size_w` = 1) OR (`size_h` = 2)) AND ((`size_w` = 3) OR (`size_h` = 4))"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE (("size_w" = 1) OR ("size_h" = 2)) AND (("size_w" = 3) OR ("size_h" = 4))"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE ((`size_w` = 1) OR (`size_h` = 2)) AND ((`size_w` = 3) OR (`size_h` = 4))"#
    /// );
    /// ```
    pub fn or(self, right: SimpleExpr) -> Self {
        self.binary(BinOper::Or, right)
    }

    /// Compares with another [`SimpleExpr`] for equality.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Char::Character)
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).mul(2).equals(Expr::col(Char::SizeH).mul(3)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character` FROM `character` WHERE `size_w` * 2 = `size_h` * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character" FROM "character" WHERE "size_w" * 2 = "size_h" * 3"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character` FROM `character` WHERE `size_w` * 2 = `size_h` * 3"#
    /// );
    /// ```
    pub fn equals<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Equal, right.into())
    }

    /// Compares with another [`SimpleExpr`] for inequality.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .column(Char::Character)
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).mul(2).not_equals(Expr::col(Char::SizeH)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character` FROM `character` WHERE `size_w` * 2 <> `size_h`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character" FROM "character" WHERE "size_w" * 2 <> "size_h""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character` FROM `character` WHERE `size_w` * 2 <> `size_h`"#
    /// );
    /// ```
    pub fn not_equals<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::NotEqual, right.into())
    }

    /// Perform addition with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col(Char::SizeW).max().add(Expr::col(Char::SizeH).max()))
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) + MAX(`size_h`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MAX("size_w") + MAX("size_h") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) + MAX(`size_h`) FROM `character`"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Add, right.into())
    }

    /// Perform subtraction with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col(Char::SizeW).max().sub(Expr::col(Char::SizeW).min()))
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) - MIN(`size_w`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MAX("size_w") - MIN("size_w") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) - MIN(`size_w`) FROM `character`"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn sub<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Sub, right.into())
    }

    /// Select an espace character in a LIKE .
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::tbl(Char::Table, Char::Character).like(r"|_Our|_").escape('|'))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`character` LIKE '|_Our|_' ESCAPE '|'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."character" LIKE '|_Our|_' ESCAPE '|'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`character` LIKE '|_Our|_' ESCAPE '|'"#
    /// );
    /// ```
    pub fn escape(self, right: char) -> Self {
        self.binary(BinOper::Escape, Expr::cust(&format!("'{}'", right)))
    }

    pub(crate) fn binary(self, op: BinOper, right: SimpleExpr) -> Self {
        SimpleExpr::Binary(Box::new(self), op, Box::new(right))
    }

    #[allow(dead_code)]
    pub(crate) fn static_conditions<T, F>(self, b: bool, if_true: T, if_false: F) -> Self
    where
        T: FnOnce(Self) -> Self,
        F: FnOnce(Self) -> Self,
    {
        if b {
            if_true(self)
        } else {
            if_false(self)
        }
    }

    pub(crate) fn need_parentheses(&self) -> bool {
        match self {
            Self::Binary(left, oper, _) => !matches!(
                (left.as_ref(), oper),
                (Self::Binary(_, BinOper::And, _), BinOper::And)
                    | (Self::Binary(_, BinOper::Or, _), BinOper::Or)
            ),
            _ => false,
        }
    }

    pub(crate) fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_, _, _))
    }

    pub(crate) fn is_logical(&self) -> bool {
        match self {
            Self::Binary(_, op, _) => {
                matches!(op, BinOper::And | BinOper::Or)
            }
            _ => false,
        }
    }

    pub(crate) fn is_between(&self) -> bool {
        matches!(
            self,
            Self::Binary(_, BinOper::Between, _) | Self::Binary(_, BinOper::NotBetween, _)
        )
    }

    pub(crate) fn is_values(&self) -> bool {
        matches!(self, Self::Values(_))
    }

    pub(crate) fn get_values(&self) -> &Vec<Value> {
        match self {
            Self::Values(vec) => vec,
            _ => panic!("not Values"),
        }
    }

    pub(crate) fn get_bin_oper(&self) -> Option<BinOper> {
        match self {
            Self::Binary(_, oper, _) => Some(*oper),
            _ => None,
        }
    }
}
