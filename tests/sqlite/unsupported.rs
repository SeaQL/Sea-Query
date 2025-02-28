use super::*;

#[test]
#[should_panic]
#[rustfmt::skip]
fn if_else_statement_is_unsupported() {
    let if_statement = IfElseStatement::new(
        Expr::col(Glyph::Id).eq(1),
        Expr::val("23").into(),
        None
    );
    if_statement.to_string(SqliteQueryBuilder);
}
