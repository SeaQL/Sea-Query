use super::DefinedTrigger;
use crate::{backend::SchemaBuilder, SchemaStatementBuilder};
use inherent::inherent;

#[derive(Debug, Clone)]
pub struct TriggerCreateStatement {
    pub(crate) trigger: DefinedTrigger,
    pub(crate) if_not_exists: bool,
}

impl TriggerCreateStatement {
    pub fn new(trigger: DefinedTrigger) -> Self {
        TriggerCreateStatement {
            trigger,
            if_not_exists: false,
        }
    }

    pub fn if_not_exists(&mut self) -> &mut Self {
        self.if_not_exists = true;
        self
    }
}

#[inherent]
impl SchemaStatementBuilder for TriggerCreateStatement {
    pub fn build<T: SchemaBuilder>(&self, schema_builder: T) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_trigger_create_statement(self, &mut sql);
        sql
    }

    pub fn build_any(&self, schema_builder: &dyn SchemaBuilder) -> String {
        let mut sql = String::with_capacity(256);
        schema_builder.prepare_trigger_create_statement(self, &mut sql);
        sql
    }

    pub fn to_string<T: SchemaBuilder>(&self, schema_builder: T) -> String;
}
