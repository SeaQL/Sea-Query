//! Base types used throughout sea-query.

use crate::{expr::*, query::*};
use std::fmt;
pub use std::rc::Rc as SeaRc; // in case we some day we want to use Arc instead of Rc

/// Identifier in query
pub trait Iden {
    fn prepare(&self, s: &mut dyn fmt::Write, q: char) {
        write!(s, "{}{}{}", q, self.quoted(q), q).unwrap();
    }

    fn quoted(&self, q: char) -> String {
        let mut b = [0; 4];
        let qq: &str = q.encode_utf8(&mut b);
        self.to_string().replace(qq, qq.repeat(2).as_str())
    }

    fn to_string(&self) -> String {
        let s = &mut String::new();
        self.unquoted(s);
        s.to_owned()
    }

    fn unquoted(&self, s: &mut dyn fmt::Write);
}

pub type DynIden = SeaRc<dyn Iden>;

pub trait IntoIden {
    fn into_iden(self) -> DynIden;
}

pub trait IdenList {
    type IntoIter: Iterator<Item = DynIden>;

    fn into_iter(self) -> Self::IntoIter;
}

impl fmt::Debug for dyn Iden {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.unquoted(formatter);
        Ok(())
    }
}

/// Column references
#[derive(Debug, Clone)]
pub enum ColumnRef {
    Column(DynIden),
    TableColumn(DynIden, DynIden),
}

pub trait IntoColumnRef {
    fn into_column_ref(self) -> ColumnRef;
}

/// Table references
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum TableRef {
    Table(DynIden),
    SchemaTable(DynIden, DynIden),
    TableAlias(DynIden, DynIden),
    SchemaTableAlias(DynIden, DynIden, DynIden),
    SubQuery(SelectStatement, DynIden),
}

pub trait IntoTableRef {
    fn into_table_ref(self) -> TableRef;
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOper {
    Not,
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOper {
    And,
    Or,
    Like,
    NotLike,
    Is,
    IsNot,
    In,
    NotIn,
    Between,
    NotBetween,
    Equal,
    NotEqual,
    SmallerThan,
    GreaterThan,
    SmallerThanOrEqual,
    GreaterThanOrEqual,
    Add,
    Sub,
    Mul,
    Div,
    Escape,
}

/// Logical chain operator
#[derive(Debug, Clone)]
pub enum LogicalChainOper {
    And(SimpleExpr),
    Or(SimpleExpr),
}

/// Join types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Join,
    InnerJoin,
    LeftJoin,
    RightJoin,
}

/// Order expression
#[derive(Debug, Clone)]
pub struct OrderExpr {
    pub(crate) expr: SimpleExpr,
    pub(crate) order: Order,
}

/// Join on types
#[derive(Debug, Clone)]
pub enum JoinOn {
    Condition(Box<SimpleExpr>),
    Columns(Vec<SimpleExpr>),
}

/// Ordering options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

/// Helper for create name alias
#[derive(Debug, Clone)]
pub struct Alias(String);

/// Common SQL Keywords
#[derive(Debug, Clone)]
pub enum Keyword {
    Null,
    Custom(DynIden),
}

// Impl begins

impl<T: 'static> IntoIden for T
where
    T: Iden,
{
    fn into_iden(self) -> DynIden {
        SeaRc::new(self)
    }
}

impl IntoIden for DynIden {
    fn into_iden(self) -> DynIden {
        self
    }
}

impl<I> IdenList for I
where
    I: IntoIden,
{
    type IntoIter = std::iter::Once<DynIden>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.into_iden())
    }
}

impl<A, B> IdenList for (A, B)
where
    A: IntoIden,
    B: IntoIden,
{
    type IntoIter = std::vec::IntoIter<DynIden>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.0.into_iden(), self.1.into_iden()].into_iter()
    }
}

impl<A, B, C> IdenList for (A, B, C)
where
    A: IntoIden,
    B: IntoIden,
    C: IntoIden,
{
    type IntoIter = std::vec::IntoIter<DynIden>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.0.into_iden(), self.1.into_iden(), self.2.into_iden()].into_iter()
    }
}

impl IntoColumnRef for ColumnRef {
    fn into_column_ref(self) -> ColumnRef {
        self
    }
}

impl<T: 'static> IntoColumnRef for T
where
    T: IntoIden,
{
    fn into_column_ref(self) -> ColumnRef {
        ColumnRef::Column(self.into_iden())
    }
}

impl<S: 'static, T: 'static> IntoColumnRef for (S, T)
where
    S: IntoIden,
    T: IntoIden,
{
    fn into_column_ref(self) -> ColumnRef {
        ColumnRef::TableColumn(self.0.into_iden(), self.1.into_iden())
    }
}

impl IntoTableRef for TableRef {
    fn into_table_ref(self) -> TableRef {
        self
    }
}

impl<T: 'static> IntoTableRef for T
where
    T: IntoIden,
{
    fn into_table_ref(self) -> TableRef {
        TableRef::Table(self.into_iden())
    }
}

impl<S: 'static, T: 'static> IntoTableRef for (S, T)
where
    S: IntoIden,
    T: IntoIden,
{
    fn into_table_ref(self) -> TableRef {
        TableRef::SchemaTable(self.0.into_iden(), self.1.into_iden())
    }
}

impl TableRef {
    /// Add or replace the current alias
    pub fn alias<A: 'static>(self, alias: A) -> Self
    where
        A: IntoIden,
    {
        match self {
            Self::Table(table) => Self::TableAlias(table, alias.into_iden()),
            Self::TableAlias(table, _) => Self::TableAlias(table, alias.into_iden()),
            Self::SchemaTable(schema, table) => {
                Self::SchemaTableAlias(schema, table, alias.into_iden())
            }
            Self::SchemaTableAlias(schema, table, _) => {
                Self::SchemaTableAlias(schema, table, alias.into_iden())
            }
            Self::SubQuery(statement, _) => Self::SubQuery(statement, alias.into_iden()),
        }
    }
}

impl Alias {
    pub fn new(n: &str) -> Self {
        Self(n.to_owned())
    }
}

impl Iden for Alias {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        write!(s, "{}", self.0).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_identifier() {
        let query = Query::select()
            .column(Alias::new("hello-World_"))
            .to_owned();

        assert_eq!(
            query.to_string(MysqlQueryBuilder),
            r#"SELECT `hello-World_`"#
        );
        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            r#"SELECT "hello-World_""#
        );
        assert_eq!(
            query.to_string(SqliteQueryBuilder),
            r#"SELECT `hello-World_`"#
        );
    }

    #[test]
    fn test_quoted_identifier_1() {
        let query = Query::select().column(Alias::new("hel`lo")).to_owned();

        assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT `hel``lo`"#);
        assert_eq!(query.to_string(SqliteQueryBuilder), r#"SELECT `hel``lo`"#);

        let query = Query::select().column(Alias::new("hel\"lo")).to_owned();

        assert_eq!(query.to_string(PostgresQueryBuilder), r#"SELECT "hel""lo""#);
    }

    #[test]
    fn test_quoted_identifier_2() {
        let query = Query::select().column(Alias::new("hel``lo")).to_owned();

        assert_eq!(query.to_string(MysqlQueryBuilder), r#"SELECT `hel````lo`"#);
        assert_eq!(query.to_string(SqliteQueryBuilder), r#"SELECT `hel````lo`"#);

        let query = Query::select().column(Alias::new("hel\"\"lo")).to_owned();

        assert_eq!(
            query.to_string(PostgresQueryBuilder),
            r#"SELECT "hel""""lo""#
        );
    }
}
