use crate::*;

pub trait QueryBuilder: QuotedBuilder {
    /// The type of placeholder the builder uses for values, and whether it is numbered.
    fn placeholder(&self) -> (&str, bool) {
        ("?", false)
    }

    /// Translate [`InsertStatement`] into SQL statement.
    fn prepare_insert_statement(
        &self,
        insert: &InsertStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        write!(sql, "INSERT").unwrap();

        if let Some(table) = &insert.table {
            write!(sql, " INTO ").unwrap();
            self.prepare_table_ref(table, sql, collector);
            write!(sql, " ").unwrap();
        }

        write!(sql, "(").unwrap();
        insert.columns.iter().fold(true, |first, col| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            col.prepare(sql, self.quote());
            false
        });
        write!(sql, ")").unwrap();

        write!(sql, " VALUES ").unwrap();
        insert.values.iter().fold(true, |first, row| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            write!(sql, "(").unwrap();
            row.iter().fold(true, |first, col| {
                if !first {
                    write!(sql, ", ").unwrap()
                }
                self.prepare_value(col, sql, collector);
                false
            });
            write!(sql, ")").unwrap();
            false
        });

        self.prepare_returning(&insert.returning, sql, collector);
    }

    /// Translate [`SelectStatement`] into SQL statement.
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

        if let Some(from) = &select.from {
            write!(sql, " FROM ").unwrap();
            self.prepare_table_ref(from, sql, collector);
        }

        if !select.join.is_empty() {
            for expr in select.join.iter() {
                write!(sql, " ").unwrap();
                self.prepare_join_expr(expr, sql, collector);
            }
        }

        self.prepare_condition(&select.wherei, "WHERE", sql, collector);

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

        if let Some(limit) = &select.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql, collector);
        }

        if let Some(offset) = &select.offset {
            write!(sql, " OFFSET ").unwrap();
            self.prepare_value(offset, sql, collector);
        }
    }

    /// Translate [`UpdateStatement`] into SQL statement.
    fn prepare_update_statement(
        &self,
        update: &UpdateStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        write!(sql, "UPDATE ").unwrap();

        if let Some(table) = &update.table {
            self.prepare_table_ref(table, sql, collector);
        }

        write!(sql, " SET ").unwrap();

        update.values.iter().fold(true, |first, row| {
            if !first {
                write!(sql, ", ").unwrap()
            }
            let (k, v) = row;
            write!(sql, "{}{}{} = ", self.quote(), k, self.quote()).unwrap();
            self.prepare_simple_expr(v, sql, collector);
            false
        });

        self.prepare_condition(&update.wherei, "WHERE", sql, collector);

        if !update.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            update.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_order_expr(expr, sql, collector);
                false
            });
        }

        if let Some(limit) = &update.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql, collector);
        }

        self.prepare_returning(&update.returning, sql, collector);
    }

    /// Translate [`DeleteStatement`] into SQL statement.
    fn prepare_delete_statement(
        &self,
        delete: &DeleteStatement,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        write!(sql, "DELETE ").unwrap();

        if let Some(table) = &delete.table {
            write!(sql, "FROM ").unwrap();
            self.prepare_table_ref(table, sql, collector);
        }

        self.prepare_condition(&delete.wherei, "WHERE", sql, collector);

        if !delete.orders.is_empty() {
            write!(sql, " ORDER BY ").unwrap();
            delete.orders.iter().fold(true, |first, expr| {
                if !first {
                    write!(sql, ", ").unwrap();
                }
                self.prepare_order_expr(expr, sql, collector);
                false
            });
        }

        if let Some(limit) = &delete.limit {
            write!(sql, " LIMIT ").unwrap();
            self.prepare_value(limit, sql, collector);
        }
    }

    /// Translate [`SimpleExpr`] into SQL statement.
    fn prepare_simple_expr(
        &self,
        simple_expr: &SimpleExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match simple_expr {
            SimpleExpr::Column(column_ref) => {
                match column_ref {
                    ColumnRef::Column(column) => column.prepare(sql, self.quote()),
                    ColumnRef::TableColumn(table, column) => {
                        table.prepare(sql, self.quote());
                        write!(sql, ".").unwrap();
                        column.prepare(sql, self.quote());
                    }
                };
            }
            SimpleExpr::Unary(op, expr) => {
                self.prepare_un_oper(op, sql, collector);
                write!(sql, " ").unwrap();
                self.prepare_simple_expr(expr, sql, collector);
            }
            SimpleExpr::FunctionCall(func, exprs) => {
                self.prepare_function(func, sql, collector);
                write!(sql, "(").unwrap();
                exprs.iter().fold(true, |first, expr| {
                    if !first {
                        write!(sql, ", ").unwrap();
                    }
                    self.prepare_simple_expr(expr, sql, collector);
                    false
                });
                write!(sql, ")").unwrap();
            }
            SimpleExpr::Binary(left, op, right) => {
                if *op == BinOper::In && right.is_values() && right.get_values().is_empty() {
                    self.binary_expr(
                        &SimpleExpr::Value(1.into()),
                        &BinOper::Equal,
                        &SimpleExpr::Value(2.into()),
                        sql,
                        collector,
                    );
                } else if *op == BinOper::NotIn
                    && right.is_values()
                    && right.get_values().is_empty()
                {
                    self.binary_expr(
                        &SimpleExpr::Value(1.into()),
                        &BinOper::Equal,
                        &SimpleExpr::Value(1.into()),
                        sql,
                        collector,
                    );
                } else {
                    self.binary_expr(left, op, right, sql, collector);
                }
            }
            SimpleExpr::SubQuery(sel) => {
                write!(sql, "(").unwrap();
                self.prepare_select_statement(sel, sql, collector);
                write!(sql, ")").unwrap();
            }
            SimpleExpr::Value(val) => {
                self.prepare_value(val, sql, collector);
            }
            SimpleExpr::Values(list) => {
                write!(sql, "(").unwrap();
                list.iter().fold(true, |first, val| {
                    if !first {
                        write!(sql, ", ").unwrap();
                    }
                    self.prepare_value(val, sql, collector);
                    false
                });
                write!(sql, ")").unwrap();
            }
            SimpleExpr::Custom(s) => {
                write!(sql, "{}", s).unwrap();
            }
            SimpleExpr::CustomWithValues(expr, values) => {
                let mut tokenizer = Tokenizer::new(expr).iter().peekable();
                let mut count = 0;
                while let Some(tok) = tokenizer.next() {
                    match tok {
                        Token::Punctuation(mark) => {
                            if mark == "?" {
                                if let Some(nxt) = tokenizer.peek() {
                                    match nxt {
                                        Token::Punctuation(mark) => {
                                            // escape '??'
                                            if mark == "?" {
                                                write!(sql, "{}", mark).unwrap();
                                                tokenizer.next();
                                                continue;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                self.prepare_value(&values[count], sql, collector);
                                count += 1;
                            } else {
                                write!(sql, "{}", mark).unwrap();
                            }
                        }
                        _ => write!(sql, "{}", tok).unwrap(),
                    }
                }
            }
            SimpleExpr::Keyword(keyword) => {
                self.prepare_keyword(keyword, sql, collector);
            }
        }
    }

    /// Translate [`SelectDistinct`] into SQL statement.
    fn prepare_select_distinct(
        &self,
        select_distinct: &SelectDistinct,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
        write!(
            sql,
            "{}",
            match select_distinct {
                SelectDistinct::All => "ALL",
                SelectDistinct::Distinct => "DISTINCT",
                SelectDistinct::DistinctRow => "DISTINCTROW",
            }
        )
        .unwrap();
    }

    /// Translate [`SelectExpr`] into SQL statement.
    fn prepare_select_expr(
        &self,
        select_expr: &SelectExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        self.prepare_simple_expr(&select_expr.expr, sql, collector);
        match &select_expr.alias {
            Some(alias) => {
                write!(sql, " AS ").unwrap();
                alias.prepare(sql, self.quote());
            }
            None => {}
        }
    }

    /// Translate [`JoinExpr`] into SQL statement.
    fn prepare_join_expr(
        &self,
        join_expr: &JoinExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        self.prepare_join_type(&join_expr.join, sql, collector);
        write!(sql, " ").unwrap();
        self.prepare_table_ref(&join_expr.table, sql, collector);
        if let Some(on) = &join_expr.on {
            write!(sql, " ").unwrap();
            self.prepare_join_on(on, sql, collector);
        }
    }

    /// Translate [`TableRef`] into SQL statement.
    fn prepare_table_ref(
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
            TableRef::SubQuery(query, alias) => {
                write!(sql, "(").unwrap();
                self.prepare_select_statement(query, sql, collector);
                write!(sql, ")").unwrap();
                write!(sql, " AS ").unwrap();
                alias.prepare(sql, self.quote());
            }
        }
    }

    /// Translate [`UnOper`] into SQL statement.
    fn prepare_un_oper(
        &self,
        un_oper: &UnOper,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
        write!(
            sql,
            "{}",
            match un_oper {
                UnOper::Not => "NOT",
            }
        )
        .unwrap();
    }

    /// Translate [`BinOper`] into SQL statement.
    fn prepare_bin_oper(
        &self,
        bin_oper: &BinOper,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
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
                BinOper::Escape => "ESCAPE",
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
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
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
        self.prepare_simple_expr(simple_expr, sql, collector);
        if need_parentheses {
            write!(sql, ")").unwrap();
        }
    }

    /// Translate [`Function`] into SQL statement.
    fn prepare_function(
        &self,
        function: &Function,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
        if let Function::Custom(iden) = function {
            iden.unquoted(sql);
        } else {
            write!(
                sql,
                "{}",
                match function {
                    Function::Max => "MAX",
                    Function::Min => "MIN",
                    Function::Sum => "SUM",
                    Function::Avg => "AVG",
                    Function::Count => "COUNT",
                    Function::IfNull => self.if_null_function(),
                    Function::CharLength => self.char_length_function(),
                    Function::Custom(_) => "",
                }
            )
            .unwrap();
        }
    }

    /// Translate [`JoinType`] into SQL statement.
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
                JoinType::InnerJoin => "INNER JOIN",
                JoinType::LeftJoin => "LEFT JOIN",
                JoinType::RightJoin => "RIGHT JOIN",
            }
        )
        .unwrap()
    }

    /// Translate [`OrderExpr`] into SQL statement.
    fn prepare_order_expr(
        &self,
        order_expr: &OrderExpr,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        self.prepare_simple_expr(&order_expr.expr, sql, collector);
        write!(sql, " ").unwrap();
        self.prepare_order(&order_expr.order, sql, collector);
    }

    /// Translate [`JoinOn`] into SQL statement.
    fn prepare_join_on(
        &self,
        join_on: &JoinOn,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        match join_on {
            JoinOn::Condition(c) => {
                write!(sql, "ON ").unwrap();
                self.prepare_simple_expr(c, sql, collector);
            }
            JoinOn::Columns(_c) => unimplemented!(),
        }
    }

    /// Translate [`Order`] into SQL statement.
    fn prepare_order(&self, order: &Order, sql: &mut SqlWriter, _collector: &mut dyn FnMut(Value)) {
        match order {
            Order::Asc => write!(sql, "ASC").unwrap(),
            Order::Desc => write!(sql, "DESC").unwrap(),
        }
    }

    /// Translate [`Value`] into SQL statement.
    fn prepare_value(&self, value: &Value, sql: &mut SqlWriter, collector: &mut dyn FnMut(Value)) {
        let (placeholder, numbered) = self.placeholder();
        sql.push_param(placeholder, numbered);
        collector(value.clone());
    }

    /// Translate [`Keyword`] into SQL statement.
    fn prepare_keyword(
        &self,
        keyword: &Keyword,
        sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
        if let Keyword::Custom(iden) = keyword {
            iden.unquoted(sql);
        } else {
            write!(
                sql,
                "{}",
                match keyword {
                    Keyword::Null => "NULL",
                    Keyword::Custom(_) => "",
                }
            )
            .unwrap();
        }
    }

    /// Convert a SQL value into syntax-specific string
    fn value_to_string(&self, v: &Value) -> String {
        let mut s = String::new();
        match v {
            Value::Null => write!(s, "NULL").unwrap(),
            Value::Bool(b) => write!(s, "{}", if *b { "TRUE" } else { "FALSE" }).unwrap(),
            Value::TinyInt(v) => write!(s, "{}", v).unwrap(),
            Value::SmallInt(v) => write!(s, "{}", v).unwrap(),
            Value::Int(v) => write!(s, "{}", v).unwrap(),
            Value::BigInt(v) => write!(s, "{}", v).unwrap(),
            Value::TinyUnsigned(v) => write!(s, "{}", v).unwrap(),
            Value::SmallUnsigned(v) => write!(s, "{}", v).unwrap(),
            Value::Unsigned(v) => write!(s, "{}", v).unwrap(),
            Value::BigUnsigned(v) => write!(s, "{}", v).unwrap(),
            Value::Float(v) => write!(s, "{}", v).unwrap(),
            Value::Double(v) => write!(s, "{}", v).unwrap(),
            Value::String(v) => self.write_string_quoted(v, &mut s),
            Value::Bytes(v) => write!(
                s,
                "x\'{}\'",
                v.iter().map(|b| format!("{:02X}", b)).collect::<String>()
            )
            .unwrap(),
            #[cfg(feature = "with-json")]
            Value::Json(v) => self.write_string_quoted(&v.to_string(), &mut s),
            #[cfg(feature = "with-chrono")]
            Value::DateTime(v) => {
                write!(s, "\'{}\'", v.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap()
            }
            #[cfg(feature = "with-chrono")]
            Value::DateTimeWithTimeZone(v) => {
                write!(s, "\'{}\'", v.format("%Y-%m-%d %H:%M:%S %:z").to_string()).unwrap()
            }
            #[cfg(feature = "with-rust_decimal")]
            Value::Decimal(v) => write!(s, "{}", v).unwrap(),
            #[cfg(feature = "with-uuid")]
            Value::Uuid(v) => write!(s, "\'{}\'", v.to_string()).unwrap(),
        };
        s
    }

    #[doc(hidden)]
    /// Hook to insert "RETURNING" statements.
    fn prepare_returning(
        &self,
        _returning: &Vec<SelectExpr>,
        _sql: &mut SqlWriter,
        _collector: &mut dyn FnMut(Value),
    ) {
    }

    #[doc(hidden)]
    /// Translate a condition to a "WHERE" clause.
    fn prepare_condition(
        &self,
        condition: &ConditionHolder,
        keyword: &str,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        if !condition.is_empty() {
            write!(sql, " {} ", keyword).unwrap();
        }
        match &condition.contents {
            ConditionHolderContents::Empty => (),
            ConditionHolderContents::Chain(conditions) => {
                for (i, log_chain_oper) in conditions.iter().enumerate() {
                    self.prepare_logical_chain_oper(
                        log_chain_oper,
                        i,
                        conditions.len(),
                        sql,
                        collector,
                    );
                }
            }
            ConditionHolderContents::Condition(c) => {
                self.prepare_condition_where(&c, sql, collector);
            }
        }
    }

    #[doc(hidden)]
    /// Translate part of a condition to part of a "WHERE" clause.
    fn prepare_condition_where(
        &self,
        condition: &Condition,
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
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
                    self.prepare_condition_where(&c, sql, collector);
                    if condition.conditions.len() > 1 {
                        write!(sql, ")").unwrap();
                    }
                }
                ConditionExpression::SimpleExpr(e) => {
                    if condition.conditions.len() > 1 && (e.is_logical() || e.is_between()) {
                        write!(sql, "(").unwrap();
                    }
                    self.prepare_simple_expr(&e, sql, collector);
                    if condition.conditions.len() > 1 && (e.is_logical() || e.is_between()) {
                        write!(sql, ")").unwrap();
                    }
                }
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
        sql: &mut SqlWriter,
        collector: &mut dyn FnMut(Value),
    ) {
        let no_paren = matches!(op, BinOper::Equal | BinOper::NotEqual | BinOper::Escape);
        let left_paren = left.need_parentheses()
            && left.is_binary()
            && *op != left.get_bin_oper().unwrap()
            && !no_paren;
        if left_paren {
            write!(sql, "(").unwrap();
        }
        self.prepare_simple_expr(left, sql, collector);
        if left_paren {
            write!(sql, ")").unwrap();
        }
        write!(sql, " ").unwrap();
        self.prepare_bin_oper(op, sql, collector);
        write!(sql, " ").unwrap();
        let no_right_paren = matches!(op, BinOper::Between | BinOper::NotBetween);
        let right_paren = (right.need_parentheses()
            || right.is_binary() && *op != left.get_bin_oper().unwrap())
            && !no_right_paren
            && !no_paren;
        if right_paren {
            write!(sql, "(").unwrap();
        }
        self.prepare_simple_expr(right, sql, collector);
        if right_paren {
            write!(sql, ")").unwrap();
        }
    }

    #[doc(hidden)]
    /// Write a string surrounded by escaped quotes.
    fn write_string_quoted(&self, string: &str, buffer: &mut String) {
        write!(buffer, "\'{}\'", escape_string(string)).unwrap()
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
}
