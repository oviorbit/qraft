use crate::{ident::IntoIdent, Ident};

pub struct InsertBuilder {
    table: Ident,
}

impl InsertBuilder {
    pub fn insert_into<T: IntoIdent>(table: T) -> Self {
        Self {
            table: table.into_ident(),
        }
    }

    pub fn field<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: IntoIdent,
        V: IntoBind
    {
        self
    }
}
