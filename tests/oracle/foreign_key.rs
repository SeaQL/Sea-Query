use super::*;

#[test]
fn create_1() {
    assert_eq!(
        ForeignKey::create()
            .name("FK_2e303c3a712662f1fc2a4d0aad6")
            .from(Char::Table, Char::FontId)
            .to(Font::Table, Font::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)
            .to_string(OracleQueryBuilder),
        vec![
            r#"ALTER TABLE "character" ADD CONSTRAINT "FK_2e303c3a712662f1fc2a4d0aad6""#,
            r#"FOREIGN KEY ("font_id") REFERENCES "font" ("id")"#,
            r#"ON DELETE CASCADE ON UPDATE CASCADE"#,
        ]
        .join(" ")
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        ForeignKey::drop()
            .name("FK_2e303c3a712662f1fc2a4d0aad6")
            .table(Char::Table)
            .to_string(OracleQueryBuilder),
        r#"ALTER TABLE "character" DROP CONSTRAINT "FK_2e303c3a712662f1fc2a4d0aad6""#
    );
}
