// use sea_query::{extension::postgres::Type, Alias, OracleQueryBuilder};

// use super::*;

// #[test]
// fn create_1() {
//     assert_eq!(
//         Type::create()
//             .as_enum(Font::Table)
//             .values(vec![Font::Name, Font::Variant, Font::Language])
//             .to_string(OracleQueryBuilder),
//         r#"CREATE TYPE "font" AS ENUM ('name', 'variant', 'language')"#
//     );
// }

// #[test]
// fn drop_1() {
//     assert_eq!(
//         Type::drop()
//             .if_exists()
//             .name(Font::Table)
//             .restrict()
//             .to_string(OracleQueryBuilder),
//         r#"DROP TYPE IF EXISTS "font" RESTRICT"#
//     )
// }

// #[test]
// fn drop_2() {
//     assert_eq!(
//         Type::drop().name(Font::Table).to_string(OracleQueryBuilder),
//         r#"DROP TYPE "font""#
//     );
// }

// #[test]
// fn drop_3() {
//     assert_eq!(
//         Type::drop()
//             .if_exists()
//             .name(Font::Table)
//             .cascade()
//             .to_string(OracleQueryBuilder),
//         r#"DROP TYPE IF EXISTS "font" CASCADE"#
//     );
// }

// #[test]
// fn alter_1() {
//     assert_eq!(
//         Type::alter()
//             .name(Font::Table)
//             .add_value(Alias::new("weight"))
//             .to_string(OracleQueryBuilder),
//         r#"ALTER TYPE "font" ADD VALUE 'weight'"#
//     )
// }
// #[test]
// fn alter_2() {
//     assert_eq!(
//         Type::alter()
//             .name(Font::Table)
//             .add_value(Alias::new("weight"))
//             .before(Font::Variant)
//             .to_string(OracleQueryBuilder),
//         r#"ALTER TYPE "font" ADD VALUE 'weight' BEFORE 'variant'"#
//     )
// }

// #[test]
// fn alter_3() {
//     assert_eq!(
//         Type::alter()
//             .name(Font::Table)
//             .add_value(Alias::new("weight"))
//             .after(Font::Variant)
//             .to_string(OracleQueryBuilder),
//         r#"ALTER TYPE "font" ADD VALUE 'weight' AFTER 'variant'"#
//     )
// }

// #[test]
// fn alter_4() {
//     assert_eq!(
//         Type::alter()
//             .name(Font::Table)
//             .rename_to(Alias::new("typeface"))
//             .to_string(OracleQueryBuilder),
//         r#"ALTER TYPE "font" RENAME TO 'typeface'"#
//     )
// }

// #[test]
// fn alter_5() {
//     assert_eq!(
//         Type::alter()
//             .name(Font::Table)
//             .rename_value(Font::Variant, Font::Language)
//             .to_string(OracleQueryBuilder),
//         r#"ALTER TYPE "font" RENAME VALUE 'variant' TO 'language'"#
//     )
// }
