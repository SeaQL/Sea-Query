use super::*;

impl TableBuilder for OracleQueryBuilder {
    fn prepare_column_def(&self, column_def: &ColumnDef, sql: &mut SqlWriter) {
        column_def.name.prepare(sql, self.quote());

        self.prepare_column_type_check_auto_increment(column_def, sql);

        for column_spec in column_def.spec.iter() {
            if let ColumnSpec::AutoIncrement = column_spec {
                continue;
            }
            write!(sql, " ").unwrap();
            self.prepare_column_spec(column_spec, sql);
        }
    }

    fn prepare_column_type(&self, column_type: &ColumnType, sql: &mut SqlWriter) {
        write!(
            sql,
            "{}",
            match column_type {
                ColumnType::Char(length) => match length {
                    Some(length) => format!("char({})", length),
                    None => "char".into(),
                },
                ColumnType::String(length) => match length {
                    Some(length) => format!("varchar({})", length),
                    None => "varchar".into(),
                },
                ColumnType::Text => "text".into(),
                ColumnType::TinyInteger(length) | ColumnType::TinyUnsigned(length) =>
                    match length {
                        Some(length) => format!("smallint({})", length),
                        None => "smallint".into(),
                    },
                ColumnType::SmallInteger(length) | ColumnType::SmallUnsigned(length) =>
                    match length {
                        Some(length) => format!("smallint({})", length),
                        None => "smallint".into(),
                    },
                ColumnType::Integer(length) | ColumnType::Unsigned(length) => match length {
                    Some(length) => format!("integer({})", length),
                    None => "integer".into(),
                },
                ColumnType::BigInteger(length) | ColumnType::BigUnsigned(length) => match length {
                    Some(length) => format!("bigint({})", length),
                    None => "bigint".into(),
                },
                ColumnType::Float(precision) => match precision {
                    Some(precision) => format!("real({})", precision),
                    None => "real".into(),
                },
                ColumnType::Double(precision) => match precision {
                    Some(precision) => format!("double precision({})", precision),
                    None => "double precision".into(),
                },
                ColumnType::Decimal(precision) => match precision {
                    Some((precision, scale)) => format!("decimal({}, {})", precision, scale),
                    None => "decimal".into(),
                },
                ColumnType::DateTime(precision) => match precision {
                    Some(precision) => format!("timestamp({}) without time zone", precision),
                    None => "timestamp without time zone".into(),
                },
                ColumnType::Timestamp(precision) => match precision {
                    Some(precision) => format!("timestamp({})", precision),
                    None => "timestamp".into(),
                },
                ColumnType::TimestampWithTimeZone(precision) => match precision {
                    Some(precision) => format!("timestamp({}) with time zone", precision),
                    None => "timestamp with time zone".into(),
                },
                ColumnType::Time(precision) => match precision {
                    Some(precision) => format!("time({})", precision),
                    None => "time".into(),
                },
                ColumnType::Date => "date".into(),
                ColumnType::Binary(length) => match length {
                    Some(_) | None => "bytea".into(),
                },
                ColumnType::Boolean => "bool".into(),
                ColumnType::Money(precision) => match precision {
                    Some((precision, scale)) => format!("money({}, {})", precision, scale),
                    None => "money".into(),
                },
                ColumnType::Json => "json".into(),
                ColumnType::JsonBinary => "jsonb".into(),
                ColumnType::Uuid => "uuid".into(),
                ColumnType::Array(elem_type) => format!("{}[]", elem_type.as_ref().unwrap()),
                ColumnType::Custom(iden) => iden.to_string(),
                ColumnType::Enum(name, _) => name.into(),
                _ => unimplemented!(),
                // ColumnType::Interval(fields, precision) => {
                //     let mut typ = "interval".to_string();
                //     if let Some(fields) = fields {
                //         typ.push_str(&format!(" {}", fields));
                //     }
                //     if let Some(precision) = precision {
                //         typ.push_str(&format!("({})", precision));
                //     }
                //     typ
                // }
            }
        )
        .unwrap()
    }

