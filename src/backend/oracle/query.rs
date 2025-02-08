use super::*;
// use crate::extension::postgres::*;

impl QueryBuilder for OracleQueryBuilder {
    fn placeholder(&self) -> (&str, bool) {
        (":", true)
    }

    fn if_null_function(&self) -> &str {
        "COALESCE"
    }

    fn write_string_quoted(&self, string: &str, buffer: &mut String) {
        let escaped = escape_string(string);
        let string = if escaped.find('\\').is_some() {
            "E'".to_owned() + &escaped + "'"
        } else {
            "'".to_owned() + &escaped + "'"
        };
        write!(buffer, "{}", string).unwrap()
    }

    fn prepare_bin_oper(
        &self,
        bin_oper: &BinOper,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match bin_oper {
            // BinOper::Matches => write!(sql, "@@").unwrap(),
            // BinOper::Contains => write!(sql, "@>").unwrap(),
            // BinOper::Contained => write!(sql, "<@").unwrap(),
            // BinOper::Concatenate => write!(sql, "||").unwrap(),
            _ => self.prepare_bin_oper_common(bin_oper, sql, collector),
        }
    }

    fn prepare_function(
        &self,
        function: &Function,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match function {
            // Function::PgFunction(function) => write!(
            //     sql,
            //     "{}",
            //     match function {
            //         PgFunction::ToTsquery => "TO_TSQUERY",
            //         PgFunction::ToTsvector => "TO_TSVECTOR",
            //         PgFunction::PhrasetoTsquery => "PHRASETO_TSQUERY",
            //         PgFunction::PlaintoTsquery => "PLAINTO_TSQUERY",
            //         PgFunction::WebsearchToTsquery => "WEBSEARCH_TO_TSQUERY",
            //         PgFunction::TsRank => "TS_RANK",
            //         PgFunction::TsRankCd => "TS_RANK_CD",
            //         #[cfg(feature = "postgres-array")]
            //         PgFunction::Any => "ANY",
            //         #[cfg(feature = "postgres-array")]
            //         PgFunction::Some => "SOME",
            //         #[cfg(feature = "postgres-array")]
            //         PgFunction::All => "ALL",
            //     }
            // )
            // .unwrap(),
            _ => self.prepare_function_common(function, sql, collector),
        }
    }

    fn prepare_simple_expr(
        &self,
        simple_expr: &SimpleExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match simple_expr {
            SimpleExpr::AsEnum(type_name, expr) => {
                let simple_expr = expr.clone().cast_as(SeaRc::clone(type_name));
                self.prepare_simple_expr_common(&simple_expr, sql, collector);
            }
            _ => QueryBuilder::prepare_simple_expr_common(self, simple_expr, sql, collector),
        }
    }

    fn prepare_order_expr(
        &self,
        order_expr: &OrderExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        if !matches!(order_expr.order, Order::Field(_)) {
            self.prepare_simple_expr(&order_expr.expr, sql, collector);
        }
        write!(sql, " ").unwrap();
        self.prepare_order(order_expr, sql, collector);
        match order_expr.nulls {
            None => (),
            Some(NullOrdering::Last) => write!(sql, " NULLS LAST").unwrap(),
            Some(NullOrdering::First) => write!(sql, " NULLS FIRST").unwrap(),
        }
    }

    fn prepare_query_statement(
        &self,
        query: &SubQueryStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        query.prepare_statement(self, sql, collector);
    }

    /// Translate [`SelectExpr`] into oracle SQL statement.
    fn prepare_select_expr(
        &self,
        select_expr: &SelectExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        self.prepare_simple_expr(&select_expr.expr, sql, collector);
        match &select_expr.window {
            Some(WindowSelectType::Name(name)) => {
                write!(sql, " OVER ").unwrap();
                name.prepare(sql, self.quote())
            }
            Some(WindowSelectType::Query(window)) => {
                write!(sql, " OVER ").unwrap();
                write!(sql, "( ").unwrap();
                self.prepare_window_statement(window, sql, collector);
                write!(sql, " ) ").unwrap();
            }
            None => {}
        };
        // oracle override
        match &select_expr.alias {
            Some(alias) => {
                write!(sql, " ").unwrap();
                alias.prepare(sql, self.quote());
            }
            None => {}
        };
    }

