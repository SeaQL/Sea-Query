use crate::*;
use std::ops::Deref;

pub trait QueryBuilder: QuotedBuilder + EscapeBuilder + TableRefBuilder {
    /// The type of placeholder the builder uses for values, and whether it is numbered.
    fn placeholder(&self) -> (&str, bool) {
        ("?", false)
    }

    /// Prefix for tuples in VALUES list (e.g. ROW for Mysql)
    fn values_list_tuple_prefix(&self) -> &str {
        ""
    }

    /// Translate [`InsertStatement`] into SQL statement.
    fn prepare_insert_statement(&self, insert: &InsertStatement, sql: &mut dyn SqlWriter) {
        self.prepare_insert(insert.replace, sql);

        if let Some(table) = &insert.table {
            write!(sql, " INTO ").unwrap();
            self.prepare_table_ref(table, sql);
            write!(sql, " ").unwrap();
        }

        if insert.default_values.is_some() && insert.columns.is_empty() && insert.source.is_none() {
            let num_rows = insert.default_values.unwrap();
            self.insert_default_values(num_rows, sql);
        } else {
            write!(sql, "(").unwrap();
            insert.columns.iter().fold(true, |first, col| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                col.prepare(sql.as_writer(), self.quote());
                false
            });
            write!(sql, ")").unwrap();

            if let Some(source) = &insert.source {
                write!(sql, " ").unwrap();
                match source {
                    InsertValueSource::Values(values) => {
                        write!(sql, "VALUES ").unwrap();
                        values.iter().fold(true, |first, row| {
                            if !first {
                                write!(sql, ", ").unwrap()
                            }
                            write!(sql, "(").unwrap();
                            row.iter().fold(true, |first, col| {
                                if !first {
                                    write!(sql, ", ").unwrap()
                                }
                                self.prepare_simple_expr(col, sql);
                                false
                            });
                            write!(sql, ")").unwrap();
                            false
                        });
                    }
                    InsertValueSource::Select(select_query) => {
                        self.prepare_select_statement(select_query.deref(), sql);
                    }
                }
            }
        }

        self.prepare_on_conflict(&insert.on_conflict, sql);