    fn prepare_column_spec(&self, column_spec: &ColumnSpec, sql: &mut SqlWriter) {
        match column_spec {
            ColumnSpec::Null => write!(sql, "NULL"),
            ColumnSpec::NotNull => write!(sql, "NOT NULL"),
            ColumnSpec::Default(value) => write!(sql, "DEFAULT {}", self.value_to_string(value)),
            ColumnSpec::AutoIncrement => write!(sql, ""),
            ColumnSpec::UniqueKey => write!(sql, "UNIQUE"),
            ColumnSpec::PrimaryKey => write!(sql, "PRIMARY KEY"),
            ColumnSpec::Extra(string) => write!(sql, "{}", string),
        }
        .unwrap()
    }

    fn prepare_table_alter_statement(&self, alter: &TableAlterStatement, sql: &mut SqlWriter) {
        if alter.options.is_empty() {
            panic!("No alter option found")
        };
        write!(sql, "ALTER TABLE ").unwrap();
        if let Some(table) = &alter.table {
            table.prepare(sql, self.quote());
            write!(sql, " ").unwrap();
        }

        alter.options.iter().fold(true, |first, option| {
            if !first {
                write!(sql, ", ").unwrap();
            };
            match option {
                TableAlterOption::AddColumn(AddColumnOption {
                    column,
                    if_not_exists,
                }) => {
                    write!(sql, "ADD COLUMN ").unwrap();
                    if *if_not_exists {
                        write!(sql, "IF NOT EXISTS ").unwrap();
                    }
                    self.prepare_column_def(column, sql);
                }
                TableAlterOption::ModifyColumn(column_def) => {
                    write!(sql, "ALTER COLUMN ").unwrap();
                    column_def.name.prepare(sql, self.quote());
                    write!(sql, " TYPE").unwrap();
                    self.prepare_column_type_check_auto_increment(column_def, sql);
                    for column_spec in column_def.spec.iter() {
                        if let ColumnSpec::AutoIncrement = column_spec {
                            continue;
                        }
                        write!(sql, ", ").unwrap();
                        write!(sql, "ALTER COLUMN ").unwrap();
                        column_def.name.prepare(sql, self.quote());
                        write!(sql, " SET ").unwrap();
                        self.prepare_column_spec(column_spec, sql);
                    }
                }
                TableAlterOption::RenameColumn(from_name, to_name) => {
                    write!(sql, "RENAME COLUMN ").unwrap();
                    from_name.prepare(sql, self.quote());
                    write!(sql, " TO ").unwrap();
                    to_name.prepare(sql, self.quote());
                }
                TableAlterOption::DropColumn(column_name) => {
                    write!(sql, "DROP COLUMN ").unwrap();
                    column_name.prepare(sql, self.quote());
                }
            }
            false
        });
    }

    fn prepare_table_rename_statement(&self, rename: &TableRenameStatement, sql: &mut SqlWriter) {
        write!(sql, "ALTER TABLE ").unwrap();
        if let Some(from_name) = &rename.from_name {
            from_name.prepare(sql, self.quote());
        }
        write!(sql, " RENAME TO ").unwrap();
        if let Some(to_name) = &rename.to_name {
            to_name.prepare(sql, self.quote());
        }
    }
}

impl OracleQueryBuilder {
    fn prepare_column_type_check_auto_increment(
        &self,
        column_def: &ColumnDef,
        sql: &mut SqlWriter,
    ) {
        if let Some(column_type) = &column_def.types {
            write!(sql, " ").unwrap();
            let is_auto_increment = column_def
                .spec
                .iter()
                .position(|s| matches!(s, ColumnSpec::AutoIncrement));
            if is_auto_increment.is_some() {
                match &column_type {
                    ColumnType::SmallInteger(_) => write!(sql, "smallserial").unwrap(),
                    ColumnType::Integer(_) => write!(sql, "serial").unwrap(),
                    ColumnType::BigInteger(_) => write!(sql, "bigserial").unwrap(),
                    _ => unimplemented!(),
                }
            } else {
                self.prepare_column_type(column_type, sql);
            }
        }
    }
}
