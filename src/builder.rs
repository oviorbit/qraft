use crate::{bind::{Binds, IntoBinds}, col::{ColumnSchema, Columns, IntoColumns, IntoTable, TableSchema}, dialect::HasDialect, ident::TableIdent, raw::IntoRaw, scalar::{IntoScalar, IntoScalarIdent}, sub::Subquery, writer::{FormatContext, FormatWriter }};

#[derive(Debug, Default, Clone)]
pub struct Builder {
    query: String,
    distinct: bool,
    maybe_table: Option<TableIdent>,
    columns: Columns,
    binds: Binds,
}

impl Builder {
    pub fn table_as<T: TableSchema>() -> Self {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(T::table()),
            columns: Columns::None,
            binds: Binds::None,
        }
    }

    pub fn table<T>(table: T) -> Self
    where
        T: IntoTable
    {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(table.into_table()),
            columns: Columns::None,
            binds: Binds::None,
        }
    }

    pub fn from<T: IntoTable>(&mut self, table: T) -> &mut Self
    {
        self.maybe_table = Some(table.into_table());
        self
    }

    // where stuff
    pub fn where_eq<C, S>(&mut self, column: C, scalar: S) -> &mut Self
    where
        C: IntoScalarIdent,
        S: IntoScalar
    {
        self
    }

    // select stuff

    pub fn select_raw<T: IntoRaw, B: IntoBinds>(&mut self, value: T, binds: B) -> &mut Self {
        let raw = value.into_raw();
        self.columns = Columns::One(TableIdent::Raw(raw));
        self.binds.append(binds.into_binds());
        self
    }

    pub fn select_as<T: ColumnSchema>(&mut self) -> &mut Self {
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

    // building the builder

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
    use crate::{bind::{self, Bind}, col::ColumnSchema, column_static, dialect::Postgres, sub};

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
    impl TableSchema for User {
        fn table() -> TableIdent {
            TableIdent::ident_static("users")
        }
    }

    // generated ?
    impl ColumnSchema for User {
        fn columns() -> Columns {
            [column_static("id"), column_static("admin")].into_columns()
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
        builder.select_raw("id, count(*)", Binds::None);
        assert_eq!("select id, count(*) from \"users\"", builder.to_sql::<Postgres>());
    }

    #[test]
    fn test_select_raw_bound() {
        let mut builder = Builder::table("users");
        builder.select_raw("price + ? as fee", [5]);
        assert_eq!("select price + $1 as fee from \"users\"", builder.to_sql::<Postgres>());
        assert_eq!(1, builder.binds.len());
        let value = match builder.binds {
            bind::Array::None => panic!("should have one value"),
            bind::Array::One(value) => value,
            bind::Array::Many(_) => panic!("wrong size"),
        };
        assert!(matches!(value, Bind::I32(5)));
    }

    #[test]
    fn test_scalar_where() {
        let mut builder = Builder::table("users");
        builder.where_eq("id", |builder: &mut Builder| {
            builder.select("id").from("roles");
        });
        assert_eq!("select * from \"users\" where \"id\" = $1", builder.to_sql::<Postgres>());
    }
}
