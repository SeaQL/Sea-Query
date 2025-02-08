use super::*;

impl IndexBuilder for DatabendQueryBuilder {
    fn prepare_index_create_statement(
        &self,
        _create: &IndexCreateStatement,
        _sql: &mut dyn SqlWriter,
    ) {
    }

    fn prepare_table_ref_index_stmt(&self, _table_ref: &TableRef, _sql: &mut dyn SqlWriter) {}

    fn prepare_index_drop_statement(&self, _drop: &IndexDropStatement, _sql: &mut dyn SqlWriter) {}

    fn prepare_index_prefix(&self, _create: &IndexCreateStatement, _sql: &mut dyn SqlWriter) {}

    fn write_column_index_prefix(&self, _col_prefix: &Option<u32>, _sql: &mut dyn SqlWriter) {}
}
