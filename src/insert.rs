use crate::{
    Binds, Dialect, Ident, IntoTable, TableRef,
    bind::Array,
    col::IntoColumns,
    expr::TakeBindings,
    ident::{IntoIdent, RawOrIdent},
    row::{IntoRow, Row},
    writer::{FormatContext, FormatWriter},
};
use crate::{Builder, HasDialect};

pub type Columns = Array<RawOrIdent>;

impl IntoTable for RawOrIdent {
    fn into_table(self) -> crate::TableRef {
        match self {
            RawOrIdent::Ident(ident) => TableRef::Ident(ident),
            RawOrIdent::Raw(raw) => TableRef::Raw(raw),
        }
    }
}

#[derive(Debug)]
pub struct InsertBuilder {
    table: Ident,
    columns: Columns,
    binds: Binds,
    rows: Vec<Row>,
    maybe_conflict_cols: Option<Array<RawOrIdent>>,
    maybe_sets: Option<Array<RawOrIdent>>,
    maybe_select: Option<Box<Builder>>,
}

impl FormatWriter for Array<Ident> {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut FormatContext<'_, W>,
    ) -> std::fmt::Result {
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
            columns: Columns::None,
            binds: Binds::None,
            maybe_conflict_cols: None,
            maybe_sets: None,
            maybe_select: None,
            rows: Vec::new(),
        }
    }

    pub fn row<R: IntoRow>(&mut self, row: R) -> &mut Self {
        let mut row = row.into_row();
        self.binds.append(row.take_bindings());
        self.rows.push(row);
        self
    }

    pub fn rows<I, R>(&mut self, rows: I) -> &mut Self
    where
        I: IntoIterator<Item = R>,
        R: IntoRow,
    {
        for row in rows {
            self.row(row);
        }
        self
    }

    pub fn upsert<C, S>(&mut self, conflicted: C, set_cols: S) -> &mut Self
    where
        C: IntoColumns,
        S: IntoColumns,
    {
        let conflicted = conflicted.into_columns();
        let set_cols = set_cols.into_columns();

        let target = self.maybe_conflict_cols.get_or_insert_default();
        target.append(conflicted);
        let target = self.maybe_sets.get_or_insert_default();
        target.append(set_cols);

        self
    }

    pub fn select<C, F>(&mut self, cols: C, select: F) -> &mut Self
    where
        F: FnOnce(&mut Builder),
        C: IntoColumns,
    {
        // todo something with cols
        self.columns.append(cols.into_columns());
        let mut builder = Builder::default();
        select(&mut builder);
        self.binds = Binds::None;
        let select_binds = builder.take_bindings();
        self.binds.append(select_binds);
        self.maybe_select = Some(Box::new(builder));

        self
    }

    pub fn build(&mut self) -> Self {
        Self {
            table: std::mem::take(&mut self.table),
            columns: self.columns.take(),
            binds: self.binds.take(),
            rows: std::mem::take(&mut self.rows),
            maybe_conflict_cols: self.maybe_conflict_cols.take(),
            maybe_sets: self.maybe_sets.take(),
            maybe_select: self.maybe_select.take(),
        }
    }

    pub fn to_sql<Database: HasDialect>(&self) -> String {
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
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut FormatContext<'_, W>,
    ) -> std::fmt::Result {
        // sanity check
        context.writer.write_str("insert into ")?;
        self.table.format_writer(context)?;
        context.writer.write_str(" (")?;
        if !self.columns.is_empty() {
            self.columns.format_writer(context)?;
        } else if let Some(row) = &self.rows.first() {
            // if no columns are specified, we use the first row's columns
            row.format_idents(context)?;
        } else {
            return Err(std::fmt::Error);
        }
        context.writer.write_str(") ")?;

        if let Some(ref select_builder) = self.maybe_select {
            select_builder.format_writer(context)?;
        } else {
            context.writer.write_str("values ")?;
            // print the rows
            for (i, row) in self.rows.iter().enumerate() {
                if i > 0 {
                    context.writer.write_str(", ")?;
                }
                context.writer.write_char('(')?;
                row.format_values(context)?;
                context.writer.write_char(')')?;
            }
        }
        if let Some(ref conflicts) = self.maybe_conflict_cols {
            if !conflicts.is_empty()
                && matches!(context.dialect, Dialect::Postgres | Dialect::Sqlite)
            {
                context.writer.write_str(" on conflict (")?;
                conflicts.format_writer(context)?;
                context.writer.write_char(')')?;
            } else if matches!(context.dialect, Dialect::MySql) {
                context.writer.write_str(" on duplicate key update ")?;
            }
        }
        if let Some(ref sets) = self.maybe_sets {
            if !sets.is_empty() && matches!(context.dialect, Dialect::Postgres | Dialect::Sqlite) {
                context.writer.write_str(" do update set ")?;
                for (index, set) in sets.iter().enumerate() {
                    if index > 0 {
                        context.writer.write_str(", ")?;
                    }
                    set.format_writer(context)?;
                    context.writer.write_str(" = ")?;
                    let col_name = set.table_name();
                    let ident = Ident::new(smol_str::format_smolstr!("excluded.{}", col_name));
                    ident.format_writer(context)?;
                }
            } else if matches!(context.dialect, Dialect::MySql) {
                for (index, set) in sets.iter().enumerate() {
                    if index > 0 {
                        context.writer.write_str(", ")?;
                    }
                    set.format_writer(context)?;
                    context.writer.write_str(" = values(")?;
                    set.format_writer(context)?;
                    context.writer.write_char(')')?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{MySql, Postgres, Sqlite, lit};

    use super::*;

    #[test]
    fn test_format_upsert() {
        let insert = InsertBuilder::insert_into("users")
            .row(|row: &mut Row| {
                row.field("username", "ovior").field("name", "ovior");
            })
            .upsert(["id"], ["username", "name"])
            .build();

        assert_eq!(
            r#"insert into "users" ("username", "name") values ($1, $2) on conflict ("id") do update set "username" = "excluded"."username", "name" = "excluded"."name""#,
            insert.to_sql::<Postgres>()
        );
        assert_eq!(
            r#"insert into `users` (`username`, `name`) values (?, ?) on duplicate key update `username` = values(`username`), `name` = values(`name`)"#,
            insert.to_sql::<MySql>()
        );
        assert_eq!(
            r#"insert into "users" ("username", "name") values (?1, ?2) on conflict ("id") do update set "username" = "excluded"."username", "name" = "excluded"."name""#,
            insert.to_sql::<Sqlite>()
        );
    }

    #[test]
    fn insert_builder() {
        let insert = Builder::insert_into("jobs")
            .select(["model_type", "model_id", "type"], |builder| {
                builder
                    .from("jobs")
                    .select_raw("'topic', ?, 'fetch topic posts'", 1)
                    .where_not_exists(|b: &mut Builder| {
                        b.select_one()
                            .where_eq("model_type", lit("topic"))
                            .where_eq("model_id", 1)
                            .where_eq("status", lit("queued"));
                    });
            })
            .build();

        assert_eq!(
            r#"insert into "jobs" ("model_type", "model_id", "type") select 'topic', $1, 'fetch topic posts' from "jobs" where not exists (select 1 where "model_type" = 'topic' and "model_id" = $2 and "status" = 'queued')"#,
            insert.to_sql::<Postgres>()
        );
    }

    #[test]
    fn insert_builder_row() {
        let insert = InsertBuilder::insert_into("users")
            .row(|row: &mut Row| {
                row.field("username", "ovior").field("name", "ovior");
            })
            .row(|row: &mut Row| {
                row.field("username", "ovior").field("name", "ovior");
            })
            .build();

        assert_eq!(
            r#"insert into "users" ("username", "name") values ($1, $2), ($3, $4)"#,
            insert.to_sql::<Postgres>()
        );
    }
}
