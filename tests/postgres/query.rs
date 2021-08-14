use super::*;

#[test]
fn select_1() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .limit(10)
            .offset(100)
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" LIMIT 10 OFFSET 100"#
    );
}

#[test]
fn select_2() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .and_where(Expr::col(Char::SizeW).eq(3))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3"#
    );
}

#[test]
fn select_3() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .and_where(Expr::col(Char::SizeW).eq(3))
            .and_where(Expr::col(Char::SizeH).eq(4))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 AND "size_h" = 4"#
    );
}

#[test]
fn select_4() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect])
            .from_subquery(
                Query::select()
                    .columns(vec![Glyph::Image, Glyph::Aspect])
                    .from(Glyph::Table)
                    .take(),
                Alias::new("subglyph")
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM (SELECT "image", "aspect" FROM "glyph") AS "subglyph""#
    );
}

#[test]
fn select_5() {
    assert_eq!(
        Query::select()
            .column((Glyph::Table, Glyph::Image))
            .from(Glyph::Table)
            .and_where(Expr::tbl(Glyph::Table, Glyph::Aspect).is_in(vec![3, 4]))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "glyph"."image" FROM "glyph" WHERE "glyph"."aspect" IN (3, 4)"#
    );
}

#[test]
fn select_6() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .exprs(vec![Expr::col(Glyph::Image).max(),])
            .from(Glyph::Table)
            .group_by_columns(vec![Glyph::Aspect,])
            .and_having(Expr::col(Glyph::Aspect).gt(2))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect" HAVING "aspect" > 2"#
    );
}

#[test]
fn select_7() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .from(Glyph::Table)
            .and_where(Expr::expr(Expr::col(Glyph::Aspect).if_null(0)).gt(2))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2"#
    );
}

#[test]
fn select_8() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character,])
            .from(Char::Table)
            .left_join(
                Font::Table,
                Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id)
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id""#
    );
}

#[test]
fn select_9() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character,])
            .from(Char::Table)
            .left_join(
                Font::Table,
                Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id)
            )
            .inner_join(
                Glyph::Table,
                Expr::tbl(Char::Table, Char::Character).equals(Glyph::Table, Glyph::Image)
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" LEFT JOIN "font" ON "character"."font_id" = "font"."id" INNER JOIN "glyph" ON "character"."character" = "glyph"."image""#
    );
}

#[test]
fn select_10() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character,])
            .from(Char::Table)
            .left_join(
                Font::Table,
                Expr::tbl(Char::Table, Char::FontId)
                    .equals(Font::Table, Font::Id)
                    .and(Expr::tbl(Char::Table, Char::FontId).equals(Font::Table, Font::Id))
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" LEFT JOIN "font" ON ("character"."font_id" = "font"."id") AND ("character"."font_id" = "font"."id")"#
    );
}

#[test]
fn select_11() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .from(Glyph::Table)
            .and_where(Expr::expr(Expr::col(Glyph::Aspect).if_null(0)).gt(2))
            .order_by(Glyph::Image, Order::Desc)
            .order_by((Glyph::Table, Glyph::Aspect), Order::Asc)
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "image" DESC, "glyph"."aspect" ASC"#
    );
}

#[test]
fn select_12() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .from(Glyph::Table)
            .and_where(Expr::expr(Expr::col(Glyph::Aspect).if_null(0)).gt(2))
            .order_by_columns(vec![(Glyph::Id, Order::Asc), (Glyph::Aspect, Order::Desc),])
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "id" ASC, "aspect" DESC"#
    );
}

#[test]
fn select_13() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .from(Glyph::Table)
            .and_where(Expr::expr(Expr::col(Glyph::Aspect).if_null(0)).gt(2))
            .order_by_columns(vec![
                ((Glyph::Table, Glyph::Id), Order::Asc),
                ((Glyph::Table, Glyph::Aspect), Order::Desc),
            ])
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM "glyph" WHERE COALESCE("aspect", 0) > 2 ORDER BY "glyph"."id" ASC, "glyph"."aspect" DESC"#
    );
}

