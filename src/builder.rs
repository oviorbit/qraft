use crate::{col::{Columns, IntoColumns}, ident::TableIdent};

#[derive(Debug)]
pub struct Builder {
    query: String,

    distinct: bool,

    maybe_table: Option<TableIdent>,

    /// If the columns is None, we know it's a wildcard.
    columns: Columns,
}

impl Builder {
    pub fn select<T>(&mut self, table: T) -> &mut Self
    where
        T: IntoColumns
    {
        let cols = table.into_columns();
        self
    }

    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }
}