        self.prepare_returning(&insert.returning, sql);
    }

    /// Translate [`SelectStatement`] into SQL statement.
    fn prepare_select_statement(&self, select: &SelectStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "SELECT ").unwrap();

        if let Some(distinct) = &select.distinct {
            self.prepare_select_distinct(distinct, sql);
            write!(sql, " ").unwrap();
        }

        select.selects.iter().fold(true, |first, expr| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            self.prepare_select_expr(expr, sql);
            false
        });

        if !select.from.is_empty() {
            write!(sql, " FROM ").unwrap();
            select.from.iter().fold(true, |first, table_ref| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_table_ref(table_ref, sql);
                false
            });
        }

        if !select.join.is_empty() {
            for expr in select.join.iter() {
                write!(sql, " ").unwrap();
                self.prepare_join_expr(expr, sql);
            }
        }

        self.prepare_condition(&select.r#where, "WHERE", sql);

        if !select.groups.is_empty() {
            write!(sql, " GROUP BY ").unwrap();
            select.groups.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_simple_expr(expr, sql);
                false
            });
        }

        self.prepare_condition(&select.having, "HAVING", sql);

        if !select.unions.is_empty() {
            select.unions.iter().for_each(|(union_type, query)| {
                match union_type {
                    UnionType::Intersect => write!(sql, " INTERSECT ").unwrap(),
                    UnionType::Distinct => write!(sql, " UNION ").unwrap(),
                    UnionType::Except => write!(sql, " EXCEPT ").unwrap(),
                    UnionType::All => write!(sql, " UNION ALL ").unwrap(),
                }
                self.prepare_select_statement(query, sql);
            });
        }

        if !select.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            select.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_order_expr(expr, sql);
                false
            });
        }

        if let Some(limit) = &select.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql);
        }

        if let Some(offset) = &select.offset {
            write!(sql, " OFFSET ").unwrap();
            self.prepare_value(offset, sql);
        }

        if let Some(lock) = &select.lock {
            write!(sql, " ").unwrap();
            self.prepare_select_lock(lock, sql);
        }

        if let Some((name, query)) = &select.window {
            write!(sql, " WINDOW ").unwrap();
            name.prepare(sql.as_writer(), self.quote());
            write!(sql, " AS ").unwrap();
            self.prepare_window_statement(query, sql);
        }
    }

    /// Translate [`UpdateStatement`] into SQL statement.
    fn prepare_update_statement(&self, update: &UpdateStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "UPDATE ").unwrap();

        if let Some(table) = &update.table {
            self.prepare_table_ref(table, sql);
        }

        write!(sql, " SET ").unwrap();

        update.values.iter().fold(true, |first, row| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            let (col, v) = row;
            col.prepare(sql.as_writer(), self.quote());
            write!(sql, " = ").unwrap();
            self.prepare_simple_expr(v, sql);
            false
        });

        self.prepare_condition(&update.r#where, "WHERE", sql);

        if !update.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            update.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_order_expr(expr, sql);
                false
            });
        }

        if let Some(limit) = &update.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql);
        }

        self.prepare_returning(&update.returning, sql);
    }

    /// Translate [`DeleteStatement`] into SQL statement.
    fn prepare_delete_statement(&self, delete: &DeleteStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "DELETE ").unwrap();

        if let Some(table) = &delete.table {
            write!(sql, "FROM ").unwrap();
            self.prepare_table_ref(table, sql);
        }

        self.prepare_condition(&delete.wherei, "WHERE", sql);

        if !delete.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            delete.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_order_expr(expr, sql);
                false
            });
        }

        if let Some(limit) = &delete.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql);
        }

        self.prepare_returning(&delete.returning, sql);
    }

    /// Translate [`SimpleExpr`] into SQL statement.
    fn prepare_simple_expr(&self, simple_expr: &SimpleExpr, sql: &mut dyn SqlWriter) {
        self.prepare_simple_expr_common(simple_expr, sql);
    }

    fn prepare_simple_expr_common(&self, simple_expr: &SimpleExpr, sql: &mut dyn SqlWriter) {
        match simple_expr {
            SimpleExpr::Column(column_ref) => {
                self.prepare_column_ref(column_ref, sql);
            }
            SimpleExpr::Tuple(exprs) => {
                self.prepare_tuple(exprs, sql);
            }
            SimpleExpr::Unary(op, expr) => {
                self.prepare_un_oper(op, sql);
                write!(sql, " ").unwrap();
                self.prepare_simple_expr(expr, sql);
            }
            SimpleExpr::FunctionCall(func, exprs) => {
                self.prepare_function(func, sql);
                self.prepare_tuple(exprs, sql);
            }
            SimpleExpr::Binary(left, op, right) => {
                if *op == BinOper::In && right.is_values() && right.get_values().is_empty() {
                    self.binary_expr(
                        &SimpleExpr::Value(1i32.into()),
                        &BinOper::Equal,
                        &SimpleExpr::Value(2i32.into()),
                        sql,
                    );
                } else if *op == BinOper::NotIn
                    && right.is_values()
                    && right.get_values().is_empty()
                {
                    self.binary_expr(
                        &SimpleExpr::Value(1i32.into()),
                        &BinOper::Equal,
                        &SimpleExpr::Value(1i32.into()),
                        sql,
                    );
                } else {
                    self.binary_expr(left, op, right, sql);
                }
            }
            SimpleExpr::SubQuery(oper, sel) => {
                if let Some(oper) = oper {
                    self.prepare_sub_query_oper(oper, sql);
                }
                write!(sql, "(").unwrap();
                self.prepare_query_statement(sel.deref(), sql);
                write!(sql, ")").unwrap();
            }
            SimpleExpr::Value(val) => {
                self.prepare_value(val, sql);
            }
            SimpleExpr::Values(list) => {
                write!(sql, "(").unwrap();
                list.iter().fold(true, |first, val| {
                    if !first {
                        write!(sql, ", ").unwrap();
                    }
                    self.prepare_value(val, sql);
                    false
                });
                write!(sql, ")").unwrap();
            }
            SimpleExpr::Custom(s) => {
                write!(sql, "{}", s).unwrap();
            }
            SimpleExpr::CustomWithValues(expr, values) => {
                let (placeholder, numbered) = self.placeholder();
                let mut tokenizer = Tokenizer::new(expr).iter().peekable();
                let mut count = 0;
                while let Some(token) = tokenizer.next() {
                    match token {
                        Token::Punctuation(mark) if mark == placeholder => match tokenizer.peek() {
                            Some(Token::Punctuation(mark)) if mark == placeholder => {
                                write!(sql, "{}", mark).unwrap();
                                tokenizer.next();
                            }
                            Some(Token::Unquoted(tok)) if numbered => {
                                if let Ok(num) = tok.parse::<usize>() {
                                    self.prepare_value(&values[num - 1], sql);
                                }
                                tokenizer.next();
                            }
                            _ => {
                                self.prepare_value(&values[count], sql);
                                count += 1;
                            }
                        },
                        _ => write!(sql, "{}", token).unwrap(),
                    };
                }
            }
            SimpleExpr::Keyword(keyword) => {
                self.prepare_keyword(keyword, sql);
            }
            SimpleExpr::AsEnum(_, expr) => {
                self.prepare_simple_expr(expr, sql);
            }
            SimpleExpr::Case(case_stmt) => {
                self.prepare_case_statement(case_stmt, sql);
            }
            SimpleExpr::Constant(val) => {
                self.prepare_constant(val, sql);
            }
        }
    }

    /// Translate [`CaseStatement`] into SQL statement.
    fn prepare_case_statement(&self, stmts: &CaseStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "(CASE").unwrap();

        let CaseStatement { when, r#else } = stmts;

        for case in when.iter() {
            write!(sql, " WHEN (").unwrap();
            self.prepare_condition_where(&case.condition, sql);
            write!(sql, ") THEN ").unwrap();

            self.prepare_simple_expr(&case.result, sql);
        }
        if let Some(r#else) = r#else.clone() {
            write!(sql, " ELSE ").unwrap();
            self.prepare_simple_expr(&r#else, sql);
        }

        write!(sql, " END)").unwrap();
    }

    /// Translate [`SelectDistinct`] into SQL statement.
    fn prepare_select_distinct(&self, select_distinct: &SelectDistinct, sql: &mut dyn SqlWriter) {
        match select_distinct {
            SelectDistinct::All => write!(sql, "ALL").unwrap(),
            SelectDistinct::Distinct => write!(sql, "DISTINCT").unwrap(),
            _ => {}
        }
    }

    /// Translate [`LockType`] into SQL statement.
    fn prepare_select_lock(&self, lock: &LockClause, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "FOR {}",
            match lock.r#type {
                LockType::Update => "UPDATE",
                LockType::NoKeyUpdate => "NO KEY UPDATE",
                LockType::Share => "SHARE",
                LockType::KeyShare => "KEY SHARE",
            }
        )
        .unwrap();
        if !lock.tables.is_empty() {
            write!(sql, " OF ").unwrap();
            lock.tables.iter().fold(true, |first, table_ref| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_table_ref(table_ref, sql);
                false
            });
        }
        if let Some(behavior) = lock.behavior {
            match behavior {
                LockBehavior::Nowait => write!(sql, " NOWAIT").unwrap(),
                LockBehavior::SkipLocked => write!(sql, " SKIP LOCKED").unwrap(),
            }
        }
    }

    /// Translate [`SelectExpr`] into SQL statement.
    fn prepare_select_expr(&self, select_expr: &SelectExpr, sql: &mut dyn SqlWriter) {
        self.prepare_simple_expr(&select_expr.expr, sql);
        match &select_expr.window {
            Some(WindowSelectType::Name(name)) => {
                write!(sql, " OVER ").unwrap();
                name.prepare(sql.as_writer(), self.quote())
            }
            Some(WindowSelectType::Query(window)) => {
                write!(sql, " OVER ").unwrap();
                write!(sql, "( ").unwrap();
                self.prepare_window_statement(window, sql);
                write!(sql, " )").unwrap();
            }
            None => {}
        };

        match &select_expr.alias {
            Some(alias) => {
                write!(sql, " AS ").unwrap();
                alias.prepare(sql.as_writer(), self.quote());
            }
            None => {}
        };
    }

    /// Translate [`JoinExpr`] into SQL statement.
    fn prepare_join_expr(&self, join_expr: &JoinExpr, sql: &mut dyn SqlWriter) {
        self.prepare_join_type(&join_expr.join, sql);
        write!(sql, " ").unwrap();
        self.prepare_join_table_ref(join_expr, sql);
        if let Some(on) = &join_expr.on {
            self.prepare_join_on(on, sql);
        }
    }

    fn prepare_join_table_ref(&self, join_expr: &JoinExpr, sql: &mut dyn SqlWriter) {
        if join_expr.lateral {
            write!(sql, "LATERAL ").unwrap();
        }
        self.prepare_table_ref(&join_expr.table, sql);
    }

    /// Translate [`TableRef`] into SQL statement.
    fn prepare_table_ref(&self, table_ref: &TableRef, sql: &mut dyn SqlWriter) {
        match table_ref {
            TableRef::SubQuery(query, alias) => {
                write!(sql, "(").unwrap();
                self.prepare_select_statement(query, sql);
                write!(sql, ")").unwrap();
                write!(sql, " AS ").unwrap();
                alias.prepare(sql.as_writer(), self.quote());
            }
            TableRef::ValuesList(values, alias) => {
                write!(sql, "(").unwrap();
                self.prepare_values_list(values, sql);
                write!(sql, ")").unwrap();
                write!(sql, " AS ").unwrap();
                alias.prepare(sql.as_writer(), self.quote());
            }
            _ => self.prepare_table_ref_iden(table_ref, sql),
        }
    }

    fn prepare_column_ref(&self, column_ref: &ColumnRef, sql: &mut dyn SqlWriter) {
        match column_ref {
            ColumnRef::Column(column) => column.prepare(sql.as_writer(), self.quote()),
            ColumnRef::TableColumn(table, column) => {
                table.prepare(sql.as_writer(), self.quote());
                write!(sql, ".").unwrap();
                column.prepare(sql.as_writer(), self.quote());
            }
            ColumnRef::SchemaTableColumn(schema, table, column) => {
                schema.prepare(sql.as_writer(), self.quote());
                write!(sql, ".").unwrap();
                table.prepare(sql.as_writer(), self.quote());
                write!(sql, ".").unwrap();
                column.prepare(sql.as_writer(), self.quote());
            }
            ColumnRef::Asterisk => {
                write!(sql, "*").unwrap();
            }
            ColumnRef::TableAsterisk(table) => {
                table.prepare(sql.as_writer(), self.quote());
                write!(sql, ".*").unwrap();
            }
        };
    }

    /// Translate [`UnOper`] into SQL statement.
    fn prepare_un_oper(&self, un_oper: &UnOper, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "{}",
            match un_oper {
                UnOper::Not => "NOT",
            }
        )
        .unwrap();
    }

    fn prepare_bin_oper_common(&self, bin_oper: &BinOper, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "{}",
            match bin_oper {
                BinOper::And => "AND",
                BinOper::Or => "OR",
                BinOper::Like => "LIKE",
                BinOper::NotLike => "NOT LIKE",
                BinOper::Is => "IS",
                BinOper::IsNot => "IS NOT",
                BinOper::In => "IN",
                BinOper::NotIn => "NOT IN",
                BinOper::Between => "BETWEEN",
                BinOper::NotBetween => "NOT BETWEEN",
                BinOper::Equal => "=",
                BinOper::NotEqual => "<>",
                BinOper::SmallerThan => "<",
                BinOper::GreaterThan => ">",
                BinOper::SmallerThanOrEqual => "<=",
                BinOper::GreaterThanOrEqual => ">=",
                BinOper::Add => "+",
                BinOper::Sub => "-",
                BinOper::Mul => "*",
                BinOper::Div => "/",
                BinOper::Mod => "%",
                BinOper::LShift => "<<",
                BinOper::RShift => ">>",
                BinOper::As => "AS",
                BinOper::Escape => "ESCAPE",
                #[allow(unreachable_patterns)]
                _ => unimplemented!(),
            }
        )
        .unwrap();
    }

    /// Translate [`BinOper`] into SQL statement.
    fn prepare_bin_oper(&self, bin_oper: &BinOper, sql: &mut dyn SqlWriter) {
        self.prepare_bin_oper_common(bin_oper, sql);
    }

    /// Translate [`SubQueryOper`] into SQL statement.
    fn prepare_sub_query_oper(&self, oper: &SubQueryOper, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "{}",
            match oper {
                SubQueryOper::Exists => "EXISTS",
                SubQueryOper::Any => "ANY",
                SubQueryOper::Some => "SOME",
                SubQueryOper::All => "ALL",
            }
        )
        .unwrap();
    }

    /// Translate [`LogicalChainOper`] into SQL statement.
    fn prepare_logical_chain_oper(
        &self,
        log_chain_oper: &LogicalChainOper,
        i: usize,
        length: usize,
        sql: &mut dyn SqlWriter,
    ) {
        let (simple_expr, oper) = match log_chain_oper {
            LogicalChainOper::And(simple_expr) => (simple_expr, "AND"),
            LogicalChainOper::Or(simple_expr) => (simple_expr, "OR"),
        };
        if i > 0 {
            write!(sql, " {} ", oper).unwrap();
        }
        let both_binary = match simple_expr {
            SimpleExpr::Binary(_, _, right) => {
                matches!(right.as_ref(), SimpleExpr::Binary(_, _, _))
            }
            _ => false,
        };
        let need_parentheses = length > 1 && both_binary;
        if need_parentheses {
            write!(sql, "(").unwrap();
        }
        self.prepare_simple_expr(simple_expr, sql);
        if need_parentheses {
            write!(sql, ")").unwrap();
        }
    }

    /// Translate [`Function`] into SQL statement.
    fn prepare_function_common(&self, function: &Function, sql: &mut dyn SqlWriter) {
        if let Function::Custom(iden) = function {
            iden.unquoted(sql.as_writer());
        } else {
            write!(
                sql,
                "{}",
                match function {
                    Function::Max => "MAX",
                    Function::Min => "MIN",
                    Function::Sum => "SUM",
                    Function::Avg => "AVG",
                    Function::Abs => "ABS",
                    Function::Coalesce => "COALESCE",
                    Function::Count => "COUNT",
                    Function::IfNull => self.if_null_function(),
                    Function::CharLength => self.char_length_function(),
                    Function::Cast => "CAST",
                    Function::Lower => "LOWER",
                    Function::Upper => "UPPER",
                    Function::Custom(_) => "",
                    Function::Random => self.random_function(),
                    #[cfg(feature = "backend-postgres")]
                    Function::PgFunction(_) => unimplemented!(),
                }
            )
            .unwrap();
        }
    }

    /// Translate [`QueryStatement`] into SQL statement.
    fn prepare_query_statement(&self, query: &SubQueryStatement, sql: &mut dyn SqlWriter);

    fn prepare_with_query(&self, query: &WithQuery, sql: &mut dyn SqlWriter) {
        self.prepare_with_clause(&query.with_clause, sql);
        self.prepare_query_statement(query.query.as_ref().unwrap().deref(), sql);
    }

    fn prepare_with_clause(&self, with_clause: &WithClause, sql: &mut dyn SqlWriter) {
        self.prepare_with_clause_start(with_clause, sql);
        self.prepare_with_clause_common_tables(with_clause, sql);
        if with_clause.recursive {
            self.prepare_with_clause_recursive_options(with_clause, sql);
        }
    }

    fn prepare_with_clause_recursive_options(
        &self,
        with_clause: &WithClause,
        sql: &mut dyn SqlWriter,
    ) {
        if with_clause.recursive {
            if let Some(search) = &with_clause.search {
                write!(
                    sql,
                    "SEARCH {} FIRST BY ",
                    match &search.order.as_ref().unwrap() {
                        SearchOrder::BREADTH => "BREADTH",
                        SearchOrder::DEPTH => "DEPTH",
                    }
                )
                .unwrap();

                self.prepare_simple_expr(&search.expr.as_ref().unwrap().expr, sql);

                write!(sql, " SET ").unwrap();

                search
                    .expr
                    .as_ref()
                    .unwrap()
                    .alias
                    .as_ref()
                    .unwrap()
                    .prepare(sql.as_writer(), self.quote());
                write!(sql, " ").unwrap();
            }
            if let Some(cycle) = &with_clause.cycle {
                write!(sql, "CYCLE ").unwrap();

                self.prepare_simple_expr(cycle.expr.as_ref().unwrap(), sql);

                write!(sql, " SET ").unwrap();

                cycle
                    .set_as
                    .as_ref()
                    .unwrap()
                    .prepare(sql.as_writer(), self.quote());
                write!(sql, " USING ").unwrap();
                cycle
                    .using
                    .as_ref()
                    .unwrap()
                    .prepare(sql.as_writer(), self.quote());
                write!(sql, " ").unwrap();
            }
        }
    }

    fn prepare_with_clause_common_tables(&self, with_clause: &WithClause, sql: &mut dyn SqlWriter) {
        let mut cte_first = true;
        assert_ne!(
            with_clause.cte_expressions.len(),
            0,
            "Cannot build a with query that has no common table expression!"
        );

        if with_clause.recursive {
            assert_eq!(
                with_clause.cte_expressions.len(),
                1,
                "Cannot build a recursive query with more than one common table! \
                A recursive with query must have a single cte inside it that has a union query of \
                two queries!"
            );
        }
        for cte in &with_clause.cte_expressions {
            if !cte_first {
                write!(sql, ", ").unwrap();
            }
            cte_first = false;

            self.prepare_with_query_clause_common_table(cte, sql);
        }
    }

    fn prepare_with_query_clause_common_table(
        &self,
        cte: &CommonTableExpression,
        sql: &mut dyn SqlWriter,
    ) {
        cte.table_name
            .as_ref()
            .unwrap()
            .prepare(sql.as_writer(), self.quote());

        if cte.cols.is_empty() {
            write!(sql, " ").unwrap();
        } else {
            write!(sql, " (").unwrap();

            let mut col_first = true;
            for col in &cte.cols {
                if !col_first {
                    write!(sql, ", ").unwrap();
                }
                col_first = false;
                col.prepare(sql.as_writer(), self.quote());
            }

            write!(sql, ") ").unwrap();
        }

        write!(sql, "AS ").unwrap();

        self.prepare_with_query_clause_materialization(cte, sql);

        write!(sql, "(").unwrap();

        self.prepare_query_statement(cte.query.as_ref().unwrap().deref(), sql);

        write!(sql, ") ").unwrap();
    }

    fn prepare_with_query_clause_materialization(
        &self,
        cte: &CommonTableExpression,
        sql: &mut dyn SqlWriter,
    ) {
        if let Some(materialized) = cte.materialized {
            write!(
                sql,
                "{} MATERIALIZED ",
                if materialized { "" } else { "NOT" }
            )
            .unwrap()
        }
    }

    fn prepare_with_clause_start(&self, with_clause: &WithClause, sql: &mut dyn SqlWriter) {
        write!(sql, "WITH ").unwrap();

        if with_clause.recursive {
            write!(sql, "RECURSIVE ").unwrap();
        }
    }

    fn prepare_insert(&self, replace: bool, sql: &mut dyn SqlWriter) {
        if replace {
            write!(sql, "REPLACE").unwrap();
        } else {
            write!(sql, "INSERT").unwrap();
        }
    }

    fn prepare_function(&self, function: &Function, sql: &mut dyn SqlWriter) {
        self.prepare_function_common(function, sql)
    }

    /// Translate [`JoinType`] into SQL statement.
    fn prepare_join_type(&self, join_type: &JoinType, sql: &mut dyn SqlWriter) {
        write!(
            sql,
            "{}",
            match join_type {
                JoinType::Join => "JOIN",
                JoinType::CrossJoin => "CROSS JOIN",
                JoinType::InnerJoin => "INNER JOIN",
                JoinType::LeftJoin => "LEFT JOIN",
                JoinType::RightJoin => "RIGHT JOIN",
            }
        )
        .unwrap()
    }

    /// Translate [`OrderExpr`] into SQL statement.
    fn prepare_order_expr(&self, order_expr: &OrderExpr, sql: &mut dyn SqlWriter) {
        if !matches!(order_expr.order, Order::Field(_)) {
            self.prepare_simple_expr(&order_expr.expr, sql);
        }
        self.prepare_order(order_expr, sql);
    }

    /// Translate [`JoinOn`] into SQL statement.
    fn prepare_join_on(&self, join_on: &JoinOn, sql: &mut dyn SqlWriter) {
        match join_on {
            JoinOn::Condition(c) => self.prepare_condition(c, "ON", sql),
            JoinOn::Columns(_c) => unimplemented!(),
        }
    }

    /// Translate [`Order`] into SQL statement.
    fn prepare_order(&self, order_expr: &OrderExpr, sql: &mut dyn SqlWriter) {
        match &order_expr.order {
            Order::Asc => write!(sql, "ASC").unwrap(),
            Order::Desc => write!(sql, "DESC").unwrap(),
            Order::Field(values) => self.prepare_field_order(order_expr, values, sql),
        }
    }

    /// Translate [`Order::Field`] into SQL statement
    fn prepare_field_order(
        &self,
        order_expr: &OrderExpr,
        values: &Values,
        sql: &mut dyn SqlWriter,
    ) {
        write!(sql, "CASE ").unwrap();
        let mut i = 0;
        for value in &values.0 {
            write!(sql, "WHEN ").unwrap();
            self.prepare_simple_expr(&order_expr.expr, sql);
            write!(sql, "=").unwrap();
            let value = self.value_to_string(value);
            write!(sql, "{}", value).unwrap();
            write!(sql, " THEN {} ", i).unwrap();
            i += 1;
        }
        write!(sql, "ELSE {} END", i).unwrap();
    }

    /// Write [`Value`] into SQL statement as parameter.
    fn prepare_value(&self, value: &Value, sql: &mut dyn SqlWriter);

    /// Write [`Value`] inline.
    fn prepare_constant(&self, value: &Value, sql: &mut dyn SqlWriter) {
        let string = self.value_to_string(value);
        write!(sql, "{}", string).unwrap();
    }

    /// Translate a `&[ValueTuple]` into a VALUES list.
    fn prepare_values_list(&self, value_tuples: &[ValueTuple], sql: &mut dyn SqlWriter) {
        write!(sql, "VALUES ").unwrap();
        value_tuples.iter().fold(true, |first, value_tuple| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            write!(sql, "{}", self.values_list_tuple_prefix()).unwrap();
            write!(sql, "(").unwrap();
            value_tuple.clone().into_iter().fold(true, |first, value| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_value(&value, sql);
                false
            });

            write!(sql, ")").unwrap();
            false
        });
    }

    /// Translate [`SimpleExpr::Tuple`] into SQL statement.
    fn prepare_tuple(&self, exprs: &[SimpleExpr], sql: &mut dyn SqlWriter) {
        write!(sql, "(").unwrap();
        exprs.iter().fold(true, |first, expr| {
            if !first {
                write!(sql, ", ").unwrap();
            }
            self.prepare_simple_expr(expr, sql);
            false
        });
        write!(sql, ")").unwrap();
    }

    /// Translate [`Keyword`] into SQL statement.
    fn prepare_keyword(&self, keyword: &Keyword, sql: &mut dyn SqlWriter) {
        match keyword {
            Keyword::Null => write!(sql, "NULL").unwrap(),
            Keyword::CurrentDate => write!(sql, "CURRENT_DATE").unwrap(),
            Keyword::CurrentTime => write!(sql, "CURRENT_TIME").unwrap(),
            Keyword::CurrentTimestamp => write!(sql, "CURRENT_TIMESTAMP").unwrap(),
            Keyword::Custom(iden) => iden.unquoted(sql.as_writer()),
        }
    }

    /// Convert a SQL value into syntax-specific string
    fn value_to_string(&self, v: &Value) -> String {
        let mut s = String::new();
        match v {
            Value::Bool(None)
            | Value::TinyInt(None)
            | Value::SmallInt(None)
            | Value::Int(None)
            | Value::BigInt(None)
            | Value::TinyUnsigned(None)
            | Value::SmallUnsigned(None)
            | Value::Unsigned(None)
            | Value::BigUnsigned(None)
            | Value::Float(None)
            | Value::Double(None)
            | Value::String(None)
            | Value::Char(None)
            | Value::Bytes(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-json")]
            Value::Json(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-time")]
            Value::TimeDate(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-time")]
            Value::TimeTime(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-time")]
            Value::TimeDateTime(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-time")]
            Value::TimeDateTimeWithTimeZone(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-bigdecimal")]
            Value::BigDecimal(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-ipnetwork")]
            Value::IpNetwork(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "with-mac_address")]
            Value::MacAddress(None) => write!(s, "NULL").unwrap(),
            #[cfg(feature = "postgres-array")]
            Value::Array(_, None) => write!(s, "NULL").unwrap(),
            Value::Bool(Some(b)) => write!(s, "{}", if *b { "TRUE" } else { "FALSE" }).unwrap(),
            Value::TinyInt(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::SmallInt(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::Int(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::BigInt(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::TinyUnsigned(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::SmallUnsigned(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::Unsigned(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::BigUnsigned(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::Float(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::Double(Some(v)) => write!(s, "{}", v).unwrap(),
            Value::String(Some(v)) => self.write_string_quoted(v, &mut s),
            Value::Char(Some(v)) => {
                self.write_string_quoted(std::str::from_utf8(&[*v as u8]).unwrap(), &mut s)
            }
            Value::Bytes(Some(v)) => write!(
                s,
                "x'{}'",
                v.iter().map(|b| format!("{:02X}", b)).collect::<String>()
            )
            .unwrap(),
            #[cfg(feature = "with-json")]
            Value::Json(Some(v)) => self.write_string_quoted(&v.to_string(), &mut s),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDate(Some(v)) => write!(s, "'{}'", v.format("%Y-%m-%d")).unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoTime(Some(v)) => write!(s, "'{}'", v.format("%H:%M:%S")).unwrap(),
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTime(Some(v)) => {
                write!(s, "'{}'", v.format("%Y-%m-%d %H:%M:%S")).unwrap()
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeUtc(Some(v)) => {
                write!(s, "'{}'", v.format("%Y-%m-%d %H:%M:%S %:z")).unwrap()
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeLocal(Some(v)) => {
                write!(s, "'{}'", v.format("%Y-%m-%d %H:%M:%S %:z")).unwrap()
            }
            #[cfg(feature = "with-chrono")]
            Value::ChronoDateTimeWithTimeZone(Some(v)) => {
                write!(s, "'{}'", v.format("%Y-%m-%d %H:%M:%S %:z")).unwrap()
            }
            #[cfg(feature = "with-time")]
            Value::TimeDate(Some(v)) => {
                write!(s, "'{}'", v.format(time_format::FORMAT_DATE).unwrap()).unwrap()
            }
            #[cfg(feature = "with-time")]
            Value::TimeTime(Some(v)) => {
                write!(s, "'{}'", v.format(time_format::FORMAT_TIME).unwrap()).unwrap()
            }
            #[cfg(feature = "with-time")]
            Value::TimeDateTime(Some(v)) => {
                write!(s, "'{}'", v.format(time_format::FORMAT_DATETIME).unwrap()).unwrap()
            }
            #[cfg(feature = "with-time")]
            Value::TimeDateTimeWithTimeZone(Some(v)) => write!(
                s,
                "'{}'",
                v.format(time_format::FORMAT_DATETIME_TZ).unwrap()
            )
            .unwrap(),
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(Some(v)) => write!(s, "{}", v).unwrap(),
            #[cfg(feature = "with-bigdecimal")]
            Value::BigDecimal(Some(v)) => write!(s, "{}", v).unwrap(),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(Some(v)) => write!(s, "'{}'", v).unwrap(),
            #[cfg(feature = "postgres-array")]
            Value::Array(_, Some(v)) => write!(
                s,
                "'{{{}}}'",
                v.iter()
                    .map(|element| self.value_to_string(element))
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .unwrap(),
            #[cfg(feature = "with-ipnetwork")]
            Value::IpNetwork(Some(v)) => write!(s, "'{}'", v).unwrap(),
            #[cfg(feature = "with-mac_address")]
            Value::MacAddress(Some(v)) => write!(s, "'{}'", v).unwrap(),
            #[cfg(feature = "sea-query-sqlx")]
            Value::CustomSqlx(v) => v.0.format_sql(&mut s).unwrap(),
        };
        s
    }

    #[doc(hidden)]
    /// Write ON CONFLICT expression
    fn prepare_on_conflict(&self, on_conflict: &Option<OnConflict>, sql: &mut dyn SqlWriter) {
        if let Some(on_conflict) = on_conflict {
            self.prepare_on_conflict_keywords(sql);
            self.prepare_on_conflict_target(&on_conflict.target, sql);
            self.prepare_on_conflict_condition(&on_conflict.target_where, sql);
            self.prepare_on_conflict_action(&on_conflict.action, sql);
            self.prepare_on_conflict_condition(&on_conflict.action_where, sql);
        }
    }

    #[doc(hidden)]
    /// Write ON CONFLICT target
    fn prepare_on_conflict_target(
        &self,
        on_conflict_target: &Option<OnConflictTarget>,
        sql: &mut dyn SqlWriter,
    ) {
        if let Some(target) = on_conflict_target {
            match target {
                OnConflictTarget::ConflictColumns(columns) => {
                    write!(sql, "(").unwrap();
                    columns.iter().fold(true, |first, col| {
                        if !first {
                            write!(sql, ", ").unwrap()
                        }
                        col.prepare(sql.as_writer(), self.quote());
                        false
                    });
                    write!(sql, ")").unwrap();
                }
            }
        }
    }

    #[doc(hidden)]
    /// Write ON CONFLICT action
    fn prepare_on_conflict_action(
        &self,
        on_conflict_action: &Option<OnConflictAction>,
        sql: &mut dyn SqlWriter,
    ) {
        if let Some(action) = on_conflict_action {
            match action {
                OnConflictAction::DoNothing => {
                    write!(sql, " DO NOTHING").unwrap();
                }
                OnConflictAction::UpdateColumns(columns) => {
                    self.prepare_on_conflict_do_update_keywords(sql);
                    columns.iter().fold(true, |first, col| {
                        if !first {
                            write!(sql, ", ").unwrap()
                        }
                        col.prepare(sql.as_writer(), self.quote());
                        write!(sql, " = ").unwrap();
                        self.prepare_on_conflict_excluded_table(col, sql);
                        false
                    });
                }
                OnConflictAction::UpdateExprs(column_exprs) => {
                    self.prepare_on_conflict_do_update_keywords(sql);
                    column_exprs.iter().fold(true, |first, (col, expr)| {
                        if !first {
                            write!(sql, ", ").unwrap()
                        }
                        col.prepare(sql.as_writer(), self.quote());
                        write!(sql, " = ").unwrap();
                        self.prepare_simple_expr(expr, sql);
                        false
                    });
                }
            }
        }
    }

    #[doc(hidden)]
    /// Write ON CONFLICT keywords
    fn prepare_on_conflict_keywords(&self, sql: &mut dyn SqlWriter) {
        write!(sql, " ON CONFLICT ").unwrap();
    }

    #[doc(hidden)]
    /// Write ON CONFLICT keywords
    fn prepare_on_conflict_do_update_keywords(&self, sql: &mut dyn SqlWriter) {
        write!(sql, " DO UPDATE SET ").unwrap();
    }

    #[doc(hidden)]
    /// Write ON CONFLICT update action by retrieving value from the excluded table
    fn prepare_on_conflict_excluded_table(&self, col: &DynIden, sql: &mut dyn SqlWriter) {
        write!(sql, "{0}excluded{0}", self.quote()).unwrap();
        write!(sql, ".").unwrap();
        col.prepare(sql.as_writer(), self.quote());
    }

    #[doc(hidden)]
    /// Write ON CONFLICT conditions
    fn prepare_on_conflict_condition(
        &self,
        on_conflict_condition: &ConditionHolder,
        sql: &mut dyn SqlWriter,
    ) {
        self.prepare_condition(on_conflict_condition, "WHERE", sql)
    }

    #[doc(hidden)]
    /// Hook to insert "RETURNING" statements.
    fn prepare_returning(&self, returning: &Option<ReturningClause>, sql: &mut dyn SqlWriter) {
        if let Some(returning) = returning {
            write!(sql, " RETURNING ").unwrap();
            match &returning {
                ReturningClause::All => write!(sql, "*").unwrap(),
                ReturningClause::Columns(cols) => {
                    cols.iter().fold(true, |first, column_ref| {
                        if !first {
                            write!(sql, ", ").unwrap()
                        }
                        self.prepare_column_ref(column_ref, sql);
                        false
                    });
                }
                ReturningClause::Exprs(exprs) => {
                    exprs.iter().fold(true, |first, expr| {
                        if !first {
                            write!(sql, ", ").unwrap()
                        }
                        self.prepare_simple_expr(expr, sql);
                        false
                    });
                }
            }
        }
    }

    #[doc(hidden)]
    /// Translate a condition to a "WHERE" clause.
    fn prepare_condition(
        &self,
        condition: &ConditionHolder,
        keyword: &str,
        sql: &mut dyn SqlWriter,
    ) {
        if !condition.is_empty() {
            write!(sql, " {} ", keyword).unwrap();
        }
        match &condition.contents {
            ConditionHolderContents::Empty => (),
            ConditionHolderContents::Chain(conditions) => {
                for (i, log_chain_oper) in conditions.iter().enumerate() {
                    self.prepare_logical_chain_oper(log_chain_oper, i, conditions.len(), sql);
                }
            }
            ConditionHolderContents::Condition(c) => {
                self.prepare_condition_where(c, sql);
            }
        }
    }

    #[doc(hidden)]
    /// Translate part of a condition to part of a "WHERE" clause.
    fn prepare_condition_where(&self, condition: &Condition, sql: &mut dyn SqlWriter) {
        if condition.negate {
            write!(sql, "NOT (").unwrap();
        }
        let mut is_first = true;
        for cond in &condition.conditions {
            if is_first {
                is_first = false;
            } else {
                match condition.condition_type {
                    ConditionType::Any => write!(sql, " OR ").unwrap(),
                    ConditionType::All => write!(sql, " AND ").unwrap(),
                }
            }
            match cond {
                ConditionExpression::Condition(c) => {
                    if condition.conditions.len() > 1 {
                        write!(sql, "(").unwrap();
                    }
                    self.prepare_condition_where(c, sql);
                    if condition.conditions.len() > 1 {
                        write!(sql, ")").unwrap();
                    }
                }
                ConditionExpression::SimpleExpr(e) => {
                    if condition.conditions.len() > 1 && (e.is_logical() || e.is_between()) {
                        write!(sql, "(").unwrap();
                    }
                    self.prepare_simple_expr(e, sql);
                    if condition.conditions.len() > 1 && (e.is_logical() || e.is_between()) {
                        write!(sql, ")").unwrap();
                    }
                }
            }
        }
        if condition.negate {
            write!(sql, ")").unwrap();
        }
    }

    #[doc(hidden)]
    /// Translate [`Frame`] into SQL statement.
    fn prepare_frame(&self, frame: &Frame, sql: &mut dyn SqlWriter) {
        match *frame {
            Frame::UnboundedPreceding => write!(sql, "UNBOUNDED PRECEDING").unwrap(),
            Frame::Preceding(v) => {
                self.prepare_value(&v.into(), sql);
                write!(sql, "PRECEDING").unwrap();
            }
            Frame::CurrentRow => write!(sql, "CURRENT ROW").unwrap(),
            Frame::Following(v) => {
                self.prepare_value(&v.into(), sql);
                write!(sql, "FOLLOWING").unwrap();
            }
            Frame::UnboundedFollowing => write!(sql, "UNBOUNDED FOLLOWING").unwrap(),
        }
    }

    #[doc(hidden)]
    /// Translate [`WindowStatement`] into SQL statement.
    fn prepare_window_statement(&self, window: &WindowStatement, sql: &mut dyn SqlWriter) {
        if !window.partition_by.is_empty() {
            write!(sql, "PARTITION BY ").unwrap();
            window.partition_by.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_simple_expr(expr, sql);
                false
            });
        }

        if !window.order_by.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            window.order_by.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_order_expr(expr, sql);
                false
            });
        }

        if let Some(frame) = &window.frame {
            match frame.r#type {
                FrameType::Range => write!(sql, " RANGE ").unwrap(),
                FrameType::Rows => write!(sql, " ROWS ").unwrap(),
            };
            if let Some(end) = &frame.end {
                write!(sql, "BETWEEN ").unwrap();
                self.prepare_frame(&frame.start, sql);
                write!(sql, " AND ").unwrap();
                self.prepare_frame(end, sql);
            } else {
                self.prepare_frame(&frame.start, sql);
            }
        }
    }

    #[doc(hidden)]
    /// Translate a binary expr to SQL.
    fn binary_expr(
        &self,
        left: &SimpleExpr,
        op: &BinOper,
        right: &SimpleExpr,
        sql: &mut dyn SqlWriter,
    ) {
        let no_paren = matches!(op, BinOper::Equal | BinOper::NotEqual);
        let left_paren = left.need_parentheses()
            && left.is_binary()
            && *op != left.get_bin_oper().unwrap()
            && !no_paren;
        if left_paren {
            write!(sql, "(").unwrap();
        }
        self.prepare_simple_expr(left, sql);
        if left_paren {
            write!(sql, ")").unwrap();
        }
        write!(sql, " ").unwrap();
        self.prepare_bin_oper(op, sql);
        write!(sql, " ").unwrap();
        let no_right_paren = matches!(
            op,
            BinOper::Between | BinOper::NotBetween | BinOper::Like | BinOper::NotLike
        );
        let right_paren = (right.need_parentheses()
            || right.is_binary() && *op != left.get_bin_oper().unwrap())
            && !no_right_paren
            && !no_paren;
        if right_paren {
            write!(sql, "(").unwrap();
        }
        self.prepare_simple_expr(right, sql);
        if right_paren {
            write!(sql, ")").unwrap();
        }
    }

    #[doc(hidden)]
    /// Write a string surrounded by escaped quotes.
    fn write_string_quoted(&self, string: &str, buffer: &mut String) {
        write!(buffer, "'{}'", self.escape_string(string)).unwrap()
    }

    #[doc(hidden)]
    /// The name of the function that represents the "if null" condition.
    fn if_null_function(&self) -> &str {
        "IFNULL"
    }

    #[doc(hidden)]
    /// The name of the function that returns the char length.
    fn char_length_function(&self) -> &str {
        "CHAR_LENGTH"
    }

    #[doc(hidden)]
    /// The name of the function that returns a random number
    fn random_function(&self) -> &str {
        // Returning it with parens as part of the name because the tuple preparer can't deal with empty lists
        "RANDOM"
    }

    /// The keywords for insert default row.
    fn insert_default_keyword(&self) -> &str {
        "(DEFAULT)"
    }

    /// Write insert default rows expression.
    fn insert_default_values(&self, num_rows: u32, sql: &mut dyn SqlWriter) {
        write!(sql, "VALUES ").unwrap();
        (0..num_rows).fold(true, |first, _| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            write!(sql, "{}", self.insert_default_keyword()).unwrap();
            false
        });
    }
}

impl SubQueryStatement {
    pub(crate) fn prepare_statement(
        &self,
        query_builder: &dyn QueryBuilder,
        sql: &mut dyn SqlWriter,
    ) {
        use SubQueryStatement::*;
        match self {
            SelectStatement(stmt) => query_builder.prepare_select_statement(stmt, sql),
            InsertStatement(stmt) => query_builder.prepare_insert_statement(stmt, sql),
            UpdateStatement(stmt) => query_builder.prepare_update_statement(stmt, sql),
            DeleteStatement(stmt) => query_builder.prepare_delete_statement(stmt, sql),
            WithStatement(stmt) => query_builder.prepare_with_query(stmt, sql),
        }
    }
}

pub(crate) struct CommonSqlQueryBuilder;

impl QueryBuilder for CommonSqlQueryBuilder {
    fn prepare_query_statement(&self, query: &SubQueryStatement, sql: &mut dyn SqlWriter) {
        query.prepare_statement(self, sql);
    }

    fn prepare_value(&self, value: &Value, sql: &mut dyn SqlWriter) {
        sql.push_param(value.clone(), self as _);
    }
}

impl QuotedBuilder for CommonSqlQueryBuilder {
    fn quote(&self) -> char {
        '"'
    }
}

impl EscapeBuilder for CommonSqlQueryBuilder {}

impl TableRefBuilder for CommonSqlQueryBuilder {}