#[test]
fn select_14() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Id, Glyph::Aspect,])
            .expr(Expr::col(Glyph::Image).max())
            .from(Glyph::Table)
            .group_by_columns(vec![
                (Glyph::Table, Glyph::Id),
                (Glyph::Table, Glyph::Aspect),
            ])
            .and_having(Expr::col(Glyph::Aspect).gt(2))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "id", "aspect", MAX("image") FROM "glyph" GROUP BY "glyph"."id", "glyph"."aspect" HAVING "aspect" > 2"#
    );
}

#[test]
fn select_15() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character])
            .from(Char::Table)
            .and_where(Expr::col(Char::FontId).is_null())
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "font_id" IS NULL"#
    );
}

#[test]
fn select_16() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character])
            .from(Char::Table)
            .and_where(Expr::col(Char::FontId).is_null())
            .and_where(Expr::col(Char::Character).is_not_null())
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "font_id" IS NULL AND "character" IS NOT NULL"#
    );
}

#[test]
fn select_17() {
    assert_eq!(
        Query::select()
            .columns(vec![(Glyph::Table, Glyph::Image),])
            .from(Glyph::Table)
            .and_where(Expr::tbl(Glyph::Table, Glyph::Aspect).between(3, 5))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "glyph"."image" FROM "glyph" WHERE "glyph"."aspect" BETWEEN 3 AND 5"#
    );
}

#[test]
fn select_18() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect,])
            .from(Glyph::Table)
            .and_where(Expr::col(Glyph::Aspect).between(3, 5))
            .and_where(Expr::col(Glyph::Aspect).not_between(8, 10))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect" FROM "glyph" WHERE ("aspect" BETWEEN 3 AND 5) AND ("aspect" NOT BETWEEN 8 AND 10)"#
    );
}

#[test]
fn select_19() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character])
            .from(Char::Table)
            .and_where(Expr::col(Char::Character).eq("A"))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "character" = 'A'"#
    );
}

#[test]
fn select_20() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .and_where(Expr::col(Char::Character).like("A"))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "character" LIKE 'A'"#
    );
}

#[test]
fn select_21() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character])
            .from(Char::Table)
            .or_where(Expr::col(Char::Character).like("A%"))
            .or_where(Expr::col(Char::Character).like("%B"))
            .or_where(Expr::col(Char::Character).like("%C%"))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "character" LIKE 'A%' OR "character" LIKE '%B' OR "character" LIKE '%C%'"#
    );
}

#[test]
fn select_22() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .cond_where(
                Cond::all()
                    .add(
                        Cond::any().add(Expr::col(Char::Character).like("C")).add(
                            Expr::col(Char::Character)
                                .like("D")
                                .and(Expr::col(Char::Character).like("E"))
                        )
                    )
                    .add(
                        Expr::col(Char::Character)
                            .like("F")
                            .or(Expr::col(Char::Character).like("G"))
                    )
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE ("character" LIKE 'C' OR (("character" LIKE 'D') AND ("character" LIKE 'E'))) AND (("character" LIKE 'F') OR ("character" LIKE 'G'))"#
    );
}

#[test]
fn select_23() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .and_where_option(None)
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character""#
    );
}

#[test]
fn select_24() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .conditions(
                true,
                |x| {
                    x.and_where(Expr::col(Char::FontId).eq(5));
                },
                |_| ()
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "font_id" = 5"#
    );
}

#[test]
fn select_25() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .and_where(
                Expr::col(Char::SizeW)
                    .mul(2)
                    .equals(Expr::col(Char::SizeH).div(2))
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "size_w" * 2 = "size_h" / 2"#
    );
}

#[test]
fn select_26() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .and_where(
                Expr::expr(Expr::col(Char::SizeW).add(1))
                    .mul(2)
                    .equals(Expr::expr(Expr::col(Char::SizeH).div(2)).sub(1))
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE ("size_w" + 1) * 2 = ("size_h" / 2) - 1"#
    );
}

#[test]
fn select_27() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .and_where(Expr::col(Char::SizeW).eq(3))
            .and_where(Expr::col(Char::SizeH).eq(4))
            .and_where(Expr::col(Char::SizeH).eq(5))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 AND "size_h" = 4 AND "size_h" = 5"#
    );
}

