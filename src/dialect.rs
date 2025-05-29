#[derive(Debug)]
pub enum Dialect {
    Postgres,
    MySql,
    Sqlite
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
