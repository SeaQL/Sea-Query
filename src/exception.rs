//! Custom SQL exceptions and errors
use inherent::inherent;

use crate::backend::SchemaBuilder;

/// SQL Exceptions
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionStatement {
    pub(crate) message: String,
}

impl ExceptionStatement {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

pub trait ExceptionStatementBuilder {
    /// Build corresponding SQL statement for certain database backend and return SQL string
    fn build<T: SchemaBuilder>(&self, schema_builder: T) -> String;

    /// Build corresponding SQL statement for certain database backend and return SQL string
    fn build_any(&self, schema_builder: &dyn SchemaBuilder) -> String;

    /// Build corresponding SQL statement for certain database backend and return SQL string
    fn to_string<T: SchemaBuilder>(&self, schema_builder: T) -> String {
        self.build(schema_builder)
    }
}

#[inherent]
impl ExceptionStatementBuilder for ExceptionStatement {
    pub fn build<T: SchemaBuilder>(&self, schema_builder: T) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_exception_statement(self, &mut sql);
        sql
    }

    pub fn build_any(&self, schema_builder: &dyn SchemaBuilder) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_exception_statement(self, &mut sql);
        sql
    }

    pub fn to_string<T: SchemaBuilder>(&self, schema_builder: T) -> String;
}
