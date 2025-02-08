use super::*;

#[test]
fn create_1() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(
                ColumnDef::new(Glyph::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
            )
            .col(ColumnDef::new(Glyph::Aspect).double().not_null())
            .col(ColumnDef::new(Glyph::Image).text())
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE "glyph" ("#,
            r#""id" serial NOT NULL PRIMARY KEY,"#,
            r#""aspect" double precision NOT NULL,"#,
            r#""image" text"#,
            r#")"#,
        ]
        .join(" ")
    );
}

#[test]
fn create_2() {
    assert_eq!(
        Table::create()
            .table(Font::Table)
            .col(
                ColumnDef::new(Font::Id)
                    .integer()
                    .not_null()
                    .primary_key()
                    .auto_increment()
            )
            .col(ColumnDef::new(Font::Name).string().not_null())
            .col(ColumnDef::new(Font::Variant).string_len(255).not_null())
            .col(ColumnDef::new(Font::Language).string_len(255).not_null())
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE "font" ("#,
            r#""id" serial NOT NULL PRIMARY KEY,"#,
            r#""name" varchar NOT NULL,"#,
            r#""variant" varchar(255) NOT NULL,"#,
            r#""language" varchar(255) NOT NULL"#,
            r#")"#,
        ]
        .join(" ")
    );
}

#[test]
fn create_3() {
    assert_eq!(
        Table::create()
            .table(Char::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Char::Id)
                    .integer()
                    .not_null()
                    .primary_key()
                    .auto_increment()
            )
            .col(ColumnDef::new(Char::FontSize).integer().not_null())
            .col(ColumnDef::new(Char::Character).string_len(255).not_null())
            .col(ColumnDef::new(Char::SizeW).unsigned().not_null())
            .col(ColumnDef::new(Char::SizeH).unsigned().not_null())
            .col(
                ColumnDef::new(Char::FontId)
                    .integer()
                    .default(Value::Int(None))
            )
            .foreign_key(
                ForeignKey::create()
                    .name("FK_2e303c3a712662f1fc2a4d0aad6")
                    .from(Char::Table, Char::FontId)
                    .to(Font::Table, Font::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
            )
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE IF NOT EXISTS "character" ("#,
            r#""id" serial NOT NULL PRIMARY KEY,"#,
            r#""font_size" integer NOT NULL,"#,
            r#""character" varchar(255) NOT NULL,"#,
            r#""size_w" integer NOT NULL,"#,
            r#""size_h" integer NOT NULL,"#,
            r#""font_id" integer DEFAULT NULL,"#,
            r#"CONSTRAINT "FK_2e303c3a712662f1fc2a4d0aad6""#,
            r#"FOREIGN KEY ("font_id") REFERENCES "font" ("id")"#,
            r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
            r#")"#,
        ]
        .join(" ")
    );
}

#[test]
fn create_4() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Image).custom(Glyph::Aspect))
            .to_string(OracleQueryBuilder),
        vec![r#"CREATE TABLE "glyph" ("#, r#""image" aspect"#, r#")"#,].join(" ")
    );
}

#[test]
fn create_5() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Image).json())
            .col(ColumnDef::new(Glyph::Aspect).json_binary())
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE "glyph" ("#,
            r#""image" json,"#,
            r#""aspect" jsonb"#,
            r#")"#,
        ]
        .join(" ")
    );
}

#[test]
fn create_6() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(
                ColumnDef::new(Glyph::Id)
                    .integer()
                    .not_null()
                    .extra("ANYTHING I WANT TO SAY".to_owned())
            )
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE "glyph" ("#,
            r#""id" integer NOT NULL ANYTHING I WANT TO SAY"#,
            r#")"#,
        ]
        .join(" ")
    );
}

// #[test]
// fn create_7() {
//     assert_eq!(
//         Table::create()
//             .table(Glyph::Table)
//             .col(
//                 ColumnDef::new(Glyph::Aspect)
//                     .interval(None, None)
//                     .not_null()
//             )
//             .to_string(OracleQueryBuilder),
//         vec![
//             r#"CREATE TABLE "glyph" ("#,
//             r#""aspect" interval NOT NULL"#,
//             r#")"#,
//         ]
//         .join(" ")
//     );
// }

