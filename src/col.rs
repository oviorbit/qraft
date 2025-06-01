use std::fmt;

use crate::{
    bind::Array, expr::{sub::AliasSubFn, TakeBindings}, ident::{Ident, IntoIdent, TableRef}, writer::FormatWriter, Builder, Raw
};

pub type Projections = Array<TableRef>;

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

pub trait IntoProjections {
    fn into_projections(self) -> Projections;
}

pub trait IntoProjectionsWithSub {
    fn into_projections_with_sub(self) -> Projections;
}

impl<T> IntoProjectionsWithSub for T
where
    T: IntoProjections,
{
    fn into_projections_with_sub(self) -> Projections {
        self.into_projections()
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
    fn take_bindings(&mut self) -> crate::Binds {
        self.inner.take_bindings()
    }
}

impl IntoProjectionsWithSub for AliasSub {
    fn into_projections_with_sub(self) -> Projections {
        let table_ref = TableRef::AliasSub(self);
        Projections::One(table_ref)
    }
}

impl IntoProjectionsWithSub for AliasSubFn {
    fn into_projections_with_sub(self) -> Projections {
        let table_ref = TableRef::AliasSubFn(self);
        Projections::One(table_ref)
    }
}

impl IntoTable for AliasSub {
    fn into_table(self) -> TableRef {
        TableRef::AliasSub(self)
    }
}

pub trait IntoTable {
    fn into_table(self) -> TableRef;
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

impl IntoProjections for &str {
    fn into_projections(self) -> Projections {
        Projections::One(self.into_table())
    }
}

impl IntoProjections for String {
    fn into_projections(self) -> Projections {
        Projections::One(self.into_table())
    }
}

impl IntoProjections for Raw {
    fn into_projections(self) -> Projections {
        Projections::One(self.into_table())
    }
}

impl IntoProjections for Ident {
    fn into_projections(self) -> Projections {
        Projections::One(self.into_table())
    }
}

impl IntoProjections for TableRef {
    fn into_projections(self) -> Projections {
        Projections::One(self.into_table())
    }
}

impl<const N: usize> IntoProjections for [&str; N] {
    fn into_projections(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(self[0].into_table())
        } else {
            let vec: Vec<TableRef> = self.map(|t| t.into_table()).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [String; N] {
    fn into_projections(self) -> Projections {
        let vec: Vec<TableRef> = self.map(|t| t.into_table()).to_vec();
        Projections::Many(vec)
    }
}

impl<const N: usize> IntoProjections for [Ident; N] {
    fn into_projections(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(self[0].clone().into_table())
        } else {
            let vec: Vec<TableRef> = self.map(|t| t.into_table()).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [Raw; N] {
    fn into_projections(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(self[0].clone().into_table())
        } else {
            let vec: Vec<TableRef> = self.map(|t| t.into_table()).to_vec();
            Projections::Many(vec)
        }
    }
}

impl<const N: usize> IntoProjections for [TableRef; N] {
    fn into_projections(self) -> Projections {
        // cheap clone O(1)
        if N == 1 {
            Projections::One(self[0].clone())
        } else {
            let vec: Vec<TableRef> = self.to_vec();
            Projections::Many(vec)
        }
    }
}

impl IntoProjections for Vec<&str> {
    fn into_projections(self) -> Projections {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<String> {
    fn into_projections(self) -> Projections {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<Ident> {
    fn into_projections(self) -> Projections {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<Raw> {
    fn into_projections(self) -> Projections {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Vec<TableRef> {
    fn into_projections(self) -> Projections {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Projections::Many(vec)
    }
}

impl IntoProjections for Projections {
    fn into_projections(self) -> Projections {
        self
    }
}

impl<T: ProjectionSchema> IntoProjections for T {
    fn into_projections(self) -> Projections {
        T::projections()
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
        value.into_projections()
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
