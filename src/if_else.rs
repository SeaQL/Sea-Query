use crate::{QueryBuilder, SimpleExpr};

#[derive(Debug, Clone, PartialEq)]
pub struct IfElseStatement {
    pub when: SimpleExpr,
    pub then: SimpleExpr,
    pub otherwise: Option<SimpleExpr>,
}

impl IfElseStatement {
    pub fn new(when: SimpleExpr, then: SimpleExpr, otherwise: Option<SimpleExpr>) -> Self {
        Self {
            when,
            then,
            otherwise,
        }
    }

    pub fn to_string<T: QueryBuilder>(&self, query_builder: T) -> String {
        let mut sql = String::with_capacity(256);
        query_builder.prepare_if_else_statement(&Box::new(self.clone()), &mut sql);
        sql
    }
}
pub trait IfElseStatementBuilder {
    /// Build corresponding SQL statement for certain database backend and return SQL string
    fn build<T: QueryBuilder>(&self, query_builder: T) -> String;

    /// Build corresponding SQL statement for certain database backend and return SQL string
    fn to_string<T: QueryBuilder>(&self, query_builder: T) -> String {
        self.build(query_builder)
    }
}
