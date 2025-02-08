use sea_query::{tests_cfg::*, *};

mod foreign_key;
mod index;
#[cfg(feature = "postgres-interval")]
mod interval;
#[allow(deprecated)]
mod query;
mod table;
mod types;
