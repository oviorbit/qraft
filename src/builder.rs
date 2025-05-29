use crate::{col::{Columns, IntoColumns}, ident::TableIdent, writer::FormatWriter};

#[derive(Debug, Default)]
pub struct Builder {
    query: String,

    distinct: bool,

    maybe_table: Option<TableIdent>,

    /// If the columns is None, we know it's a wildcard.
    columns: Columns,
}

impl Builder {
    pub fn select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoColumns
    {
        self.columns = cols.into_columns();
        self
    }

    pub fn add_select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoColumns
    {
        let other = cols.into_columns();
        self.columns.append(other);
        self
    }

    pub fn reset_select(&mut self) -> &mut Self {
        self.columns.reset();
        self
    }

    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }
}

impl FormatWriter for Builder {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        context.writer.write_str("select ")?;
        self.columns.format_writer(context)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
