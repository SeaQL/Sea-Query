use crate::*;

pub trait TriggerRefBuilder: QuotedBuilder {
    /// Translate [`TriggerRef`] that without values into SQL statement.
    fn prepare_trigger_ref_iden(&self, table_ref: &TriggerRef, sql: &mut dyn SqlWriter) {
        match table_ref {
            TriggerRef::Trigger(iden) => {
                iden.prepare(sql.as_writer(), self.quote());
            }
        }
    }
}
