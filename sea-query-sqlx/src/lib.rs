use std::fmt::Debug;

pub trait SqlxValueTrait: Debug + Send + Sync {
    #[cfg(feature = "sqlx-mysql")]
    fn add_mysql(&self, args: &mut sqlx::mysql::MySqlArguments) -> bool;
    #[cfg(feature = "sqlx-postgres")]
    fn add_postgres(&self, args: &mut sqlx::postgres::PgArguments) -> bool;
    #[cfg(feature = "sqlx-sqlite")]
    fn add_sqlite(&self, args: &mut sqlx::sqlite::SqliteArguments) -> bool;

    fn format_sql(&self, s: &mut dyn std::fmt::Write) -> std::fmt::Result;
}
