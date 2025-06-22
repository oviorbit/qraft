use qraft::{col::TableSchema, ident::TableRef, Builder};

#[derive(Debug, Qorm)]
pub struct User {
    username: String,
    email: String,
}

pub trait Query {
    fn query() -> Builder;
}

impl<T> Query for T
where
    T: TableSchema,
{
    fn query() -> Builder {
        Builder::table_schema::<T>()
    }
}

impl TableSchema for User {
    fn table() -> qraft::ident::TableRef {
        TableRef::ident_static("users")
    }
}
