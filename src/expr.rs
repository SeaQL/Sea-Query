//! Building blocks of SQL statements.
//!
//! [`Expr`] representing the primitive building block in the expressions.
//!
//! [`SimpleExpr`] is the expression common among select fields, where clauses and many other places.

use crate::{func::*, query::*, types::*, value::*};

/// Helper to build a [`SimpleExpr`].
#[derive(Debug, Clone)]
pub struct Expr {
    pub(crate) left: SimpleExpr,
    pub(crate) right: Option<SimpleExpr>,
    pub(crate) uopr: Option<UnOper>,
    pub(crate) bopr: Option<BinOper>,
}

/// Represents a Simple Expression in SQL.
///
/// [`SimpleExpr`] is a node in the expression tree and can represent identifiers, function calls,
/// various operators and sub-queries.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleExpr {
    Column(ColumnRef),
    Tuple(Vec<SimpleExpr>),
    Unary(UnOper, Box<SimpleExpr>),
    FunctionCall(FunctionCall),
    Binary(Box<SimpleExpr>, BinOper, Box<SimpleExpr>),
    SubQuery(Option<SubQueryOper>, Box<SubQueryStatement>),
    Value(Value),
    Values(Vec<Value>),
    Custom(String),
    CustomWithExpr(String, Vec<SimpleExpr>),
    Keyword(Keyword),
    AsEnum(DynIden, Box<SimpleExpr>),
    Case(Box<CaseStatement>),
    Constant(Value),
}

pub trait IntoSimpleExpr {
    fn into_sub_query_expr(self) -> SimpleExpr;
}

pub(crate) mod private {
    use crate::{BinOper, LikeExpr, SimpleExpr, UnOper};

    pub trait Expression: Sized {
        fn un_op(self, o: UnOper) -> SimpleExpr;

        fn bin_op<O, T>(self, op: O, right: T) -> SimpleExpr
        where
            O: Into<BinOper>,
            T: Into<SimpleExpr>;

        fn like_like<O>(self, op: O, like: LikeExpr) -> SimpleExpr
        where
            O: Into<BinOper>,
        {
            self.bin_op(
                op,
                match like.escape {
                    Some(escape) => SimpleExpr::Binary(
                        Box::new(like.pattern.into()),
                        BinOper::Escape,
                        Box::new(SimpleExpr::Constant(escape.into())),
                    ),
                    None => like.pattern.into(),
                },
            )
        }
    }
}

use private::Expression;

impl Expression for Expr {
    fn un_op(mut self, o: UnOper) -> SimpleExpr {
        self.uopr = Some(o);
        self.into()
    }

    fn bin_op<O, T>(mut self, op: O, right: T) -> SimpleExpr
    where
        O: Into<BinOper>,
        T: Into<SimpleExpr>,
    {
        self.bopr = Some(op.into());
        self.right = Some(right.into());
        self.into()
    }
}

impl Expr {
    fn new_with_left<T>(left: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        let left = left.into();
        Self {
            left,
            right: None,
            uopr: None,
            bopr: None,
        }
    }

    #[deprecated(since = "0.29.0", note = "Please use the [`Asterisk`]")]
    pub fn asterisk() -> Self {
        Self::col(Asterisk)
    }

    /// Express the target column without table prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 1"#
    /// );
    /// ```
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" = 1"#
    /// );
    /// ```
    pub fn col<T>(n: T) -> Self
    where
        T: IntoColumnRef,
    {
        Self::new_with_left(n.into_column_ref())
    }

    /// Wraps tuple of `SimpleExpr`, can be used for tuple comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(
    ///         Expr::tuple([Expr::col(Char::SizeW).into(), Expr::value(100)])
    ///             .lt(Expr::tuple([Expr::value(500), Expr::value(100)])))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE (`size_w`, 100) < (500, 100)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w", 100) < (500, 100)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w", 100) < (500, 100)"#
    /// );
    /// ```
    pub fn tuple<I>(n: I) -> Self
    where
        I: IntoIterator<Item = SimpleExpr>,
    {
        Expr::expr(SimpleExpr::Tuple(
            n.into_iter().collect::<Vec<SimpleExpr>>(),
        ))
    }

