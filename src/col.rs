use std::fmt;

use crate::{ident::{Ident, TableIdent}, writer::FormatWriter, Raw};

#[derive(Debug, Default, Clone)]
pub enum Columns {
    #[default]
    None,
    Single(TableIdent),
    Many(Vec<TableIdent>),
}

impl Columns {
    pub fn append(&mut self, other: Self) {
        let combined = match (std::mem::replace(self, Columns::None), other) {
            (Columns::None, cols) | (cols, Columns::None) => cols,
            (Columns::Single(a), Columns::Single(b)) =>
                Columns::Many(vec![a, b]),
            (Columns::Single(a), Columns::Many(mut b)) => {
                b.insert(0, a);
                Columns::Many(b)
            }
            (Columns::Many(mut a), Columns::Single(b)) => {
                a.push(b);
                Columns::Many(a)
            }
            (Columns::Many(mut a), Columns::Many(mut b)) => {
                a.append(&mut b);
                Columns::Many(a)
            }
        };
        *self = combined;
    }

    pub fn reset(&mut self) {
        *self = Columns::None;
    }

    pub fn into_vec(self) -> Vec<TableIdent> {
        match self {
            Columns::None => Vec::new(),
            Columns::Single(one) => Vec::from([one]),
            Columns::Many(many) => many,
        }
    }
}

impl FormatWriter for Columns {
    fn format_writer<W: fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> fmt::Result {
        match self {
            Columns::None => context.writer.write_char('*')?,
            Columns::Single(ident) => ident.format_writer(context)?,
            Columns::Many(idents) => {
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
    fn table() -> TableIdent;
}

pub trait ColumnSchema {
    fn columns() -> Columns;
}

pub trait IntoColumns {
    fn into_columns(self) -> Columns;
}

pub trait IntoTable {
    fn into_table(self) -> TableIdent;
}

impl IntoTable for &str {
    fn into_table(self) -> TableIdent {
        TableIdent::ident(self)
    }
}

impl IntoTable for String {
    fn into_table(self) -> TableIdent {
        TableIdent::ident(self)
    }
}

impl IntoTable for Raw {
    fn into_table(self) -> TableIdent {
        TableIdent::Raw(self)
    }
}

impl IntoTable for Ident {
    fn into_table(self) -> TableIdent {
        TableIdent::Ident(self)
    }
}

impl IntoTable for TableIdent {
    fn into_table(self) -> TableIdent {
        self
    }
}

impl<T: TableSchema> IntoTable for T {
    fn into_table(self) -> TableIdent {
        T::table()
    }
}

impl IntoColumns for &str {
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table())
    }
}

impl IntoColumns for String {
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table())
    }
}

impl IntoColumns for Raw {
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table())
    }
}

impl IntoColumns for Ident {
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table())
    }
}

impl IntoColumns for TableIdent {
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table())
    }
}

impl<const N: usize> IntoColumns for [&str; N] {
    fn into_columns(self) -> Columns {
        // cheap clone O(1)
        if N == 1 {
            Columns::Single(self[0].into_table())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table()).to_vec();
            Columns::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [String; N] {
    fn into_columns(self) -> Columns {
        let vec: Vec<TableIdent> =
            self.map(|t| t.into_table()).to_vec();
        Columns::Many(vec)
    }
}

impl<const N: usize> IntoColumns for [Ident; N] {
    fn into_columns(self) -> Columns {
        // cheap clone O(1)
        if N == 1 {
            Columns::Single(self[0].clone().into_table())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table()).to_vec();
            Columns::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [Raw; N] {
    fn into_columns(self) -> Columns {
        // cheap clone O(1)
        if N == 1 {
            Columns::Single(self[0].clone().into_table())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table()).to_vec();
            Columns::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [TableIdent; N] {
    fn into_columns(self) -> Columns {
        // cheap clone O(1)
        if N == 1 {
            Columns::Single(self[0].clone())
        } else {
            let vec: Vec<TableIdent> = self.to_vec();
            Columns::Many(vec)
        }
    }
}

impl IntoColumns for Vec<&str> {
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Vec<String> {
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Vec<Ident> {
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Vec<Raw> {
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Vec<TableIdent> {
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Columns {
    fn into_columns(self) -> Columns {
        self
    }
}

impl<T: ColumnSchema> IntoColumns for T {
    fn into_columns(self) -> Columns {
        T::columns()
    }
}

#[cfg(test)]
mod tests {
    use crate::{col, dialect::Dialect, ident_static, raw_static, tests::format_writer};

    use super::*;

    fn select<T>(value: T) -> Columns
    where
        T: IntoColumns
    {
        value.into_columns()
    }

    #[test]
    fn test_into_columns() {
        select("hello");
        select(String::from("hello"));
        select(Ident::new("test?"));
        select(["hello"]);
        select(col![ident_static("bob"), raw_static("test")]);
    }

    #[test]
    fn test_format_wildcard() {
        let s = Columns::None;
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
        let s = select(col!["id", raw_static("count(*)"), "username"]);
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\", count(*), \"username\"", wildcard);
    }
}
