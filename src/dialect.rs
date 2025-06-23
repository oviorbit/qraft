#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dialect {
    Postgres,
    MySql,
    Sqlite,
}

pub trait HasDialect {
    const DIALECT: Dialect;
}

pub struct Postgres;

impl HasDialect for Postgres {
    const DIALECT: Dialect = Dialect::Postgres;
}

pub struct MySql;

impl HasDialect for MySql {
    const DIALECT: Dialect = Dialect::MySql;
}

pub struct Sqlite;

impl HasDialect for Sqlite {
    const DIALECT: Dialect = Dialect::Sqlite;
}

#[cfg(feature = "postgres")]
impl HasDialect for sqlx::Postgres {
    const DIALECT: Dialect = Dialect::Postgres;
}

#[cfg(feature = "mysql")]
impl HasDialect for sqlx::MySql {
    const DIALECT: Dialect = Dialect::Postgres;
}

#[cfg(feature = "sqlite")]
impl HasDialect for sqlx::Sqlite {
    const DIALECT: Dialect = Dialect::Postgres;
}

pub trait HasRowsAffected {
    fn rows_affected(&self) -> usize;
}

#[cfg(feature = "mysql")]
impl HasRowsAffected for sqlx::mysql::MySqlQueryResult {
    fn rows_affected(&self) -> usize {
        self.rows_affected() as usize
    }
}

#[cfg(feature = "postgres")]
impl HasRowsAffected for sqlx::postgres::PgQueryResult {
    fn rows_affected(&self) -> usize {
        self.rows_affected() as usize
    }
}

#[cfg(feature = "sqlite")]
impl HasRowsAffected for sqlx::sqlite::SqliteQueryResult {
    fn rows_affected(&self) -> usize {
        self.rows_affected() as usize
    }
}
