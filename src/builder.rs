use crate::{
    bind::{Binds, IntoBinds},
    col::{ColumnSchema, Columns, IntoColumns, IntoTable, TableSchema},
    dialect::HasDialect,
    expr::{
        cond::{Condition, Conditions, Conjunction}, Binary, ConditionKind, Group
    },
    ident::TableIdent,
    operator::Operator,
    raw::IntoRaw,
    scalar::{ScalarExpr, TakeBindings},
    writer::{FormatContext, FormatWriter},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    #[default]
    Select,
    Where,
}

impl TakeBindings for Builder {
    fn take_bindings(&mut self) -> Binds {
        std::mem::take(&mut self.binds)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    query: String,
    ty: QueryKind,
    distinct: bool,
    maybe_table: Option<TableIdent>,
    columns: Columns,
    binds: Binds,
    maybe_where: Option<Conditions>,
}

impl Builder {
    pub fn table_as<T: TableSchema>() -> Self {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(T::table()),
            columns: Columns::None,
            binds: Binds::None,
            ty: QueryKind::Select,
            maybe_where: None,
        }
    }

    pub fn table<T>(table: T) -> Self
    where
        T: IntoTable,
    {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(table.into_table()),
            columns: Columns::None,
            binds: Binds::None,
            ty: QueryKind::Select,
            maybe_where: None,
        }
    }

    pub fn from<T: IntoTable>(&mut self, table: T) -> &mut Self {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        self.maybe_table = Some(table.into_table());
        self
    }

    // where stuff

    #[inline]
    pub fn where_binary_expr(
        &mut self,
        conjunction: Conjunction,
        mut lhs: ScalarExpr,
        operator: Operator,
        mut rhs: ScalarExpr,
    ) -> &mut Self {
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());

        let binary = Binary { lhs, operator, rhs };
        let expr = ConditionKind::Binary(binary);
        let condition = Condition::new(conjunction, expr);
        let ws = self.maybe_where.get_or_insert_default();
        ws.push(condition);

        self
    }

    pub fn where_group_expr<F>(&mut self, conjunction: Conjunction, closure: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        // with a type of where, we should ignore most actions
        let mut inner = Self {
            ty: QueryKind::Where,
            ..Default::default()
        };
        // modify the internal states with wheres
        closure(&mut inner);

        let binds = inner.take_bindings();
        if let Some(inner_conds) = inner.maybe_where {
            self.binds.append(binds);

            let group = Group {
                conditions: inner_conds,
            };
            let kind = ConditionKind::Group(group);

            let ws = self.maybe_where.get_or_insert_default();
            ws.push(Condition::new(conjunction, kind));
        }

        self
    }

    // select stuff

    pub fn select_raw<T: IntoRaw, B: IntoBinds>(&mut self, value: T, binds: B) -> &mut Self {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        let raw = value.into_raw();
        self.columns = Columns::One(TableIdent::Raw(raw));
        self.binds.append(binds.into_binds());
        self
    }

    pub fn select_as<T: ColumnSchema>(&mut self) -> &mut Self {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        self.columns = T::columns();
        self
    }

    pub fn select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoColumns,
    {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        self.columns = cols.into_columns();
        self
    }

    pub fn add_select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoColumns,
    {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        let other = cols.into_columns();
        self.columns.append(other);
        self
    }

    pub fn reset_select(&mut self) -> &mut Self {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        self.columns.reset();
        self
    }

    pub fn distinct(&mut self) -> &mut Self {
        if matches!(self.ty, QueryKind::Where) {
            return self;
        }
        self.distinct = true;
        self
    }

    // building the builder

    pub fn to_sql<Database: HasDialect>(&mut self) -> &str {
        let size_hint = 64;
        let mut str = String::with_capacity(size_hint);
        let mut context = FormatContext::new(&mut str, Database::DIALECT);
        self.format_writer(&mut context)
            .expect("should not fail on a string writer");
        self.query = str;
        self.query.as_str()
    }
}

