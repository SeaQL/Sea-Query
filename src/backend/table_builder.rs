use crate::*;

pub trait TableBuilder:
    IndexBuilder + ForeignKeyBuilder + QuotedBuilder + TableRefBuilder + QueryBuilder
{
    /// Translate [`TableCreateStatement`] into SQL statement.
    fn prepare_table_create_statement(
        &self,
        create: &TableCreateStatement,
        sql: &mut dyn SqlWriter,
    ) {
        write!(sql, "CREATE TABLE ").unwrap();

        if create.if_not_exists {
            write!(sql, "IF NOT EXISTS ").unwrap();
        }

        if let Some(table_ref) = &create.table {
            self.prepare_table_ref_table_stmt(table_ref, sql);
        }

        write!(sql, " ( ").unwrap();
        let mut first = true;

        create.columns.iter().for_each(|column_def| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_column_def(column_def, sql);
            first = false;
        });

        create.indexes.iter().for_each(|index| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_table_index_expression(index, sql);
            first = false;
        });

        create.foreign_keys.iter().for_each(|foreign_key| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_foreign_key_create_statement_internal(foreign_key, sql, Mode::Creation);
            first = false;
        });

        create.check.iter().for_each(|check| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_check_constraint(check, sql);
            first = false;
        });

        write!(sql, " )").unwrap();

        for table_opt in create.options.iter() {
            write!(sql, " ").unwrap();
            self.prepare_table_opt(table_opt, sql);
        }
    }

    /// Translate [`TableRef`] into SQL statement.
    fn prepare_table_ref_table_stmt(&self, table_ref: &TableRef, sql: &mut dyn SqlWriter) {
        match table_ref {
            TableRef::Table(_)
            | TableRef::SchemaTable(_, _)
            | TableRef::DatabaseSchemaTable(_, _, _) => self.prepare_table_ref_iden(table_ref, sql),
            _ => panic!("Not supported"),
        }
    }

    /// Translate [`ColumnDef`] into SQL statement.
    fn prepare_column_def(&self, column_def: &ColumnDef, sql: &mut dyn SqlWriter);

    /// Translate [`ColumnType`] into SQL statement.
    fn prepare_column_type(&self, column_type: &ColumnType, sql: &mut dyn SqlWriter);

    /// Translate [`ColumnSpec`] into SQL statement.
    fn prepare_column_spec(&self, column_spec: &ColumnSpec, sql: &mut dyn SqlWriter) {
        match column_spec {
            ColumnSpec::Null => write!(sql, "NULL").unwrap(),
            ColumnSpec::NotNull => write!(sql, "NOT NULL").unwrap(),
            ColumnSpec::Default(value) => {
                write!(sql, "DEFAULT ").unwrap();
                QueryBuilder::prepare_simple_expr(self, value, sql);
            }
            ColumnSpec::AutoIncrement => {
                write!(sql, "{}", self.column_spec_auto_increment_keyword()).unwrap()
            }
            ColumnSpec::UniqueKey => write!(sql, "UNIQUE").unwrap(),
            ColumnSpec::PrimaryKey => write!(sql, "PRIMARY KEY").unwrap(),
            ColumnSpec::Check(check) => self.prepare_check_constraint(check, sql),
            ColumnSpec::Extra(string) => write!(sql, "{string}").unwrap(),
        }
    }

    /// The keyword for setting a column to be auto increment.
    fn column_spec_auto_increment_keyword(&self) -> &str;

    /// Translate [`TableOpt`] into SQL statement.
    fn prepare_table_opt(&self, table_opt: &TableOpt, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "{}",
            match table_opt {
                TableOpt::Engine(s) => format!("ENGINE={s}"),
                TableOpt::Collate(s) => format!("COLLATE={s}"),
                TableOpt::CharacterSet(s) => format!("DEFAULT CHARSET={s}"),
            }
        )
        .unwrap()
    }

    /// Translate [`TablePartition`] into SQL statement.
    fn prepare_table_partition(&self, _table_partition: &TablePartition, _sql: &mut dyn SqlWriter) {
    }

    /// Translate [`TableDropStatement`] into SQL statement.
    fn prepare_table_drop_statement(&self, drop: &TableDropStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "DROP TABLE ").unwrap();

        if drop.if_exists {
            write!(sql, "IF EXISTS ").unwrap();
        }

        drop.tables.iter().fold(true, |first, table| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_table_ref_table_stmt(table, sql);
            false
        });

        for drop_opt in drop.options.iter() {
            self.prepare_table_drop_opt(drop_opt, sql);
        }
    }

    /// Translate [`TableDropOpt`] into SQL statement.
    fn prepare_table_drop_opt(&self, drop_opt: &TableDropOpt, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            " {}",
            match drop_opt {
                TableDropOpt::Restrict => "RESTRICT",
                TableDropOpt::Cascade => "CASCADE",
            }
        )
        .unwrap();
    }

    /// Translate [`TableTruncateStatement`] into SQL statement.
    fn prepare_table_truncate_statement(
        &self,
        truncate: &TableTruncateStatement,
        sql: &mut dyn SqlWriter,
    ) {
        write!(sql, "TRUNCATE TABLE ").unwrap();

        if let Some(table) = &truncate.table {
            self.prepare_table_ref_table_stmt(table, sql);
        }
    }

    /// Translate the check constraint into SQL statement
    fn prepare_check_constraint(&self, check: &SimpleExpr, sql: &mut dyn SqlWriter) {
        write!(sql, "CHECK (").unwrap();
        QueryBuilder::prepare_simple_expr(self, check, sql);
        write!(sql, ")").unwrap();
    }

    /// Translate [`TableAlterStatement`] into SQL statement.
    fn prepare_table_alter_statement(&self, alter: &TableAlterStatement, sql: &mut dyn SqlWriter);

    /// Translate [`TableRenameStatement`] into SQL statement.
    fn prepare_table_rename_statement(
        &self,
        rename: &TableRenameStatement,
        sql: &mut dyn SqlWriter,
    );
}
