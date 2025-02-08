use crate::Iden;

/// PostgreSQL `hstore` extension type.
///
/// `hstore` provides semi-structured data support by storing key/value pairs in a single column.
///
/// See [the Postgres manual, Appendix F, Section 18][PG.F.18]
///
/// [PG.F.18]: https://www.postgresql.org/docs/current/hstore.html
///
/// ### Note: Requires Postgres 8.3+
/// The `hstore` extension was first added in PostgreSQL 8.3.
///
/// # PostgreSQL Reference
/// The following set of SQL statements can be used to create a table with a `hstore` column.
///
/// ```ignore
/// create table users (username varchar primary key, password varchar, additional_data hstore);
/// create index idx_gist on users using gist (additional_data); -- Sets GIST index support.
/// create index idx_gin on users using gin (additional_data);   -- Sets GIN index support.
///
/// insert into users values ('name.surname@email.com', '@super_secret_1', 'department=>IT');
/// -- additional_data contains department => IT.
/// update users set additional_data['equipment_issued'] = null where username = 'name.surname@email.com';
/// -- additional_data now contains equipment_issued => null, department => IT.
///
/// select * from users;
/// select * from users where additional_data['department'] = 'IT';
/// select * from users where additional_data->'department' = 'IT'; -- Alternate form.
/// ```
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PgHstore;

impl Iden for PgHstore {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(s, "hstore").unwrap();
    }
}

impl From<PgHstore> for String {
    fn from(l: PgHstore) -> Self {
        l.to_string()
    }
}
