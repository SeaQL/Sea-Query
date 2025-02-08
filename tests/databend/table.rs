use super::*;
use pretty_assertions::assert_eq;

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
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE `glyph` (",
            "`id` int NOT NULL,",
            "`aspect` double NOT NULL,",
            "`image` text",
            ")",
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
                    .auto_increment()
                    .primary_key()
            )
            .col(ColumnDef::new(Font::Name).string().not_null())
            .col(ColumnDef::new(Font::Variant).string_len(255).not_null())
            .col(ColumnDef::new(Font::Language).string_len(1024).not_null())
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE `font` (",
            "`id` int NOT NULL,",
            "`name` varchar NOT NULL,",
            "`variant` varchar(255) NOT NULL,",
            "`language` varchar(1024) NOT NULL",
            ")",
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
                    .auto_increment()
                    .primary_key()
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
            .col(
                ColumnDef::new(Char::CreatedAt)
                    .timestamp()
                    .default(Expr::current_timestamp())
                    .not_null()
            )
            .foreign_key(
                ForeignKey::create()
                    .name("FK_2e303c3a712662f1fc2a4d0aad6")
                    .from(Char::Table, Char::FontId)
                    .to(Font::Table, Font::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Restrict)
            )
            .engine("InnoDB")
            .character_set("utf8mb4")
            .collate("utf8mb4_unicode_ci")
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE IF NOT EXISTS `character` (",
            "`id` int NOT NULL,",
            "`font_size` int NOT NULL,",
            "`character` varchar(255) NOT NULL,",
            "`size_w` int UNSIGNED NOT NULL,",
            "`size_h` int UNSIGNED NOT NULL,",
            "`font_id` int DEFAULT NULL,",
            "`created_at` timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL )",
        ]
        .join(" ")
    );
}

#[test]
fn create_4() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(
                ColumnDef::new(Glyph::Id)
                    .integer()
                    .not_null()
                    .extra("ANYTHING I WANT TO SAY".to_owned())
            )
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE `glyph` (",
            "`id` int NOT NULL ANYTHING I WANT TO SAY",
            ")",
        ]
        .join(" ")
    );
}

#[test]
fn create_5() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Id).integer().not_null())
            .index(Index::create().unique().name("idx-glyph-id").col(Glyph::Id))
            .to_string(DatabendQueryBuilder),
        ["CREATE TABLE `glyph` (", "`id` int NOT NULL", ")",].join(" ")
    );
}

#[test]
fn create_6() {
    assert_eq!(
        Table::create()
            .table(BinaryType::Table)
            .col(ColumnDef::new(BinaryType::BinaryLen).binary_len(32))
            .col(ColumnDef::new(BinaryType::Binary).binary())
            .col(ColumnDef::new(BinaryType::Blob).blob())
            .col(ColumnDef::new(BinaryType::TinyBlob).custom(MySqlType::TinyBlob))
            .col(ColumnDef::new(BinaryType::MediumBlob).custom(MySqlType::MediumBlob))
            .col(ColumnDef::new(BinaryType::LongBlob).custom(MySqlType::LongBlob))
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE `binary_type` (",
            "`binlen` binary(32),",
            "`bin` binary(1),",
            "`b` blob,",
            "`tb` tinyblob,",
            "`mb` mediumblob,",
            "`lb` longblob",
            ")",
        ]
        .join(" ")
    );
}

#[test]
fn create_7() {
    assert_eq!(
        Table::create()
            .table(Char::Table)
            .col(ColumnDef::new(BinaryType::Blob).custom(MySqlType::Blob))
            .col(ColumnDef::new(Char::Character).binary())
            .col(ColumnDef::new(Char::FontSize).binary_len(10))
            .col(ColumnDef::new(Char::SizeW).var_binary(10))
            .to_string(DatabendQueryBuilder),
        [
            "CREATE TABLE `character` (",
            "`b` blob,",
            "`character` binary(1),",
            "`font_size` binary(10),",
            "`size_w` varbinary(10)",
            ")",
        ]
        .join(" ")
    );
}

