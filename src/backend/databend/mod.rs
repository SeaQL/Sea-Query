pub(crate) mod foreign_key;
pub(crate) mod index;
pub(crate) mod query;
pub(crate) mod table;

use super::*;

/// Sqlite query builder.
#[derive(Default, Debug)]
pub struct DatabendQueryBuilder;

impl GenericBuilder for DatabendQueryBuilder {}

impl SchemaBuilder for DatabendQueryBuilder {}

impl QuotedBuilder for DatabendQueryBuilder {
    fn quote(&self) -> Quote {
        MysqlQueryBuilder.quote()
    }
}

impl EscapeBuilder for DatabendQueryBuilder {}

impl TableRefBuilder for DatabendQueryBuilder {}

impl PrecedenceDecider for DatabendQueryBuilder {
    fn inner_expr_well_known_greater_precedence(
        &self,
        inner: &SimpleExpr,
        outer_oper: &Oper,
    ) -> bool {
        common_inner_expr_well_known_greater_precedence(inner, outer_oper)
    }
}

impl OperLeftAssocDecider for DatabendQueryBuilder {
    fn well_known_left_associative(&self, op: &BinOper) -> bool {
        common_well_known_left_associative(op)
    }
}
