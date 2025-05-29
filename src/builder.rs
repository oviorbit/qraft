use crate::ident::TableIdent;

#[derive(Debug)]
pub struct Builder {
    query: String,

    distinct: bool,

    maybe_table: Option<TableIdent>,

    columns: Option<Vec<TableIdent>>,
}

impl Builder {
    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }
}
