use crate::*;

pub trait TriggerBuilder: TableRefBuilder + QueryBuilder {
    /// Translate [`TriggerCreateStatement`] into SQL statement.
    fn prepare_trigger_create_statement(
        &self,
        create: &TriggerCreateStatement,
        sql: &mut dyn SqlWriter,
    ) {
        write!(sql, "CREATE TRIGGER ").unwrap();
        self.prepare_create_trigger_if_not_exists(create, sql);

        let trigger_ref = match &create.trigger.name {
            Some(value) => value,
            // auto-generate trigger name
            _ => &create.trigger.trigger_ref(),
        };
        let trigger_ref: TableRef = trigger_ref.into();
        self.prepare_table_ref_iden(&trigger_ref, sql);
        write!(sql, " {} {} ON ", create.trigger.time, create.trigger.event).unwrap();
        self.prepare_table_ref_iden(&create.trigger.table, sql);
        write!(sql, " FOR EACH ROW\nBEGIN\n").unwrap();

        self.prepare_trigger_actions(&create.trigger.actions, sql);

        write!(sql, "END").unwrap();
    }

    fn prepare_trigger_actions(&self, actions: &TriggerActions, sql: &mut dyn SqlWriter) {
        for action in actions {
            self.prepare_simple_expr_common(&action, sql);
            write!(sql, ";\n").unwrap();
        }
    }

    /// Translate IF NOT EXISTS expression in [`TriggerCreateStatement`].
    fn prepare_create_trigger_if_not_exists(
        &self,
        create: &TriggerCreateStatement,
        sql: &mut dyn SqlWriter,
    ) {
        if create.if_not_exists {
            write!(sql, "IF NOT EXISTS ").unwrap();
        }
    }

    // /// Translate [`TriggerRef`] into SQL statement.
    // fn prepare_table_ref(&self, trigger_ref: &TableRef, sql: &mut dyn SqlWriter) {
    //     self.prepare_table_ref_iden(trigger_ref, sql)
    // }

    /// Translate [`TriggerDropStatement`] into SQL statement.
    fn prepare_trigger_drop_statement(&self, drop: &TriggerDropStatement, sql: &mut dyn SqlWriter) {
        write!(sql, "DROP TRIGGER ").unwrap();
        self.prepare_table_ref_iden(&drop.name.clone().into(), sql);
    }

    fn prepare_simple_expr_yeah(&self, simple_expr: &SimpleExpr, sql: &mut dyn SqlWriter) {
        self.prepare_simple_expr_common(simple_expr, sql);
    }
}
