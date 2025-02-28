use super::*;
use pretty_assertions::assert_eq;

#[rustfmt::skip]
#[test]
fn if_without_else() {
    let query = Query::select().column(Asterisk).from(Glyph::Table).take();
    let then = SimpleExpr::SubQuery(None, Box::new(query.into_sub_query_statement()));
    let if_statement = IfElseStatement::new(
        Expr::col(Glyph::Id).eq(1),
        then,
        None
    );
    assert_eq!(
        if_statement.to_string(MysqlQueryBuilder),
        [
            "IF `id` = 1 THEN",
            "(SELECT * FROM `glyph`)",
            "END IF"
        ].join("\n")
    )
}

#[rustfmt::skip]
#[test]
fn if_with_else() {
    let query = Query::select().column(Asterisk).from(Glyph::Table).take();
    let then = SimpleExpr::SubQuery(None, Box::new(query.into_sub_query_statement()));
    let if_statement = IfElseStatement::new(
        Expr::col(Glyph::Id).eq(1),
        then,
        Some(Expr::val("23").into()),
    );
    assert_eq!(
        if_statement.to_string(MysqlQueryBuilder),
        [
            "IF `id` = 1 THEN",
            "(SELECT * FROM `glyph`)",
            "ELSE",
            "'23'",
            "END IF"
        ]
        .join("\n")
    )
}

#[test]
fn if_with_elseif() {
    let query = Query::select().column(Asterisk).from(Glyph::Table).take();
    let then = SimpleExpr::SubQuery(None, Box::new(query.into_sub_query_statement()));
    let if_statement = IfElseStatement::new(
        Expr::col(Glyph::Id).eq(1),
        then,
        Some(SimpleExpr::IfElse(Box::new(IfElseStatement::new(
            Expr::col(Glyph::Id).eq(2),
            Expr::val("42").into(),
            None,
        )))),
    );
    assert_eq!(
        if_statement.to_string(MysqlQueryBuilder),
        [
            "IF `id` = 1 THEN",
            "(SELECT * FROM `glyph`)",
            "ELSEIF `id` = 2 THEN",
            "'42'",
            "END IF"
        ]
        .join("\n")
    )
}

#[test]
fn if_with_elseif_and_else() {
    let query = Query::select().column(Asterisk).from(Glyph::Table).take();
    let then = SimpleExpr::SubQuery(None, Box::new(query.into_sub_query_statement()));
    let if_statement = IfElseStatement::new(
        Expr::col(Glyph::Id).eq(1),
        then,
        Some(SimpleExpr::IfElse(Box::new(IfElseStatement::new(
            Expr::col(Glyph::Id).eq(2),
            Expr::val("42").into(),
            Some(Expr::val("9000").into()),
        )))),
    );
    assert_eq!(
        if_statement.to_string(MysqlQueryBuilder),
        [
            "IF `id` = 1 THEN",
            "(SELECT * FROM `glyph`)",
            "ELSEIF `id` = 2 THEN",
            "'42'",
            "ELSE",
            "'9000'",
            "END IF"
        ]
        .join("\n")
    );
}
