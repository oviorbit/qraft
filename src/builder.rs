use crate::ident::TableIdent;

#[derive(Debug)]
pub struct Builder {
    /// The sql query as it is being built.
    query: String,

    /// If the `select` clause should include `distinct`.
    distinct: bool,

    /// The table to select from, if specified.
    ///
    /// This is `None` in contexts where a `from` clause isn’t applicable,
    /// such as when building a subquery.
    maybe_table: Option<TableIdent>,
}

impl Builder {
    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }
}