#[test]
fn create_8() {
    assert_eq!(
        Table::create()
            .table(Font::Table)
            .col(ColumnDef::new(Font::Variant).year())
            .to_string(DatabendQueryBuilder),
        "CREATE TABLE `font` ( `variant` year )"
    );
}

#[test]
fn create_9() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .index(Index::create().name("idx-glyph-id").col(Glyph::Id))
            .to_string(DatabendQueryBuilder),
        "CREATE TABLE `glyph` (  )"
    );
}

#[test]
fn create_10() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(ColumnDef::new(Glyph::Id).enumeration(
                Alias::new("tea"),
                [Alias::new("EverydayTea"), Alias::new("BreakfastTea")]
            ),)
            .to_string(DatabendQueryBuilder),
        "CREATE TABLE `glyph` ( `id` ENUM('EverydayTea', 'BreakfastTea') )"
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        Table::drop()
            .table(Glyph::Table)
            .table(Char::Table)
            .cascade()
            .to_string(DatabendQueryBuilder),
        "DROP TABLE `glyph`, `character` ALL"
    );
}

#[test]
fn truncate_1() {
    assert_eq!(
        Table::truncate()
            .table(Font::Table)
            .to_string(DatabendQueryBuilder),
        "TRUNCATE TABLE `font`"
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
            .to_string(DatabendQueryBuilder),
        "ALTER TABLE `font` ADD COLUMN `new_col` int NOT NULL DEFAULT 100"
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
            .to_string(DatabendQueryBuilder),
        "ALTER TABLE `font` MODIFY COLUMN `new_col` bigint DEFAULT 999"
    );
}

#[test]
fn alter_3() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .rename_column(Alias::new("new_col"), Alias::new("new_column"))
            .to_string(DatabendQueryBuilder),
        "ALTER TABLE `font` RENAME COLUMN `new_col` TO `new_column`"
    );
}

#[test]
fn alter_4() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .drop_column(Alias::new("new_column"))
            .to_string(DatabendQueryBuilder),
        "ALTER TABLE `font` DROP COLUMN `new_column`"
    );
}

#[test]
fn alter_5() {
    assert_eq!(
        Table::rename()
            .table(Font::Table, Alias::new("font_new"))
            .to_string(DatabendQueryBuilder),
        "RENAME TABLE `font` TO `font_new`"
    );
}

#[test]
fn alter_7() {
    assert_eq!(
        Table::alter()
            .table(Font::Table)
            .drop_column(Alias::new("new_column"))
            .rename_column(Font::Name, Alias::new("name_new"))
            .to_string(DatabendQueryBuilder),
        "ALTER TABLE `font` DROP COLUMN `new_column`, RENAME COLUMN `name` TO `name_new`"
    );
}

#[test]
fn create_with_check_constraint() {
    assert_eq!(
        Table::create()
            .table(Glyph::Table)
            .col(
                ColumnDef::new(Glyph::Id)
                    .integer()
                    .not_null()
                    .check(Expr::col(Glyph::Id).gt(10))
            )
            .check(Expr::col(Glyph::Id).lt(20))
            .check(Expr::col(Glyph::Id).ne(15))
            .to_string(DatabendQueryBuilder),
        r#"CREATE TABLE `glyph` ( `id` int NOT NULL )"#,
    );
}

#[test]
fn alter_with_check_constraint() {
    assert_eq!(
        Table::alter()
            .table(Glyph::Table)
            .add_column(
                ColumnDef::new(Glyph::Aspect)
                    .integer()
                    .not_null()
                    .default(101)
                    .check(Expr::col(Glyph::Aspect).gt(100))
            )
            .to_string(DatabendQueryBuilder),
        r#"ALTER TABLE `glyph` ADD COLUMN `aspect` int NOT NULL DEFAULT 101"#,
    );
}