    #[deprecated(since = "0.29.0", note = "Please use the [`Asterisk`]")]
    pub fn table_asterisk<T>(t: T) -> Self
    where
        T: IntoIden,
    {
        Self::col((t.into_iden(), Asterisk))
    }

    /// Express a [`Value`], returning a [`Expr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// ```
    pub fn val<V>(v: V) -> Self
    where
        V: Into<Value>,
    {
        Self::new_with_left(v)
    }

    /// Wrap a [`SimpleExpr`] and perform some operation on it.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE IFNULL("size_w", 0) > 2"#
    /// );
    /// ```
    #[allow(clippy::self_named_constructors)]
    pub fn expr<T>(expr: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        Self::new_with_left(expr)
    }

    /// Express a [`Value`], returning a [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::value(1))
    ///     .and_where(Expr::value(2.5))
    ///     .and_where(Expr::value("3"))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 AND 2.5 AND '3'"#
    /// );
    /// ```
    pub fn value<V>(v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        v.into()
    }

    /// Express any custom expression in [`&str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::cust("1 = 1"))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 = 1"#
    /// );
    /// ```
    pub fn cust<T>(s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        SimpleExpr::Custom(s.into())
    }

    /// Express any custom expression with [`Value`]. Use this if your expression needs variables.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::Id).eq(1))
    ///     .and_where(Expr::cust_with_values("6 = ? * ?", [2, 3]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `id` = 1 AND (6 = 2 * 3)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "id" = 1 AND (6 = 2 * 3)"#
    /// );
    /// ```
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::Id).eq(1))
    ///     .and_where(Expr::cust_with_values("6 = $2 * $1", [3, 2]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "id" = 1 AND (6 = 2 * 3)"#
    /// );
    /// ```
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("6 = ? * ?", [2, 3]))
    ///     .to_owned();
    ///
    /// assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT 6 = 2 * 3"#);
    /// assert_eq!(query.to_string(SqliteQueryBuilder), r#"SELECT 6 = 2 * 3"#);
    /// ```
    /// Postgres only: use `$$` to escape `$`
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("$1 $$ $2", ["a", "b"]))
    ///     .to_owned();
    ///
    /// assert_eq!(query.to_string(PostgresQueryBuilder), r#"SELECT 'a' $ 'b'"#);
    /// ```
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_values("data @? ($1::JSONPATH)", ["hello"]))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT data @? ('hello'::JSONPATH)"#
    /// );
    /// ```
    pub fn cust_with_values<T, V, I>(s: T, v: I) -> SimpleExpr
    where
        T: Into<String>,
        V: Into<Value>,
        I: IntoIterator<Item = V>,
    {
        SimpleExpr::CustomWithExpr(
            s.into(),
            v.into_iter()
                .map(|v| Into::<Value>::into(v).into())
                .collect(),
        )
    }

    /// Express any custom expression with [`SimpleExpr`]. Use this if your expression needs other expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::val(1).add(2))
    ///     .expr(Expr::cust_with_expr("data @? ($1::JSONPATH)", "hello"))
    ///     .to_owned();
    /// let (sql, values) = query.build(PostgresQueryBuilder);
    ///
    /// assert_eq!(sql, r#"SELECT $1 + $2, data @? ($3::JSONPATH)"#);
    /// assert_eq!(
    ///     values,
    ///     Values(vec![1i32.into(), 2i32.into(), "hello".into()])
    /// );
    /// ```
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::cust_with_expr(
    ///         "json_agg(DISTINCT $1)",
    ///         Expr::col(Char::Character),
    ///     ))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT json_agg(DISTINCT "character")"#
    /// );
    /// ```
    pub fn cust_with_expr<T, E>(s: T, expr: E) -> SimpleExpr
    where
        T: Into<String>,
        E: Into<SimpleExpr>,
    {
        SimpleExpr::CustomWithExpr(s.into(), vec![expr.into()])
    }

    /// Express any custom expression with [`SimpleExpr`]. Use this if your expression needs other expressions.
    pub fn cust_with_exprs<T, I>(s: T, v: I) -> SimpleExpr
    where
        T: Into<String>,
        I: IntoIterator<Item = SimpleExpr>,
    {
        SimpleExpr::CustomWithExpr(s.into(), v.into_iter().collect())
    }

    /// Express an equal (`=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'What!' = 'Nothing' AND "id" = 1"#
    /// );
    /// ```
    pub fn eq<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Equal, v)
    }

    /// Express a not equal (`<>`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'Morning' <> 'Good' AND "id" <> 1"#
    /// );
    /// ```
    pub fn ne<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::NotEqual, v)
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::FontId)).equals((Font::Table, Font::Id)))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."font_id" = "font"."id""#
    /// );
    /// ```
    pub fn equals<C>(self, col: C) -> SimpleExpr
    where
        C: IntoColumnRef,
    {
        self.binary(BinOper::Equal, col.into_column_ref())
    }

    /// Express a not equal expression between two table columns,
    /// you will mainly use this to relate identical value between two table columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::FontId)).equals((Font::Table, Font::Id)))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."font_id" = "font"."id""#
    /// );
    /// ```
    pub fn not_equals<C>(self, col: C) -> SimpleExpr
    where
        C: IntoColumnRef,
    {
        self.binary(BinOper::NotEqual, col.into_column_ref())
    }

    /// Express a greater than (`>`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).gt(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" > 2"#
    /// );
    /// ```
    pub fn gt<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::GreaterThan, v.into())
    }

    /// Express a greater than or equal (`>=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).gte(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" >= 2"#
    /// );
    /// ```
    pub fn gte<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::GreaterThanOrEqual, v)
    }

    /// Express a less than (`<`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).lt(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" < 2"#
    /// );
    /// ```
    pub fn lt<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::SmallerThan, v)
    }

    /// Express a less than or equal (`<=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).lte(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" <= 2"#
    /// );
    /// ```
    pub fn lte<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::SmallerThanOrEqual, v)
    }

    /// Express an arithmetic addition operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).add(1).eq(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 + 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Add, v)
    }

    /// Express an arithmetic subtraction operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).sub(1).eq(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 - 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn sub<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Sub, v)
    }

    /// Express an arithmetic multiplication operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).mul(1).eq(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 * 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn mul<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Mul, v)
    }

    /// Express an arithmetic division operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).div(1).eq(2))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 / 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn div<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Div, v)
    }

    /// Express an arithmetic modulo operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).modulo(1).eq(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 % 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 % 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 % 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn modulo<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Mod, v)
    }

    /// Express a bitwise left shift.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).left_shift(1).eq(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 << 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 << 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 << 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn left_shift<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::LShift, v)
    }

    /// Express a bitwise right shift.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::val(1).right_shift(1).eq(2))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 1 >> 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 >> 1 = 2"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 1 >> 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn right_shift<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::RShift, v)
    }

    /// Express a `BETWEEN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).between(1, 10))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" BETWEEN 1 AND 10"#
    /// );
    /// ```
    pub fn between<V>(self, a: V, b: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).not_between(1, 10))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" NOT BETWEEN 1 AND 10"#
    /// );
    /// ```
    pub fn not_between<V>(self, a: V, b: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.between_or_not_between(BinOper::NotBetween, a, b)
    }

    fn between_or_not_between<V>(self, op: BinOper, a: V, b: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(
            op,
            SimpleExpr::Binary(Box::new(a.into()), BinOper::And, Box::new(b.into())),
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::Character)).like("Ours'%"))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."character" LIKE 'Ours''%'"#
    /// );
    /// ```
    ///
    /// Like with ESCAPE
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::Character)).like(LikeExpr::new(r"|_Our|_").escape('|')))
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."character" LIKE '|_Our|_' ESCAPE '|'"#
    /// );
    /// ```
    pub fn like<L: IntoLikeExpr>(self, like: L) -> SimpleExpr {
        self.like_like(BinOper::Like, like.into_like_expr())
    }

    /// Express a `NOT LIKE` expression
    pub fn not_like<L: IntoLikeExpr>(self, like: L) -> SimpleExpr {
        self.like_like(BinOper::NotLike, like.into_like_expr())
    }

    /// Express a `IS NULL` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_null())
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" IS NULL"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_null(self) -> SimpleExpr {
        self.binary(BinOper::Is, Keyword::Null)
    }

    /// Express a `IS` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::Ascii)).is(true))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`ascii` IS TRUE"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."ascii" IS TRUE"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."ascii" IS TRUE"#
    /// );
    /// ```
    pub fn is<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Is, v)
    }

    /// Express a `IS NOT NULL` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_not_null())
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."size_w" IS NOT NULL"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_null(self) -> SimpleExpr {
        self.binary(BinOper::IsNot, Keyword::Null)
    }

    /// Express a `IS NOT` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::Ascii)).is_not(true))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `character`.`ascii` IS NOT TRUE"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."ascii" IS NOT TRUE"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "character"."ascii" IS NOT TRUE"#
    /// );
    /// ```
    pub fn is_not<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::IsNot, v)
    }

    /// Create any binary operation
    ///
    /// # Examples
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .cond_where(all![
    ///         Expr::col(Char::SizeW).binary(BinOper::SmallerThan, 10),
    ///         Expr::col(Char::SizeW).binary(BinOper::GreaterThan, Expr::col(Char::SizeH))
    ///     ])
    ///     .to_owned();
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE `size_w` < 10 AND `size_w` > `size_h`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" < 10 AND "size_w" > "size_h""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" < 10 AND "size_w" > "size_h""#
    /// );
    /// ```
    pub fn binary<O, T>(self, op: O, right: T) -> SimpleExpr
    where
        O: Into<BinOper>,
        T: Into<SimpleExpr>,
    {
        self.bin_op(op, right)
    }

    /// Negates an expression with `NOT`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::expr(Expr::col((Char::Table, Char::SizeW)).is_null()).not())
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE NOT "character"."size_w" IS NULL"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> SimpleExpr {
        self.un_op(UnOper::Not)
    }

    /// Express a `MAX` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col((Char::Table, Char::SizeW)).max())
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
    ///     r#"SELECT MAX("character"."size_w") FROM "character""#
    /// );
    /// ```
    pub fn max(self) -> SimpleExpr {
        Func::max(self.left).into()
    }

    /// Express a `MIN` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col((Char::Table, Char::SizeW)).min())
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
    ///     r#"SELECT MIN("character"."size_w") FROM "character""#
    /// );
    /// ```
    pub fn min(self) -> SimpleExpr {
        Func::min(self.left).into()
    }

    /// Express a `SUM` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col((Char::Table, Char::SizeW)).sum())
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
    ///     r#"SELECT SUM("character"."size_w") FROM "character""#
    /// );
    /// ```
    pub fn sum(self) -> SimpleExpr {
        Func::sum(self.left).into()
    }

    /// Express a `COUNT` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col((Char::Table, Char::SizeW)).count())
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
    ///     r#"SELECT COUNT("character"."size_w") FROM "character""#
    /// );
    /// ```
    pub fn count(self) -> SimpleExpr {
        Func::count(self.left).into()
    }

    /// Express a `IF NULL` function.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col((Char::Table, Char::SizeW)).if_null(0))
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
    ///     r#"SELECT IFNULL("character"."size_w", 0) FROM "character""#
    /// );
    /// ```
    pub fn if_null<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        Func::if_null(self.left, v).into()
    }

    /// Express a `IN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_in([1, 2, 3]))
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
    ///     r#"SELECT "id" FROM "character" WHERE "character"."size_w" IN (1, 2, 3)"#
    /// );
    /// ```
    /// Empty value list
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_in(Vec::<u8>::new()))
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
    ///     r#"SELECT "id" FROM "character" WHERE 1 = 2"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_in<V, I>(mut self, v: I) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
        I: IntoIterator<Item = V>,
    {
        self.bopr = Some(BinOper::In);
        self.right = Some(SimpleExpr::Tuple(v.into_iter().map(|v| v.into()).collect()));
        self.into()
    }

    /// Express a `IN` sub expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::FontId])
    ///     .from(Char::Table)
    ///     .and_where(
    ///         Expr::tuple([
    ///             Expr::col(Char::Character).into(),
    ///             Expr::col(Char::FontId).into(),
    ///         ])
    ///         .in_tuples([(1, String::from("1")), (2, String::from("2"))])
    ///     )
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `font_id` FROM `character` WHERE (`character`, `font_id`) IN ((1, '1'), (2, '2'))"#
    /// );
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "font_id" FROM "character" WHERE ("character", "font_id") IN ((1, '1'), (2, '2'))"#
    /// );
    ///
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "font_id" FROM "character" WHERE ("character", "font_id") IN ((1, '1'), (2, '2'))"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn in_tuples<V, I>(mut self, v: I) -> SimpleExpr
    where
        V: IntoValueTuple,
        I: IntoIterator<Item = V>,
    {
        self.bopr = Some(BinOper::In);
        self.right = Some(SimpleExpr::Tuple(
            v.into_iter()
                .map(|m| SimpleExpr::Values(m.into_value_tuple().into_iter().collect()))
                .collect(),
        ));
        self.into()
    }

    /// Express a `NOT IN` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_not_in([1, 2, 3]))
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
    ///     r#"SELECT "id" FROM "character" WHERE "character"."size_w" NOT IN (1, 2, 3)"#
    /// );
    /// ```
    /// Empty value list
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Id])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::SizeW)).is_not_in(Vec::<u8>::new()))
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
    ///     r#"SELECT "id" FROM "character" WHERE 1 = 1"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn is_not_in<V, I>(mut self, v: I) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
        I: IntoIterator<Item = V>,
    {
        self.bopr = Some(BinOper::NotIn);
        self.right = Some(SimpleExpr::Tuple(v.into_iter().map(|v| v.into()).collect()));
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" IN (SELECT 3 + 2 * 2)"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn in_subquery(mut self, sel: SelectStatement) -> SimpleExpr {
        self.bopr = Some(BinOper::In);
        self.right = Some(SimpleExpr::SubQuery(
            None,
            Box::new(sel.into_sub_query_statement()),
        ));
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
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
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" NOT IN (SELECT 3 + 2 * 2)"#
    /// );
    /// ```
    #[allow(clippy::wrong_self_convention)]
    pub fn not_in_subquery(mut self, sel: SelectStatement) -> SimpleExpr {
        self.bopr = Some(BinOper::NotIn);
        self.right = Some(SimpleExpr::SubQuery(
            None,
            Box::new(sel.into_sub_query_statement()),
        ));
        self.into()
    }

    /// Express a `EXISTS` sub-query expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr_as(Expr::exists(Query::select().column(Char::Id).from(Char::Table).take()), Alias::new("character_exists"))
    ///     .expr_as(Expr::exists(Query::select().column(Glyph::Id).from(Glyph::Table).take()), Alias::new("glyph_exists"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT EXISTS(SELECT `id` FROM `character`) AS `character_exists`, EXISTS(SELECT `id` FROM `glyph`) AS `glyph_exists`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT EXISTS(SELECT "id" FROM "character") AS "character_exists", EXISTS(SELECT "id" FROM "glyph") AS "glyph_exists""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT EXISTS(SELECT "id" FROM "character") AS "character_exists", EXISTS(SELECT "id" FROM "glyph") AS "glyph_exists""#
    /// );
    /// ```
    pub fn exists(sel: SelectStatement) -> SimpleExpr {
        SimpleExpr::SubQuery(
            Some(SubQueryOper::Exists),
            Box::new(sel.into_sub_query_statement()),
        )
    }

    /// Express a `ANY` sub-query expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Char::Id)
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::Id).eq(Expr::any(
    ///         Query::select().column(Char::Id).from(Char::Table).take(),
    ///     )))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `id` = ANY(SELECT `id` FROM `character`)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE "id" = ANY(SELECT "id" FROM "character")"#
    /// );
    /// ```
    pub fn any(sel: SelectStatement) -> SimpleExpr {
        SimpleExpr::SubQuery(
            Some(SubQueryOper::Any),
            Box::new(sel.into_sub_query_statement()),
        )
    }

    /// Express a `SOME` sub-query expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Char::Id)
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::Id).ne(Expr::some(
    ///         Query::select().column(Char::Id).from(Char::Table).take(),
    ///     )))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `id` FROM `character` WHERE `id` <> SOME(SELECT `id` FROM `character`)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "id" FROM "character" WHERE "id" <> SOME(SELECT "id" FROM "character")"#
    /// );
    /// ```
    pub fn some(sel: SelectStatement) -> SimpleExpr {
        SimpleExpr::SubQuery(
            Some(SubQueryOper::Some),
            Box::new(sel.into_sub_query_statement()),
        )
    }

    /// Express a `ALL` sub-query expression.
    pub fn all(sel: SelectStatement) -> SimpleExpr {
        SimpleExpr::SubQuery(
            Some(SubQueryOper::All),
            Box::new(sel.into_sub_query_statement()),
        )
    }

    /// Express a `AS enum` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::col(Char::FontSize).as_enum(Alias::new("text")))
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `font_size` FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CAST("font_size" AS text) FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "font_size" FROM "character""#
    /// );
    ///
    /// let query = Query::insert()
    ///     .into_table(Char::Table)
    ///     .columns([Char::FontSize])
    ///     .values_panic([Expr::val("large").as_enum(Alias::new("FontSizeEnum"))])
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"INSERT INTO `character` (`font_size`) VALUES ('large')"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"INSERT INTO "character" ("font_size") VALUES (CAST('large' AS FontSizeEnum))"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"INSERT INTO "character" ("font_size") VALUES ('large')"#
    /// );
    /// ```
    pub fn as_enum<T>(self, type_name: T) -> SimpleExpr
    where
        T: IntoIden,
    {
        SimpleExpr::AsEnum(type_name.into_iden(), Box::new(self.into()))
    }

    /// Adds new `CASE WHEN` to existing case statement.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .expr_as(
    ///         Expr::case(
    ///                 Expr::col((Glyph::Table, Glyph::Aspect)).is_in([2, 4]),
    ///                 true
    ///              )
    ///             .finally(false),
    ///          Alias::new("is_even")
    ///     )
    ///     .from(Glyph::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT (CASE WHEN ("glyph"."aspect" IN (2, 4)) THEN TRUE ELSE FALSE END) AS "is_even" FROM "glyph""#
    /// );
    /// ```
    pub fn case<C, T>(cond: C, then: T) -> CaseStatement
    where
        C: IntoCondition,
        T: Into<SimpleExpr>,
    {
        CaseStatement::new().case(cond, then)
    }

    /// Express a `CAST AS` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::val("1").cast_as(Alias::new("integer")))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// ```
    pub fn cast_as<T>(self, type_name: T) -> SimpleExpr
    where
        T: IntoIden,
    {
        let func = Func::cast_as(self, type_name);
        SimpleExpr::FunctionCall(func)
    }

    /// Keyword `CURRENT_TIMESTAMP`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::*;
    ///
    /// let query = Query::select().expr(Expr::current_date()).to_owned();
    ///
    /// assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT CURRENT_DATE"#);
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CURRENT_DATE"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT CURRENT_DATE"#
    /// );
    /// ```
    pub fn current_date() -> Expr {
        Expr::new_with_left(Keyword::CurrentDate)
    }

    /// Keyword `CURRENT_TIMESTAMP`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::*;
    ///
    /// let query = Query::select().expr(Expr::current_time()).to_owned();
    ///
    /// assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT CURRENT_TIME"#);
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CURRENT_TIME"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT CURRENT_TIME"#
    /// );
    /// ```
    pub fn current_time() -> Expr {
        Expr::new_with_left(Keyword::CurrentTime)
    }

    /// Keyword `CURRENT_TIMESTAMP`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{Expr, MysqlQueryBuilder, PostgresQueryBuilder, Query, SqliteQueryBuilder};
    ///
    /// let query = Query::select().expr(Expr::current_timestamp()).to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT CURRENT_TIMESTAMP"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CURRENT_TIMESTAMP"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT CURRENT_TIMESTAMP"#
    /// );
    /// ```
    pub fn current_timestamp() -> Expr {
        Expr::new_with_left(Keyword::CurrentTimestamp)
    }

    /// Custom keyword.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::*;
    ///
    /// let query = Query::select()
    ///     .expr(Expr::custom_keyword(Alias::new("test")))
    ///     .to_owned();
    ///
    /// assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT test"#);
    /// assert_eq!(query.to_string(PostgresQueryBuilder), r#"SELECT test"#);
    /// assert_eq!(query.to_string(SqliteQueryBuilder), r#"SELECT test"#);
    /// ```
    pub fn custom_keyword<T>(i: T) -> Expr
    where
        T: IntoIden,
    {
        Expr::new_with_left(Keyword::Custom(i.into_iden()))
    }
}

impl From<Expr> for SimpleExpr {
    /// Convert into SimpleExpr
    fn from(src: Expr) -> Self {
        if let Some(uopr) = src.uopr {
            SimpleExpr::Unary(uopr, Box::new(src.left))
        } else if let Some(bopr) = src.bopr {
            SimpleExpr::Binary(Box::new(src.left), bopr, Box::new(src.right.unwrap()))
        } else {
            src.left
        }
    }
}

impl<T> From<T> for SimpleExpr
where
    T: Into<Value>,
{
    fn from(v: T) -> Self {
        SimpleExpr::Value(v.into())
    }
}

impl From<FunctionCall> for SimpleExpr {
    fn from(func: FunctionCall) -> Self {
        SimpleExpr::FunctionCall(func)
    }
}

impl From<ColumnRef> for SimpleExpr {
    fn from(col: ColumnRef) -> Self {
        SimpleExpr::Column(col)
    }
}

impl From<Keyword> for SimpleExpr {
    fn from(k: Keyword) -> Self {
        SimpleExpr::Keyword(k)
    }
}

impl Expression for SimpleExpr {
    fn un_op(self, op: UnOper) -> SimpleExpr {
        SimpleExpr::Unary(op, Box::new(self))
    }

    fn bin_op<O, T>(self, op: O, right: T) -> SimpleExpr
    where
        O: Into<BinOper>,
        T: Into<SimpleExpr>,
    {
        SimpleExpr::Binary(Box::new(self), op.into(), Box::new(right.into()))
    }
}

impl SimpleExpr {
    /// Negates an expression with `NOT`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .column(Char::SizeW)
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).eq(1).not())
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `size_w` FROM `character` WHERE NOT `size_w` = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "size_w" FROM "character" WHERE NOT "size_w" = 1"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "size_w" FROM "character" WHERE NOT "size_w" = 1"#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> SimpleExpr {
        self.un_op(UnOper::Not)
    }

    /// Express a logical `AND` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .cond_where(any![
    ///         Expr::col(Char::SizeW).eq(1).and(Expr::col(Char::SizeH).eq(2)),
    ///         Expr::col(Char::SizeW).eq(3).and(Expr::col(Char::SizeH).eq(4)),
    ///     ])
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE (`size_w` = 1 AND `size_h` = 2) OR (`size_w` = 3 AND `size_h` = 4)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" = 1 AND "size_h" = 2) OR ("size_w" = 3 AND "size_h" = 4)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" = 1 AND "size_h" = 2) OR ("size_w" = 3 AND "size_h" = 4)"#
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
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col(Char::SizeW).eq(1).or(Expr::col(Char::SizeH).eq(2)))
    ///     .and_where(Expr::col(Char::SizeW).eq(3).or(Expr::col(Char::SizeH).eq(4)))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE (`size_w` = 1 OR `size_h` = 2) AND (`size_w` = 3 OR `size_h` = 4)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" = 1 OR "size_h" = 2) AND ("size_w" = 3 OR "size_h" = 4)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" = 1 OR "size_h" = 2) AND ("size_w" = 3 OR "size_h" = 4)"#
    /// );
    /// ```
    pub fn or(self, right: SimpleExpr) -> Self {
        self.binary(BinOper::Or, right)
    }

    /// Express an equal (`=`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::value("What!").eq("Nothing"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'What!' = 'Nothing'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'What!' = 'Nothing'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'What!' = 'Nothing'"#
    /// );
    /// ```
    pub fn eq<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::Equal, v)
    }

    /// Express a not equal (`<>`) expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::value("Morning").ne("Good"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 'Morning' <> 'Good'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'Morning' <> 'Good'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 'Morning' <> 'Good'"#
    /// );
    /// ```
    pub fn ne<V>(self, v: V) -> SimpleExpr
    where
        V: Into<SimpleExpr>,
    {
        self.binary(BinOper::NotEqual, v)
    }

    /// Perform addition with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(
    ///         Expr::col(Char::SizeW)
    ///             .max()
    ///             .add(Expr::col(Char::SizeH).max()),
    ///     )
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
    ///     r#"SELECT MAX("size_w") + MAX("size_h") FROM "character""#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn add<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Add, right)
    }

    /// Perform multiplication with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(
    ///         Expr::col(Char::SizeW)
    ///             .max()
    ///             .mul(Expr::col(Char::SizeH).max()),
    ///     )
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) * MAX(`size_h`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MAX("size_w") * MAX("size_h") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MAX("size_w") * MAX("size_h") FROM "character""#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn mul<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Mul, right.into())
    }

    /// Perform division with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(
    ///         Expr::col(Char::SizeW)
    ///             .max()
    ///             .div(Expr::col(Char::SizeH).max()),
    ///     )
    ///     .from(Char::Table)
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT MAX(`size_w`) / MAX(`size_h`) FROM `character`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT MAX("size_w") / MAX("size_h") FROM "character""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT MAX("size_w") / MAX("size_h") FROM "character""#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn div<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Div, right.into())
    }

    /// Perform subtraction with another [`SimpleExpr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(
    ///         Expr::col(Char::SizeW)
    ///             .max()
    ///             .sub(Expr::col(Char::SizeW).min()),
    ///     )
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
    ///     r#"SELECT MAX("size_w") - MIN("size_w") FROM "character""#
    /// );
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn sub<T>(self, right: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.binary(BinOper::Sub, right)
    }

    /// Express a `CAST AS` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{tests_cfg::*, *};
    ///
    /// let query = Query::select()
    ///     .expr(Expr::value("1").cast_as(Alias::new("integer")))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT CAST('1' AS integer)"#
    /// );
    /// ```
    pub fn cast_as<T>(self, type_name: T) -> Self
    where
        T: IntoIden,
    {
        let func = Func::cast_as(self, type_name);
        Self::FunctionCall(func)
    }

    /// Create any binary operation
    ///
    /// # Examples
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .cond_where(all![
    ///         Expr::value(10).binary(BinOper::SmallerThan, Expr::col(Char::SizeW)),
    ///         Expr::value(20).binary(BinOper::GreaterThan, Expr::col(Char::SizeH))
    ///     ])
    ///     .to_owned();
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE 10 < `size_w` AND 20 > `size_h`"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 10 < "size_w" AND 20 > "size_h""#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE 10 < "size_w" AND 20 > "size_h""#
    /// );
    pub fn binary<O, T>(self, op: O, right: T) -> Self
    where
        O: Into<BinOper>,
        T: Into<SimpleExpr>,
    {
        self.bin_op(op, right)
    }

    /// Express a `LIKE` expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_query::{*, tests_cfg::*};
    ///
    /// let query = Query::select()
    ///     .columns([Char::Character, Char::SizeW, Char::SizeH])
    ///     .from(Char::Table)
    ///     .and_where(Expr::col((Char::Table, Char::FontId)).cast_as(Alias::new("TEXT")).like("a%"))
    ///     .to_owned();
    ///
    /// assert_eq!(
    ///     query.to_string(MysqlQueryBuilder),
    ///     r#"SELECT `character`, `size_w`, `size_h` FROM `character` WHERE CAST(`character`.`font_id` AS TEXT) LIKE 'a%'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(PostgresQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE CAST("character"."font_id" AS TEXT) LIKE 'a%'"#
    /// );
    /// assert_eq!(
    ///     query.to_string(SqliteQueryBuilder),
    ///     r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE CAST("character"."font_id" AS TEXT) LIKE 'a%'"#
    /// );
    /// ```
    pub fn like<L: IntoLikeExpr>(self, like: L) -> Self {
        self.like_like(BinOper::Like, like.into_like_expr())
    }

    /// Express a `NOT LIKE` expression
    pub fn not_like<L: IntoLikeExpr>(self, like: L) -> Self {
        self.like_like(BinOper::NotLike, like.into_like_expr())
    }

    pub(crate) fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_, _, _))
    }

    pub(crate) fn get_bin_oper(&self) -> Option<&BinOper> {
        match self {
            Self::Binary(_, oper, _) => Some(oper),
            _ => None,
        }
    }
}
