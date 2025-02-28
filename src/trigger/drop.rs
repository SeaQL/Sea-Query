use super::TriggerRef;
use crate::{backend::SchemaBuilder, SchemaStatementBuilder};
use inherent::inherent;

/// Drop a trigger
///
/// # Examples
///
/// ```
/// use sea_query::{tests_cfg::*, *};
///
/// let trigger = NamedTrigger::new("my_trigger")
///     .to_owned();
///
/// let drop_stmt = trigger.drop();
///
/// assert_eq!(
///     drop_stmt.to_string(MysqlQueryBuilder),
///     r#"DROP TRIGGER `my_trigger`"#
/// );
/// assert_eq!(
///     drop_stmt.to_string(PostgresQueryBuilder),
///     r#"DROP TRIGGER "my_trigger""#
/// );
/// assert_eq!(
///     drop_stmt.to_string(SqliteQueryBuilder),
///     r#"DROP TRIGGER "my_trigger""#
/// );
/// ```
///
/// # Trigger names can be derived from table name, action and action time
///
/// ```
/// use sea_query::{tests_cfg::*, *};
///
/// let trigger = UnnamedTrigger::new()
///     .before_insert(Glyph::Table);
///
/// let drop_stmt = trigger.drop();
///
/// assert_eq!(
///     drop_stmt.to_string(MysqlQueryBuilder),
///     r#"DROP TRIGGER `t_glyph_before_insert`"#
/// );
/// assert_eq!(
///     drop_stmt.to_string(PostgresQueryBuilder),
///     r#"DROP TRIGGER "t_glyph_before_insert""#
/// );
/// assert_eq!(
///     drop_stmt.to_string(SqliteQueryBuilder),
///     r#"DROP TRIGGER "t_glyph_before_insert""#
/// );
///
/// ```
#[derive(Debug, Clone)]
pub struct TriggerDropStatement {
    pub(crate) name: TriggerRef,
    pub(crate) if_exists: bool,
}

impl TriggerDropStatement {
    /// Construct drop table statement
    pub fn new(name: TriggerRef) -> Self {
        Self {
            name: name,
            if_exists: false,
        }
    }

    /// Drop table if exists
    pub fn if_exists(&mut self) -> &mut Self {
        self.if_exists = true;
        self
    }

    pub fn take(&mut self) -> Self {
        Self {
            name: std::mem::take(&mut self.name),
            if_exists: self.if_exists,
        }
    }
}

#[inherent]
impl SchemaStatementBuilder for TriggerDropStatement {
    pub fn build<T: SchemaBuilder>(&self, schema_builder: T) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_trigger_drop_statement(self, &mut sql);
        sql
    }

    pub fn build_any(&self, schema_builder: &dyn SchemaBuilder) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_trigger_drop_statement(self, &mut sql);
        sql
    }

    pub fn to_string<T: SchemaBuilder>(&self, schema_builder: T) -> String;
}
