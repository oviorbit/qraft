use std::fmt;

use crate::{
    bind::{Array, Bind, Binds}, expr::{exists::ExistsExpr, fncall::AggregateCall, r#in::InExpr, Expr, TakeBindings}, ident::{Ident, IntoIdent, RawOrIdent, TableRef}, writer::FormatWriter, Builder, Raw
};

pub type Projections = Array<Expr>;

impl FormatWriter for Projections {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> fmt::Result {
        match self {
            Projections::None => context.writer.write_char('*')?,
            Projections::One(ident) => ident.format_writer(context)?,
            Projections::Many(idents) => {
                // just format the elem seperated with comma
                for (index, elem) in idents.iter().enumerate() {
                    if index > 0 {
                        context.writer.write_str(", ")?;
                    }
                    elem.format_writer(context)?;
                }
            }
        };
        Ok(())
    }
}

pub trait TableSchema {
    fn table() -> TableRef;
}

pub trait ProjectionSchema {
    fn projections() -> Projections;
}

pub trait IntoColumns {
    fn into_columns(self) -> Array<RawOrIdent>;
}

impl FormatWriter for RawOrIdent {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            RawOrIdent::Ident(ident) => ident.format_writer(context),
            RawOrIdent::Raw(raw) => raw.format_writer(context),
        }
    }
}

impl FormatWriter for Array<RawOrIdent> {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            Self::None => {}
            Self::One(ident) => ident.format_writer(context)?,
            Self::Many(idents) => {
                for (index, elem) in idents.iter().enumerate() {
                    if index > 0 {
                        context.writer.write_str(", ")?;
                    }
                    elem.format_writer(context)?;
                }
            }
        };
        Ok(())
    }
}

impl<T> IntoColumns for T
where
    T: IntoIdent,
{
    fn into_columns(self) -> Array<RawOrIdent> {
        Array::One(RawOrIdent::Ident(self.into_ident()))
    }
}

impl IntoColumns for Raw {
    fn into_columns(self) -> Array<RawOrIdent> {
        Array::One(RawOrIdent::Raw(self))
    }
}

impl<T, const N: usize> IntoColumns for [T; N]
where
    T: IntoIdent + Clone,
{
    fn into_columns(self) -> Array<RawOrIdent> {
        // cheap clone O(1)
        if N == 1 {
            Array::One(RawOrIdent::Ident(self[0].clone().into_ident()))
        } else {
            let vec: Vec<RawOrIdent> = self.map(|t| RawOrIdent::Ident(t.into_ident())).to_vec();
            Array::Many(vec)
        }
    }
}

impl<T> IntoColumns for Vec<T>
where
    T: IntoIdent,
{
    fn into_columns(self) -> Array<RawOrIdent> {
        let vec = self
            .into_iter()
            .map(|t| RawOrIdent::Ident(t.into_ident()))
            .collect();
        Array::Many(vec)
    }
}

// add into something proj that contains stuff like subqueries and so on

pub trait IntoProjections {
    fn into_select_proj(self) -> Projections;
}

impl IntoProjections for &str {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Ident(self.into_table()))
    }
}

impl IntoProjections for String {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Ident(self.into_table()))
    }
}

impl IntoProjections for Raw {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Ident(self.into_table()))
    }
}

impl IntoProjections for Ident {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Ident(self.into_table()))
    }
}

impl IntoProjections for TableRef {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Ident(self))
    }
}