impl FormatWriter for Builder {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        context.writer.write_str("select ")?;
        if self.distinct {
            context.writer.write_str(" distinct ")?;
        }
        self.columns.format_writer(context)?;
        if let Some(ref table) = self.maybe_table {
            context.writer.write_str(" from ")?;
            table.format_writer(context)?;
        }

        if let Some(ref w) = self.maybe_where {
            // if we are not in a group and in select query
            if !w.0.is_empty() && matches!(self.ty, QueryKind::Select) {
                context.writer.write_str(" where ")?;
            }
            w.format_writer(context)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bind::{self, Bind},
        col::ColumnSchema,
        column_static,
        dialect::Postgres,
        scalar::{IntoScalar, IntoScalarIdent},
        sub,
    };

    use super::*;

    #[test]
    fn test_basic_select() {
        let mut builder = Builder::table("users");
        builder.select("i\"d");
        assert_eq!(
            "select \"i\"\"d\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
        builder.select("username");
        assert_eq!(
            "select \"username\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
        builder.add_select("id");
        assert_eq!(
            "select \"username\", \"id\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
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
        assert_eq!(
            "select \"id\", \"admin\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_select_raw() {
        let mut builder = Builder::table("users");
        builder.select_raw("id, count(*)", Binds::None);
        assert_eq!(
            "select id, count(*) from \"users\"",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_select_raw_bound() {
        let mut builder = Builder::table("users");
        builder.select_raw("price + ? as fee", [5]);
        assert_eq!(
            "select price + $1 as fee from \"users\"",
            builder.to_sql::<Postgres>()
        );
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
        builder.where_binary_expr(
            Conjunction::And,
            "id".into_scalar_ident().0,
            Operator::Eq,
            sub(|builder| {
                builder.select("id").from("roles");
            })
            .into_scalar()
            .0,
        );
        assert_eq!(
            "select * from \"users\" where \"id\" = (select \"id\" from \"roles\")",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_scalar_and_conds() {
        let mut builder = Builder::table("users");
        builder.where_binary_expr(
            Conjunction::And,
            "username".into_scalar_ident().0,
            Operator::Like,
            3.into_scalar().0,
        );
        builder.where_binary_expr(
            Conjunction::And,
            "id".into_scalar_ident().0,
            Operator::Eq,
            3.into_scalar().0,
        );
        builder.where_binary_expr(
            Conjunction::Or,
            "name".into_scalar_ident().0,
            Operator::Eq,
            3.into_scalar().0,
        );
        builder.where_binary_expr(
            Conjunction::And,
            "foo".into_scalar_ident().0,
            Operator::Eq,
            sub(|builder| {
                builder.select("id").from("bar");
            })
            .into_scalar()
            .0,
        );
        assert_eq!(
            "select * from \"users\" where \"username\"::text like $1 and \"id\" = $2 or \"name\" = $3 and \"foo\" = (select \"id\" from \"bar\")",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_scalar_value_column() {
        let mut builder = Builder::table("users");
        builder.where_binary_expr(
            Conjunction::And,
            sub(|builder| {
                builder.select("foo").from("bar");
            }).into_scalar_ident().0,
            Operator::Like,
            3.into_scalar().0,
        );
        assert_eq!(
            "select * from \"users\" where (select \"foo\" from \"bar\")::text like $1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_scalar_like() {
        let mut builder = Builder::table("users");
        builder.where_binary_expr(
            Conjunction::And,
            "username".into_scalar_ident().0,
            Operator::Like,
            3.into_scalar().0,
        );
        assert_eq!(
            "select * from \"users\" where \"username\"::text like $1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_where_group_expr() {
        let mut builder = Builder::table("users");
        builder.where_group_expr(Conjunction::And, |builder| {
            builder
                .where_binary_expr(Conjunction::And, "foo".into_scalar_ident().0, Operator::Eq, 3.into_scalar().0)
                .where_binary_expr(Conjunction::Or, "bar".into_scalar_ident().0, Operator::Like, "bob".into_scalar_ident().0);
        });
        assert_eq!(
            "select * from \"users\" where (\"foo\" = $1 or \"bar\"::text like \"bob\")",
            builder.to_sql::<Postgres>()
        );
    }
}
