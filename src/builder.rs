use crate::{col::{Columns, ColumnsIdent, IntoColumns, IntoTableIdent, Table}, dialect::HasDialect, ident::TableIdent, raw::IntoRaw, writer::{FormatContext, FormatWriter}};

#[derive(Debug, Default)]
pub struct Builder {
    query: String,

    distinct: bool,

    maybe_table: Option<TableIdent>,

    /// If the columns is None, we know it's a wildcard.
    columns: ColumnsIdent,
}

impl Builder {
    pub fn table_as<T: Table>() -> Self {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(T::table()),
            columns: ColumnsIdent::None,
        }
    }

    pub fn table<T>(table: T) -> Self
    where
        T: IntoTableIdent
    {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(table.into_table_ident()),
            columns: ColumnsIdent::None,
        }
    }

    pub fn select_raw<T: IntoRaw>(&mut self, value: T) -> &mut Self {
        let raw = value.into_raw();
        self.columns = ColumnsIdent::Single(TableIdent::Raw(raw));
        self
    }

    pub fn select_as<T: Columns>(&mut self) -> &mut Self {
        self.columns = T::columns();
        self
    }

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

    pub fn to_sql<Database: HasDialect>(&mut self) -> &str {
        let size_hint = 64;
        let mut str = String::with_capacity(size_hint);
        let mut context = FormatContext::new(&mut str, Database::DIALECT);
        self.format_writer(&mut context).expect("should not fail on a string writer");
        self.query = str;
        self.query.as_str()
    }
}



impl FormatWriter for Builder {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        context.writer.write_str("select ")?;
        if self.distinct {
            context.writer.write_str(" distinct ")?;
        }
        self.columns.format_writer(context)?;
        if let Some(ref table) = self.maybe_table {
            context.writer.write_str(" from ")?;
            table.format_writer(context)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{col::Columns, dialect::Postgres, Ident};

    use super::*;

    #[test]
    fn test_basic_select() {
        let mut builder = Builder::table("users");
        builder.select("i\"d");
        assert_eq!("select \"i\"\"d\" from \"users\"", builder.to_sql::<Postgres>());
        builder.select("username");
        assert_eq!("select \"username\" from \"users\"", builder.to_sql::<Postgres>());
        builder.add_select("id");
        assert_eq!("select \"username\", \"id\" from \"users\"", builder.to_sql::<Postgres>());
        builder.reset_select();
        assert_eq!("select * from \"users\"", builder.to_sql::<Postgres>());
    }

    #[derive(Debug, Clone)]
    struct User {
        id: i64,
        admin: bool,
    }

    // generated ?
    impl Table for User {
        fn table() -> TableIdent {
            TableIdent::ident_static("users")
        }
    }

    // generated ?
    impl Columns for User {
        fn columns() -> ColumnsIdent {
            [Ident::new_static("id"), Ident::new_static("admin")].into_columns()
        }
    }

    #[test]
    fn test_select_into_ident() {
        let mut builder = Builder::table_as::<User>();
        builder.select_as::<User>();
        assert_eq!("select \"id\", \"admin\" from \"users\"", builder.to_sql::<Postgres>());
    }

    #[test]
    fn test_select_raw() {
        let mut builder = Builder::table("users");
        builder.select_raw("id, count(*)");
        assert_eq!("select id, count(*) from \"users\"", builder.to_sql::<Postgres>());
    }
}
