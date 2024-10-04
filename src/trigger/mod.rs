use crate::{Iden, IntoTableRef, SchemaBuilder, SeaRc, TableRef};
use std::fmt;

mod create;
mod drop;

pub use create::*;
pub use drop::*;

pub trait Referencable {
    fn trigger_ref(&self) -> TriggerRef;
    fn trigger_name(&self) -> String;
}
pub trait Droppable: Referencable {
    fn drop(&self) -> TriggerDropStatement {
        TriggerDropStatement::new(self.trigger_ref())
    }
}

pub trait Creatable: Referencable {
    fn create(&self) -> TriggerCreateStatement;
}

pub trait Configurable {
    fn configure(
        &self,
        table: TableRef,
        event: TriggerEvent,
        time: TriggerActionTime,
    ) -> DefinedTrigger;
    fn before_insert<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Insert,
            TriggerActionTime::Before,
        )
    }
    fn after_insert<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Insert,
            TriggerActionTime::After,
        )
    }
    fn before_update<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Update,
            TriggerActionTime::Before,
        )
    }
    fn after_update<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Update,
            TriggerActionTime::After,
        )
    }
    fn before_delete<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Delete,
            TriggerActionTime::Before,
        )
    }
    fn after_delete<T: IntoTableRef>(&self, table: T) -> DefinedTrigger {
        self.configure(
            table.into_table_ref(),
            TriggerEvent::Delete,
            TriggerActionTime::After,
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct NamedTrigger {
    pub(crate) name: TriggerRef
}

impl NamedTrigger {
    pub fn new<T: Into<TriggerRef>>(name: T) -> NamedTrigger {
        Self {
            name: name.into()
        }
    }
}

impl Referencable for NamedTrigger {
    fn trigger_ref(&self) -> TriggerRef {
        self.name.clone()
    }
    fn trigger_name(&self) -> String {
        self.name.to_string()
    }
}

impl Droppable for NamedTrigger {}
impl Configurable for NamedTrigger {
    fn configure(
        &self,
        table: TableRef,
        event: TriggerEvent,
        time: TriggerActionTime,
    ) -> DefinedTrigger {
        DefinedTrigger {
            name: Some(self.name.clone()),
            table: table,
            event: event,
            time: time,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct UnnamedTrigger {
    pub(crate) table: Option<TableRef>,
    pub(crate) event: Option<TriggerEvent>,
    pub(crate) time: Option<TriggerActionTime>,
}

impl UnnamedTrigger {
    pub fn new() -> UnnamedTrigger {
        Self {
            table: None,
            event: None,
            time: None,
        }
    }
    // an unnamed trigger can become a named one
    pub fn name<T: Into<TriggerRef>>(&self, name: T) -> NamedTrigger {
        NamedTrigger {
            name: name.into()
        }
    }
}

impl Configurable for UnnamedTrigger {
    fn configure(
        &self,
        table: TableRef,
        event: TriggerEvent,
        time: TriggerActionTime,
    ) -> DefinedTrigger {
        DefinedTrigger {
            name: None,
            table: table,
            event: event,
            time: time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefinedTrigger {
    pub(crate) name: Option<TriggerRef>,
    pub(crate) table: TableRef,
    pub(crate) event: TriggerEvent,
    pub(crate) time: TriggerActionTime,
}

impl Referencable for DefinedTrigger {
    fn trigger_ref(&self) -> TriggerRef {
        match &self.name {
            Some(name) => name.clone(),
            _ => TriggerRef {
                name: self.trigger_name(),
            },
        }
    }
    fn trigger_name(&self) -> String {
        match &self.name {
            Some(name) => name.to_string(),
            _ => format!(
                "t_{}_{}_{}",
                self.table.to_string().to_lowercase(),
                self.time.to_string().to_lowercase(),
                self.event.to_string().to_lowercase(),
            ),
        }
    }
}

impl Creatable for DefinedTrigger {
    fn create(&self) -> TriggerCreateStatement {
        TriggerCreateStatement {
            trigger: self.clone(),
            if_not_exists: false,
        }
    }
}
impl Droppable for DefinedTrigger {}

#[derive(Debug, Clone)]
pub enum Trigger {
    UnnamedTrigger(
        Option<TableRef>,
        Option<TriggerEvent>,
        Option<TriggerActionTime>,
    ),
    NamedTrigger(
        TriggerRef,
        Option<TableRef>,
        Option<TriggerEvent>,
        Option<TriggerActionTime>,
    ),
    DefinedTrigger(
        Option<TriggerRef>,
        TableRef,
        TriggerEvent,
        TriggerActionTime,
    ),
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::UnnamedTrigger(None, None, None)
    }
}

impl Trigger {
    pub fn new() -> Trigger {
        Trigger::UnnamedTrigger(None, None, None)
    }

    pub fn with_name(name: TriggerRef) -> Trigger {
        Trigger::NamedTrigger(name, None, None, None)
    }
}

/// All available types of trigger statement
#[derive(Debug, Clone)]
pub enum TriggerStatement {
    Create(TriggerCreateStatement),
    Drop(TriggerDropStatement),
}

#[derive(Debug, Clone)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
}

impl fmt::Display for TriggerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Insert => "INSERT",
                Self::Update => "UPDATE",
                Self::Delete => "DELETE",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub enum TriggerActionTime {
    Before,
    After,
}

impl fmt::Display for TriggerActionTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Before => "BEFORE",
                Self::After => "AFTER",
            }
        )
    }
}

impl TriggerStatement {
    /// Build corresponding SQL statement for certain database backend and return SQL string
    pub fn build<T: SchemaBuilder>(&self, trigger_builder: T) -> String {
        match self {
            Self::Create(stat) => stat.build(trigger_builder),
            Self::Drop(stat) => stat.build(trigger_builder),
        }
    }

    /// Build corresponding SQL statement for certain database backend and return SQL string
    pub fn build_any(&self, trigger_builder: &dyn SchemaBuilder) -> String {
        match self {
            Self::Create(stat) => stat.build_any(trigger_builder),
            Self::Drop(stat) => stat.build_any(trigger_builder),
        }
    }

    /// Build corresponding SQL statement for certain database backend and return SQL string
    pub fn to_string<T: SchemaBuilder>(&self, trigger_builder: T) -> String {
        match self {
            Self::Create(stat) => stat.to_string(trigger_builder),
            Self::Drop(stat) => stat.to_string(trigger_builder),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TriggerRef {
    name: String,
}

impl Iden for TriggerRef {
    fn unquoted(&self, s: &mut dyn fmt::Write) {
        s.write_str(&self.name).unwrap();
    }
}

impl From<String> for TriggerRef {
    fn from(value: String) -> Self {
        Self { name: value }
    }
}

impl From<&str> for TriggerRef {
    fn from(value: &str) -> Self {
        Self {
            name: value.to_string(),
        }
    }
}

impl Into<TableRef> for TriggerRef {
    fn into(self) -> TableRef {
        TableRef::Table(SeaRc::new(self))
    }
}

impl From<&TriggerRef> for TableRef {
    fn from(value: &TriggerRef) -> Self {
        TableRef::Table(SeaRc::new(value.clone()))
    }
}

impl fmt::Display for TableRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TableRef::Table(iden) => {
                    iden.to_string()
                }
                _ => "bar".to_string(),
            }
        )
    }
}
