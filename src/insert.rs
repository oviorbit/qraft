use crate::{bind::Array, expr::{Expr, TakeBindings}, ident::IntoIdent, writer::{FormatContext, FormatWriter}, Binds, Ident, IntoRhsExpr};
use crate::HasDialect;

#[derive(Debug)]
pub struct InsertBuilder {
    table: Ident,
    columns: Array<Ident>,
    values: Vec<Expr>,
    binds: Binds,
}

impl FormatWriter for Array<Ident> {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        for (index, ident) in self.iter().enumerate() {
            if index > 0 {
                context.writer.write_str(", ")?;
            }
            ident.format_writer(context)?;
        }
        Ok(())
    }
}

impl InsertBuilder {
    pub fn insert_into<T: IntoIdent>(table: T) -> Self {
        Self {
            table: table.into_ident(),
            columns: Array::None,
            binds: Binds::None,
            values: Vec::default(),
        }
    }

    pub fn field<K, V>(&mut self, column: K, value: V) -> &mut Self
    where
        K: IntoIdent,
        V: IntoRhsExpr,
    {
        self.columns.push(column.into_ident());
        let mut value = value.into_rhs_expr();
        self.binds.append(value.take_bindings());
        self.values.push(value);
        self
    }

    pub fn to_sql<Database: HasDialect>(&mut self) -> String {
        let size_hint = 64;
        let mut str = String::with_capacity(size_hint);
        let mut context = FormatContext::new(&mut str, Database::DIALECT);
        self.format_writer(&mut context)
            .expect("should not fail on a string writer");
        str
    }

    #[cfg(any(feature = "postgres", feature = "sqlite", feature = "mysql"))]
    pub async fn execute<DB, E>(
        &mut self,
        executor: E,
    ) -> Result<<DB as sqlx::Database>::QueryResult, sqlx::Error>
    where
        DB: sqlx::Database + HasDialect,
        E: for<'c> sqlx::Executor<'c, Database = DB>,
        Binds: for<'c> sqlx::IntoArguments<'c, DB>,
        <DB as sqlx::Database>::QueryResult: crate::HasRowsAffected,
    {
        let bindings = self.binds.take();
        let sql = self.to_sql::<DB>();
        sqlx::query_with::<_, _>(&sql, bindings)
            .execute(executor)
            .await
    }
}

impl FormatWriter for InsertBuilder {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        // sanity check
        debug_assert!(self.columns.len() == self.values.len());
        context.writer.write_str("insert into ")?;
        self.table.format_writer(context)?;
        context.writer.write_str(" (")?;
        self.columns.format_writer(context)?;
        context.writer.write_str(") values (")?;
        for (i, expr) in self.values.iter().enumerate() {
            if i > 0 {
                context.writer.write_str(", ")?;
            }
            expr.format_writer(context)?;
        }
        context.writer.write_char(')')?;
        Ok(())
    }
}
