use crate::*;

pub trait TableBuilder: IndexBuilder + ForeignKeyBuilder + QuotedBuilder + TableRefBuilder {
    /// Translate [`TableCreateStatement`] into SQL statement.
    fn prepare_table_create_statement(&self, create: &TableCreateStatement, sql: &mut SqlWriter) {
        write!(sql, "CREATE TABLE ").unwrap();

        if create.if_not_exists {
            write!(sql, "IF NOT EXISTS ").unwrap();
        }

        if let Some(table_ref) = &create.table {
            self.prepare_table_ref_table_stmt(table_ref, sql);
        }

        write!(sql, " ( ").unwrap();
        let mut count = 0;

        for column_def in create.columns.iter() {
            if count > 0 {
                write!(sql, ", ").unwrap();
            }
            self.prepare_column_def(column_def, sql);
            count += 1;
        }

        for index in create.indexes.iter() {
            if count > 0 {
                write!(sql, ", ").unwrap();
            }
            self.prepare_table_index_expression(index, sql);
            count += 1;
        }

        for foreign_key in create.foreign_keys.iter() {
            if count > 0 {
                write!(sql, ", ").unwrap();
            }
            self.prepare_foreign_key_create_statement_internal(foreign_key, sql, Mode::Creation);
            count += 1;
        }

        write!(sql, " )").unwrap();

        for table_opt in create.options.iter() {
            write!(sql, " ").unwrap();
            self.prepare_table_opt(table_opt, sql);
        }
    }

    /// Translate [`TableRef`] into SQL statement.
    fn prepare_table_ref_table_stmt(&self, table_ref: &TableRef, sql: &mut SqlWriter) {
        match table_ref {
            TableRef::Table(_)
            | TableRef::SchemaTable(_, _)
            | TableRef::DatabaseSchemaTable(_, _, _) => self.prepare_table_ref_iden(table_ref, sql),
            _ => panic!("Not supported"),
        }
    }

    /// Translate [`ColumnDef`] into SQL statement.
    fn prepare_column_def(&self, column_def: &ColumnDef, sql: &mut SqlWriter);

    /// Translate [`ColumnType`] into SQL statement.
    fn prepare_column_type(&self, column_type: &ColumnType, sql: &mut SqlWriter);

    /// Translate [`ColumnSpec`] into SQL statement.
    fn prepare_column_spec(&self, column_spec: &ColumnSpec, sql: &mut SqlWriter)
    where
        Self: QueryBuilder + Sized,
    {
        match column_spec {
            ColumnSpec::Null => write!(sql, "NULL"),
            ColumnSpec::NotNull => write!(sql, "NOT NULL"),
            ColumnSpec::Default(expr) => {
                write!(
                    sql,
                    "DEFAULT {}",
                    collect_parameters_as_string(self, |sql, collector| {
                        QueryBuilder::prepare_simple_expr(self, expr, sql, collector)
                    })
                )
            }
            ColumnSpec::AutoIncrement => {
                write!(sql, "{}", self.column_spec_auto_increment_keyword())
            }
            ColumnSpec::UniqueKey => write!(sql, "UNIQUE"),
            ColumnSpec::PrimaryKey => write!(sql, "PRIMARY KEY"),
            ColumnSpec::Extra(string) => write!(sql, "{}", string),
        }
        .unwrap()
    }

    /// Translate [`TableOpt`] into SQL statement.
    fn prepare_table_opt(&self, table_opt: &TableOpt, sql: &mut SqlWriter) {
        write!(
            sql,
            "{}",
            match table_opt {
                TableOpt::Engine(s) => format!("ENGINE={}", s),
                TableOpt::Collate(s) => format!("COLLATE={}", s),
                TableOpt::CharacterSet(s) => format!("DEFAULT CHARSET={}", s),
            }
        )
        .unwrap()
    }

    /// Translate [`TablePartition`] into SQL statement.
    fn prepare_table_partition(&self, _table_partition: &TablePartition, _sql: &mut SqlWriter) {}

    /// Translate [`TableDropStatement`] into SQL statement.
    fn prepare_table_drop_statement(&self, drop: &TableDropStatement, sql: &mut SqlWriter) {
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

        let mut table_drop_opt = String::new();
        for drop_opt in drop.options.iter() {
            write!(&mut table_drop_opt, " ").unwrap();
            self.prepare_table_drop_opt(drop_opt, &mut table_drop_opt);
        }
        write!(sql, "{}", table_drop_opt.trim_end()).unwrap();
    }

    /// Translate [`TableDropOpt`] into SQL statement.
    fn prepare_table_drop_opt(&self, drop_opt: &TableDropOpt, sql: &mut dyn std::fmt::Write) {
        write!(
            sql,
            "{}",
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
        sql: &mut SqlWriter,
    ) {
        write!(sql, "TRUNCATE TABLE ").unwrap();

        if let Some(table) = &truncate.table {
            self.prepare_table_ref_table_stmt(table, sql);
        }
    }

    /// Translate [`TableAlterStatement`] into SQL statement.
    fn prepare_table_alter_statement(&self, alter: &TableAlterStatement, sql: &mut SqlWriter);

    /// Translate [`TableRenameStatement`] into SQL statement.
    fn prepare_table_rename_statement(&self, rename: &TableRenameStatement, sql: &mut SqlWriter);

    /// The keyword for setting a column to be auto increment.
    fn column_spec_auto_increment_keyword(&self) -> &str;
}
