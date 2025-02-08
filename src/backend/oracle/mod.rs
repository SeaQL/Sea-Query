pub(crate) mod foreign_key;
pub(crate) mod index;
pub(crate) mod query;
pub(crate) mod table;
pub(crate) mod types;

use super::*;

/// Postgres query builder.
#[derive(Debug)]
pub struct OracleQueryBuilder;

impl Default for OracleQueryBuilder {
    fn default() -> Self {
        Self
    }
}

impl GenericBuilder for OracleQueryBuilder {}

impl SchemaBuilder for OracleQueryBuilder {}

impl QuotedBuilder for OracleQueryBuilder {
    fn quote(&self) -> char {
        '"'
    }
}
