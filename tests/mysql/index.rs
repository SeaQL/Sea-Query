use super::*;
use pretty_assertions::assert_eq;

#[test]
fn create_1() {
    assert_eq!(
        Index::create()
            .name("idx-glyph-aspect")
            .table(Glyph::Table)
            .col(Glyph::Aspect)
            .to_string(MysqlQueryBuilder),
        "CREATE INDEX `idx-glyph-aspect` ON `glyph` (`aspect`)"
    );
}

#[test]
fn create_2() {
    assert_eq!(
        Index::create()
            .unique()
            .name("idx-glyph-aspect-image")
            .table(Glyph::Table)
            .col(Glyph::Aspect)
            .col(Glyph::Image)
            .to_string(MysqlQueryBuilder),
        "CREATE UNIQUE INDEX `idx-glyph-aspect-image` ON `glyph` (`aspect`, `image`)"
    );
}

#[test]
fn create_3() {
    assert_eq!(
        Index::create()
            .full_text()
            .name("idx-glyph-image")
            .table(Glyph::Table)
            .col(Glyph::Image)
            .to_string(MysqlQueryBuilder),
        "CREATE FULLTEXT INDEX `idx-glyph-image` ON `glyph` (`image`)"
    );
}

#[test]
fn create_4() {
    assert_eq!(
        Index::create()
            .index_type(IndexType::Hash)
            .name("idx-glyph-image")
            .table(Glyph::Table)
            .col(Glyph::Image)
            .to_string(MysqlQueryBuilder),
        "CREATE INDEX `idx-glyph-image` ON `glyph` (`image`) USING HASH"
    );
}

#[test]
fn create_without_name() {
    assert_eq!(
        Index::create()
            .index_type(IndexType::Hash)
            .table(Glyph::Table)
            .col(Glyph::Image)
            .to_string(MysqlQueryBuilder),
        "CREATE INDEX `idx-image` ON `glyph` (`image`) USING HASH"
    );
}

#[test]
fn drop_1() {
    assert_eq!(
        Index::drop()
            .name("idx-glyph-aspect")
            .table(Glyph::Table)
            .to_string(MysqlQueryBuilder),
        "DROP INDEX `idx-glyph-aspect` ON `glyph`"
    );
}