impl<const N: usize> IntoProjections for [&str; N] {
    fn into_select_proj(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(Expr::Ident(self[0].into_table()))
        } else {
            let vec: Vec<Expr> = self.map(|t| Expr::Ident(t.into_table())).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [String; N] {
    fn into_select_proj(self) -> Projections {
        let vec: Vec<Expr> = self.map(|t| Expr::Ident(t.into_table())).to_vec();
        Projections::Many(vec)
    }
}

impl<const N: usize> IntoProjections for [Ident; N] {
    fn into_select_proj(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(Expr::Ident(self[0].clone().into_table()))
        } else {
            let vec: Vec<Expr> = self.map(|t| Expr::Ident(t.into_table())).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [Raw; N] {
    fn into_select_proj(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(Expr::Ident(self[0].clone().into_table()))
        } else {
            let vec: Vec<Expr> = self.map(|t| Expr::Ident(t.into_table())).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [TableRef; N] {
    fn into_select_proj(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(Expr::Ident(self[0].clone()))
        } else {
            let vec: Vec<Expr> = self.map(Expr::Ident).to_vec();
            Projections::Many(vec)
        }
    }
}

impl IntoProjections for Vec<&str> {
    fn into_select_proj(self) -> Projections {
        let vec = self
            .into_iter()
            .map(|t| Expr::Ident(t.into_table()))
            .collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<String> {
    fn into_select_proj(self) -> Projections {
        let vec = self
            .into_iter()
            .map(|t| Expr::Ident(t.into_table()))
            .collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<Ident> {
    fn into_select_proj(self) -> Projections {
        let vec = self
            .into_iter()
            .map(|t| Expr::Ident(t.into_table()))
            .collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<Raw> {
    fn into_select_proj(self) -> Projections {
        let vec = self
            .into_iter()
            .map(|t| Expr::Ident(t.into_table()))
            .collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<TableRef> {
    fn into_select_proj(self) -> Projections {
        let vec = self
            .into_iter()
            .map(|t| Expr::Ident(t.into_table()))
            .collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Projections {
    fn into_select_proj(self) -> Projections {
        self
    }
}

impl<T: ProjectionSchema> IntoProjections for T {
    fn into_select_proj(self) -> Projections {
        T::projections()
    }
}

impl IntoProjections for Bind {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Bind(self))
    }
}

impl IntoProjections for Expr {
    fn into_select_proj(self) -> Projections {
        Projections::One(self)
    }
}

impl IntoProjections for AggregateCall {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::AggregateCall(self))
    }
}

impl IntoProjections for AliasSub {
    fn into_select_proj(self) -> Projections {
        let table_ref = TableRef::AliasSub(self);
        Projections::One(Expr::Ident(table_ref))
    }
}

impl IntoProjections for InExpr {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::In(Box::new(self)))
    }
}

impl IntoProjections for ExistsExpr {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Exists(self))
    }
}

impl IntoProjections for Builder {
    fn into_select_proj(self) -> Projections {
        Projections::One(Expr::Subquery(Box::new(self)))
    }
}

#[derive(Debug, Clone)]
pub struct AliasSub {
    pub(crate) alias: Ident,
    pub(crate) inner: Box<Builder>,
}

impl AliasSub {
    pub fn new<I>(inner: Builder, alias: I) -> Self
    where
        I: IntoIdent,
    {
        Self {
            alias: alias.into_ident(),
            inner: Box::new(inner),
        }
    }
}

impl FormatWriter for AliasSub {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        context.writer.write_char('(')?;
        self.inner.format_writer(context)?;
        context.writer.write_char(')')?;
        context.writer.write_str(" as ")?;
        self.alias.format_writer(context)?;
        Ok(())
    }
}

impl TakeBindings for AliasSub {
    fn take_bindings(&mut self) -> Binds {
        self.inner.take_bindings()
    }
}

pub trait IntoTable {
    fn into_table(self) -> TableRef;
}

impl IntoTable for AliasSub {
    fn into_table(self) -> TableRef {
        TableRef::AliasSub(self)
    }
}

impl IntoTable for &str {
    fn into_table(self) -> TableRef {
        TableRef::ident(self)
    }
}

impl IntoTable for String {
    fn into_table(self) -> TableRef {
        TableRef::ident(self)
    }
}

impl IntoTable for Raw {
    fn into_table(self) -> TableRef {
        TableRef::Raw(self)
    }
}

impl IntoTable for Ident {
    fn into_table(self) -> TableRef {
        TableRef::Ident(self)
    }
}

impl IntoTable for TableRef {
    fn into_table(self) -> TableRef {
        self
    }
}

impl<T: TableSchema> IntoTable for T {
    fn into_table(self) -> TableRef {
        T::table()
    }
}

#[cfg(test)]
mod tests {
    use crate::{column_static, dialect::Dialect, raw_static, tests::format_writer};

    use super::*;

    fn select<T>(value: T) -> Projections
    where
        T: IntoProjections,
    {
        value.into_select_proj()
    }

    #[test]
    fn test_into_columns() {
        select("hello");
        select(String::from("hello"));
        select(Ident::new("test?"));
        select(["hello"]);
        select([
            column_static("bob").into_table(),
            raw_static("test").into_table(),
        ]);
    }

    #[test]
    fn test_format_wildcard() {
        let s = Projections::None;
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("*", wildcard);
    }

    #[test]
    fn test_single_column() {
        let s = select("id");
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\"", wildcard);
    }

    #[test]
    fn test_multi_column() {
        let s = select([
            "id".into_table(),
            raw_static("count(*)").into_table(),
            "username".into_table(),
        ]);
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\", count(*), \"username\"", wildcard);
    }
}
