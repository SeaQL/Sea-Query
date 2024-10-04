use super::*;
use pretty_assertions::assert_eq;

#[test]
fn unnamed_trigger_can_receive_name() {
    let unnamed_trigger = UnnamedTrigger::new();
    let named_trigger = unnamed_trigger.name("my_trigger");
    assert_eq!(named_trigger.trigger_name().to_string(), "my_trigger");
}

#[test]
fn create_unnamed_trigger() {
    assert_eq!(
        UnnamedTrigger::new()
            .before_insert(Glyph::Table)
            .create()
            .to_string(MysqlQueryBuilder),
        [
            "CREATE TRIGGER `t_glyph_before_insert`",
            "BEFORE INSERT ON `glyph`",
            "FOR EACH ROW\nBEGIN\n\nEND",
        ]
        .join(" ")
    );
}

#[test]
fn create_named_trigger() {
    assert_eq!(
        UnnamedTrigger::new()
            .name("my_trigger")
            .before_insert(Glyph::Table)
            .create()
            .to_string(MysqlQueryBuilder),
        [
            "CREATE TRIGGER `my_trigger`",
            "BEFORE INSERT ON `glyph`",
            "FOR EACH ROW\nBEGIN\n\nEND",
        ]
        .join(" ")
    );
}

#[test]
fn drop_named_trigger() {
    let trigger = NamedTrigger::new("my_trigger");
    assert_eq!(
        trigger.drop().to_string(MysqlQueryBuilder),
        "DROP TRIGGER `my_trigger`"
    );
}

#[test]
fn drop_unnamed_trigger() {
    let trigger = UnnamedTrigger::new().before_delete(Glyph::Table);
    assert_eq!(
        trigger.drop().to_string(MysqlQueryBuilder),
        "DROP TRIGGER `t_glyph_before_delete`"
    );
}