// #[test]
// fn create_8() {
//     assert_eq!(
//         Table::create()
//             .table(Glyph::Table)
//             .col(
//                 ColumnDef::new(Glyph::Aspect)
//                     .interval(Some(PgInterval::YearToMonth), None)
//                     .not_null()
//             )
//             .to_string(OracleQueryBuilder),
//         vec![
//             r#"CREATE TABLE "glyph" ("#,
//             r#""aspect" interval YEAR TO MONTH NOT NULL"#,
//             r#")"#,
//         ]
//         .join(" ")
//     );
// }

// #[test]
// fn create_9() {
//     assert_eq!(
//         Table::create()
//             .table(Glyph::Table)
//             .col(
//                 ColumnDef::new(Glyph::Aspect)
//                     .interval(None, Some(42))
//                     .not_null()
//             )
//             .to_string(OracleQueryBuilder),
//         vec![
//             r#"CREATE TABLE "glyph" ("#,
//             r#""aspect" interval(42) NOT NULL"#,
//             r#")"#,
//         ]
//         .join(" ")
//     );
// }

// #[test]
// fn create_10() {
//     assert_eq!(
//         Table::create()
//             .table(Glyph::Table)
//             .col(
//                 ColumnDef::new(Glyph::Aspect)
//                     .interval(Some(PgInterval::Hour), Some(43))
//                     .not_null()
//             )
//             .to_string(OracleQueryBuilder),
//         vec![
//             r#"CREATE TABLE "glyph" ("#,
//             r#""aspect" interval HOUR(43) NOT NULL"#,
//             r#")"#,
//         ]
//         .join(" ")
//     );
// }

#[test]
fn create_11() {
    assert_eq!(
        Table::create()
            .table(Char::Table)
            .col(
                ColumnDef::new(Char::CreatedAt)
                    .timestamp_with_time_zone_len(0)
                    .not_null()
            )
            .to_string(OracleQueryBuilder),
        vec![
            r#"CREATE TABLE "character" ("#,
            r#""created_at" timestamp(0) with time zone NOT NULL"#,
            r#")"#,
        ]
        .join(" ")
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        Table::drop()
            .table(Glyph::Table)
            .table(Char::Table)
            .cascade()
            .to_string(OracleQueryBuilder),
        r#"DROP TABLE "glyph", "character" CASCADE"#
    );
}

#[test]
fn truncate_1() {
    assert_eq!(
        Table::truncate()
            .table(Font::Table)
            .to_string(OracleQueryBuilder),
        r#"TRUNCATE TABLE "font""#
    );
}

#[test]
fn alter_1() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .add_column(
                ColumnDef::new(Alias::new("new_col"))
                    .integer()
                    .not_null()
                    .default(100)
            )
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "font" ADD COLUMN "new_col" integer NOT NULL DEFAULT 100"#
    );
}

#[test]
fn alter_2() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .modify_column(
                ColumnDef::new(Alias::new("new_col"))
                    .big_integer()
                    .default(999)
            )
            .to_string(OracleQueryBuilder),
        vec![
            r#"ALTER TABLE "font""#,
            r#"ALTER COLUMN "new_col" TYPE bigint,"#,
            r#"ALTER COLUMN "new_col" SET DEFAULT 999"#,
        ]
        .join(" ")
    );
}

#[test]
fn alter_3() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .rename_column(Alias::new("new_col"), Alias::new("new_column"))
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "font" RENAME COLUMN "new_col" TO "new_column""#
    );
}

#[test]
fn alter_4() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .drop_column(Alias::new("new_column"))
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "font" DROP COLUMN "new_column""#
    );
}

#[test]
fn alter_5() {
    assert_eq!(
        Table::rename()
            .table(Font::Table, Alias::new("font_new"))
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "font" RENAME TO "font_new""#
    );
}

#[test]
#[should_panic(expected = "No alter option found")]
fn alter_6() {
    Table::alter().to_string(OracleQueryBuilder);
}

#[test]
fn alter_7() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .add_column(ColumnDef::new(Alias::new("new_col")).integer())
            .rename_column(Font::Name, Alias::new("name_new"))
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "font" ADD COLUMN "new_col" integer, RENAME COLUMN "name" TO "name_new""#
    );
}