#[test]
fn select_28() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .or_where(Expr::col(Char::SizeW).eq(3))
            .or_where(Expr::col(Char::SizeH).eq(4))
            .or_where(Expr::col(Char::SizeH).eq(5))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 OR "size_h" = 4 OR "size_h" = 5"#
    );
}

#[test]
#[should_panic]
fn select_29() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .and_where(Expr::col(Char::SizeW).eq(3))
            .or_where(Expr::col(Char::SizeH).eq(4))
            .and_where(Expr::col(Char::SizeH).eq(5))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE "size_w" = 3 OR "size_h" = 4 AND "size_h" = 5"#
    );
}

#[test]
fn select_30() {
    assert_eq!(
        Query::select()
            .columns(vec![Char::Character, Char::SizeW, Char::SizeH])
            .from(Char::Table)
            .and_where(
                Expr::col(Char::SizeW)
                    .mul(2)
                    .add(Expr::col(Char::SizeH).div(3))
                    .equals(Expr::value(4))
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character", "size_w", "size_h" FROM "character" WHERE ("size_w" * 2) + ("size_h" / 3) = 4"#
    );
}

#[test]
fn select_31() {
    assert_eq!(
        Query::select()
            .expr((1..10_i32).fold(Expr::value(0), |expr, i| { expr.add(Expr::value(i)) }))
            .to_string(PostgresQueryBuilder),
        r#"SELECT 0 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9"#
    );
}

#[test]
fn select_32() {
    assert_eq!(
        Query::select()
            .expr_as(Expr::col(Char::Character), Alias::new("C"))
            .from(Char::Table)
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" AS "C" FROM "character""#
    );
}

#[test]
fn select_33() {
    assert_eq!(
        Query::select()
            .column(Glyph::Image)
            .from(Glyph::Table)
            .and_where(
                Expr::col(Glyph::Aspect)
                    .in_subquery(Query::select().expr(Expr::cust("3 + 2 * 2")).take())
            )
            .to_string(PostgresQueryBuilder),
        r#"SELECT "image" FROM "glyph" WHERE "aspect" IN (SELECT 3 + 2 * 2)"#
    );
}

#[test]
fn select_34a() {
    assert_eq!(
        Query::select()
            .column(Glyph::Aspect)
            .expr(Expr::col(Glyph::Image).max())
            .from(Glyph::Table)
            .group_by_columns(vec![Glyph::Aspect,])
            .or_having(
                Expr::col(Glyph::Aspect)
                    .gt(2)
                    .or(Expr::col(Glyph::Aspect).lt(8))
            )
            .or_having(
                Expr::col(Glyph::Aspect)
                    .gt(12)
                    .and(Expr::col(Glyph::Aspect).lt(18))
            )
            .or_having(Expr::col(Glyph::Aspect).gt(32))
            .to_string(PostgresQueryBuilder),
        vec![
            r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect""#,
            r#"HAVING (("aspect" > 2) OR ("aspect" < 8))"#,
            r#"OR (("aspect" > 12) AND ("aspect" < 18))"#,
            r#"OR "aspect" > 32"#,
        ]
        .join(" ")
    );
}

#[test]
#[should_panic]
fn select_34b() {
    assert_eq!(
        Query::select()
            .column(Glyph::Aspect)
            .expr(Expr::col(Glyph::Image).max())
            .from(Glyph::Table)
            .group_by_columns(vec![Glyph::Aspect,])
            .or_having(
                Expr::col(Glyph::Aspect)
                    .gt(2)
                    .or(Expr::col(Glyph::Aspect).lt(8))
            )
            .and_having(
                Expr::col(Glyph::Aspect)
                    .gt(22)
                    .or(Expr::col(Glyph::Aspect).lt(28))
            )
            .to_string(PostgresQueryBuilder),
        vec![
            r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect""#,
            r#"HAVING (("aspect" > 2) OR ("aspect" < 8))"#,
            r#"AND (("aspect" > 22) OR ("aspect" < 28))"#,
        ]
        .join(" ")
    );
}

#[test]
fn select_35() {
    let (statement, values) = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .and_where(Expr::col(Glyph::Aspect).is_null())
        .build(sea_query::PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL"#
    );
    assert_eq!(values.0, vec![]);
}

#[test]
fn select_36() {
    let (statement, values) = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(Cond::any().add(Expr::col(Glyph::Aspect).is_null()))
        .build(sea_query::PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL"#
    );
    assert_eq!(values.0, vec![]);
}

#[test]
fn select_37() {
    let (statement, values) = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(Cond::any().add(Cond::all()).add(Cond::any()))
        .build(sea_query::PostgresQueryBuilder);

    assert_eq!(statement, r#"SELECT "id" FROM "glyph""#);
    assert_eq!(values.0, vec![]);
}

#[test]
fn select_38() {
    let (statement, values) = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(
            Cond::any()
                .add(Expr::col(Glyph::Aspect).is_null())
                .add(Expr::col(Glyph::Aspect).is_not_null()),
        )
        .build(sea_query::PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL OR "aspect" IS NOT NULL"#
    );
    assert_eq!(values.0, vec![]);
}

#[test]
fn select_39() {
    let (statement, values) = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(
            Cond::all()
                .add(Expr::col(Glyph::Aspect).is_null())
                .add(Expr::col(Glyph::Aspect).is_not_null()),
        )
        .build(sea_query::PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL AND "aspect" IS NOT NULL"#
    );
    assert_eq!(values.0, vec![]);
}

#[test]
fn select_40() {
    let statement = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(any![
            Expr::col(Glyph::Aspect).is_null(),
            all![
                Expr::col(Glyph::Aspect).is_not_null(),
                Expr::col(Glyph::Aspect).lt(8)
            ]
        ])
        .to_string(sea_query::PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" IS NULL OR ("aspect" IS NOT NULL AND "aspect" < 8)"#
    );
}

#[test]
fn select_41() {
    assert_eq!(
        Query::select()
            .columns(vec![Glyph::Aspect])
            .exprs(vec![Expr::col(Glyph::Image).max()])
            .from(Glyph::Table)
            .group_by_columns(vec![Glyph::Aspect])
            .cond_having(any![Expr::col(Glyph::Aspect).gt(2)])
            .to_string(PostgresQueryBuilder),
        r#"SELECT "aspect", MAX("image") FROM "glyph" GROUP BY "aspect" HAVING "aspect" > 2"#
    );
}

#[test]
fn select_42() {
    let statement = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(
            Cond::all()
                .add_option(Some(Expr::col(Glyph::Aspect).lt(8)))
                .add(Expr::col(Glyph::Aspect).is_not_null()),
        )
        .to_string(PostgresQueryBuilder);

    assert_eq!(
        statement,
        r#"SELECT "id" FROM "glyph" WHERE "aspect" < 8 AND "aspect" IS NOT NULL"#
    );
}

#[test]
fn select_43() {
    let statement = sea_query::Query::select()
        .column(Glyph::Id)
        .from(Glyph::Table)
        .cond_where(Cond::all().add_option::<SimpleExpr>(None))
        .to_string(PostgresQueryBuilder);

    assert_eq!(statement, r#"SELECT "id" FROM "glyph""#);
}

#[test]
fn select_44() {
    assert_eq!(
        Query::select()
            .column(Char::Character)
            .from(Char::Table)
            .and_where(Expr::col(Char::Character).like("A").escape('\\'))
            .to_string(PostgresQueryBuilder),
        r#"SELECT "character" FROM "character" WHERE "character" LIKE 'A' ESCAPE '\'"#
    );
}

#[test]
#[allow(clippy::approx_constant)]
#[cfg(feature = "with-json")]
fn insert_1() {
    assert_eq!(
        Query::insert()
            .into_table(Glyph::Table)
            .json(json!({
                "image": "24B0E11951B03B07F8300FD003983F03F0780060",
                "aspect": 2.1345,
            }))
            .to_string(PostgresQueryBuilder),
        r#"INSERT INTO "glyph" ("aspect", "image") VALUES (2.1345, '24B0E11951B03B07F8300FD003983F03F0780060')"#
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn insert_2() {
    assert_eq!(
        Query::insert()
            .into_table(Glyph::Table)
            .columns(vec![Glyph::Image, Glyph::Aspect,])
            .values_panic(vec![
                "04108048005887010020060000204E0180400400".into(),
                3.1415.into(),
            ])
            .to_string(PostgresQueryBuilder),
        r#"INSERT INTO "glyph" ("image", "aspect") VALUES ('04108048005887010020060000204E0180400400', 3.1415)"#
    );
}

#[test]
#[allow(clippy::approx_constant)]
fn insert_3() {
    assert_eq!(
        Query::insert()
            .into_table(Glyph::Table)
            .columns(vec![Glyph::Image, Glyph::Aspect,])
            .values_panic(vec![
                "04108048005887010020060000204E0180400400".into(),
                3.1415.into(),
            ])
            .values_panic(vec![Value::Null, 2.1345.into(),])
            .to_string(PostgresQueryBuilder),
        r#"INSERT INTO "glyph" ("image", "aspect") VALUES ('04108048005887010020060000204E0180400400', 3.1415), (NULL, 2.1345)"#
    );
}

#[test]
#[cfg(feature = "with-chrono")]
fn insert_4() {
    assert_eq!(
        Query::insert()
            .into_table(Glyph::Table)
            .columns(vec![Glyph::Image])
            .values_panic(vec![chrono::NaiveDateTime::from_timestamp(0, 0).into()])
            .to_string(PostgresQueryBuilder),
        "INSERT INTO \"glyph\" (\"image\") VALUES ('1970-01-01 00:00:00')"
    );
}

#[test]
#[cfg(feature = "with-uuid")]
fn insert_5() {
    assert_eq!(
        Query::insert()
            .into_table(Glyph::Table)
            .columns(vec![Glyph::Image])
            .values_panic(vec![uuid::Uuid::nil().into()])
            .to_string(PostgresQueryBuilder),
        "INSERT INTO \"glyph\" (\"image\") VALUES ('00000000-0000-0000-0000-000000000000')"
    );
}

#[test]
fn update_1() {
    assert_eq!(
        Query::update()
            .table(Glyph::Table)
            .values(vec![
                (Glyph::Aspect, 2.1345.into()),
                (
                    Glyph::Image,
                    "24B0E11951B03B07F8300FD003983F03F0780060".into()
                ),
            ])
            .and_where(Expr::col(Glyph::Id).eq(1))
            .to_string(PostgresQueryBuilder),
        r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '24B0E11951B03B07F8300FD003983F03F0780060' WHERE "id" = 1"#
    );
}

#[test]
#[cfg(feature = "with-json")]
fn update_2() {
    assert_eq!(
        Query::update()
            .table(Glyph::Table)
            .json(json!({
                "aspect": 2.1345,
                "image": "24B0E11951B03B07F8300FD003983F03F0780060",
            }))
            .and_where(Expr::col(Glyph::Id).eq(1))
            .to_string(PostgresQueryBuilder),
        r#"UPDATE "glyph" SET "aspect" = 2.1345, "image" = '24B0E11951B03B07F8300FD003983F03F0780060' WHERE "id" = 1"#
    );
}

#[test]
fn update_3() {
    assert_eq!(
        Query::update()
            .table(Glyph::Table)
            .value_expr(Glyph::Aspect, Expr::cust("60 * 24 * 24"))
            .values(vec![(
                Glyph::Image,
                "24B0E11951B03B07F8300FD003983F03F0780060".into()
            ),])
            .and_where(Expr::col(Glyph::Id).eq(1))
            .to_string(PostgresQueryBuilder),
        r#"UPDATE "glyph" SET "aspect" = 60 * 24 * 24, "image" = '24B0E11951B03B07F8300FD003983F03F0780060' WHERE "id" = 1"#
    );
}

#[test]
fn delete_1() {
    assert_eq!(
        Query::delete()
            .from_table(Glyph::Table)
            .and_where(Expr::col(Glyph::Id).eq(1))
            .to_string(PostgresQueryBuilder),
        r#"DELETE FROM "glyph" WHERE "id" = 1"#
    );
}