    /// Translate [`SelectStatement`] into oracle SQL statement.
    fn prepare_select_statement(
        &self,
        select: &SelectStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        write!(sql, "SELECT ").unwrap();

        if let Some(distinct) = &select.distinct {
            write!(sql, " ").unwrap();
            self.prepare_select_distinct(distinct, sql, collector);
            write!(sql, " ").unwrap();
        }

        select.selects.iter().fold(true, |first, expr| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            self.prepare_select_expr(expr, sql, collector);
            false
        });

        if !select.from.is_empty() {
            write!(sql, " FROM ").unwrap();
            select.from.iter().fold(true, |first, table_ref| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                backend::query_builder::QueryBuilder::prepare_table_ref(
                    self, table_ref, sql, collector,
                );
                false
            });
        }

        if !select.join.is_empty() {
            for expr in select.join.iter() {
                write!(sql, " ").unwrap();
                self.prepare_join_expr(expr, sql, collector);
            }
        }

        self.prepare_condition(&select.r#where, "WHERE", sql, collector);

        if !select.groups.is_empty() {
            write!(sql, " GROUP BY ").unwrap();
            select.groups.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_simple_expr(expr, sql, collector);
                false
            });
        }

        self.prepare_condition(&select.having, "HAVING", sql, collector);

        if !select.unions.is_empty() {
            select.unions.iter().for_each(|(union_type, query)| {
                match union_type {
                    UnionType::Distinct => write!(sql, " UNION ").unwrap(),
                    UnionType::All => write!(sql, " UNION ALL ").unwrap(),
                }
                self.prepare_select_statement(query, sql, collector);
            });
        }

        if !select.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            select.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_order_expr(expr, sql, collector);
                false
            });
        }

        // oracle override - requires oracle 11c or later!
        if let Some(offset) = &select.offset {
            write!(sql, " OFFSET ").unwrap();
            self.prepare_value(offset, sql, collector);
            write!(sql, " ROWS ").unwrap();
        }

        if let Some(limit) = &select.limit {
            write!(sql, " FETCH NEXT ").unwrap();
            self.prepare_value(limit, sql, collector);
            write!(sql, " ROWS ONLY ").unwrap();
        }

        if let Some(lock) = &select.lock {
            write!(sql, " ").unwrap();
            self.prepare_select_lock(lock, sql, collector);
        }

        if let Some((name, query)) = &select.window {
            write!(sql, " WINDOW ").unwrap();
            name.prepare(sql, self.quote());
            write!(sql, " AS ").unwrap();
            self.prepare_window_statement(query, sql, collector);
        }
    }
    /// Oracle override.
    fn prepare_table_ref_common(
        &self,
        table_ref: &TableRef,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match table_ref {
            TableRef::Table(iden) => {
                iden.prepare(sql, self.quote());
            }
            TableRef::SchemaTable(schema, table) => {
                schema.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                table.prepare(sql, self.quote());
            }
            TableRef::DatabaseSchemaTable(database, schema, table) => {
                database.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                schema.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                table.prepare(sql, self.quote());
            }
            TableRef::TableAlias(iden, alias) => {
                iden.prepare(sql, self.quote());
                write!(sql, " AS ").unwrap();
                alias.prepare(sql, self.quote());
            }
            TableRef::SchemaTableAlias(schema, table, alias) => {
                schema.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                table.prepare(sql, self.quote());
                write!(sql, " AS ").unwrap();
                alias.prepare(sql, self.quote());
            }
            TableRef::DatabaseSchemaTableAlias(database, schema, table, alias) => {
                database.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                schema.prepare(sql, self.quote());
                write!(sql, ".").unwrap();
                table.prepare(sql, self.quote());
                write!(sql, " AS ").unwrap();
                alias.prepare(sql, self.quote());
            }
            TableRef::SubQuery(query, alias) => {
                // oracle override
                write!(sql, "(").unwrap();
                self.prepare_select_statement(query, sql, collector);
                write!(sql, ")").unwrap();
                write!(sql, " ").unwrap();
                alias.prepare(sql, self.quote());
            }
        }
    }

    /// Translate [`JoinType`] into oracle SQL statement.
    fn prepare_join_type(
        &self,
        join_type: &JoinType,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
        write!(
            sql,
            "{}",
            match join_type {
                JoinType::Join => "JOIN",
                JoinType::InnerJoin => "JOIN",
                JoinType::LeftJoin => "LEFT OUTER JOIN",
                JoinType::RightJoin => "RIGHT JOIN",
            }
        )
        .unwrap()
    }
}
