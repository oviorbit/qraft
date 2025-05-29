use std::fmt;

use crate::{ident::{Ident, TableIdent}, writer::FormatWriter, Raw};

#[derive(Debug, Default, Clone)]
pub enum ColumnsIdent {
    #[default]
    None,
    Single(TableIdent),
    Many(Vec<TableIdent>),
}

impl ColumnsIdent {
    pub fn append(&mut self, other: Self) {
        let combined = match (std::mem::replace(self, ColumnsIdent::None), other) {
            (ColumnsIdent::None, cols) | (cols, ColumnsIdent::None) => cols,
            (ColumnsIdent::Single(a), ColumnsIdent::Single(b)) =>
                ColumnsIdent::Many(vec![a, b]),
            (ColumnsIdent::Single(a), ColumnsIdent::Many(mut b)) => {
                b.insert(0, a);
                ColumnsIdent::Many(b)
            }
            (ColumnsIdent::Many(mut a), ColumnsIdent::Single(b)) => {
                a.push(b);
                ColumnsIdent::Many(a)
            }
            (ColumnsIdent::Many(mut a), ColumnsIdent::Many(mut b)) => {
                a.append(&mut b);
                ColumnsIdent::Many(a)
            }
        };
        *self = combined;
    }

    pub fn reset(&mut self) {
        *self = ColumnsIdent::None;
    }

    pub fn into_vec(self) -> Vec<TableIdent> {
        match self {
            ColumnsIdent::None => Vec::new(),
            ColumnsIdent::Single(one) => Vec::from([one]),
            ColumnsIdent::Many(many) => many,
        }
    }
}

impl FormatWriter for ColumnsIdent {
    fn format_writer<W: fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> fmt::Result {
        match self {
            ColumnsIdent::None => context.writer.write_char('*')?,
            ColumnsIdent::Single(ident) => ident.format_writer(context)?,
            ColumnsIdent::Many(idents) => {
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

pub trait Table {
    fn table() -> TableIdent;
}

pub trait Columns {
    fn columns() -> ColumnsIdent;
}

pub trait IntoColumns {
    fn into_columns(self) -> ColumnsIdent;
}

pub trait IntoTableIdent {
    fn into_table_ident(self) -> TableIdent;
}

impl IntoTableIdent for &str {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::ident(self)
    }
}

impl IntoTableIdent for String {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::ident(self)
    }
}

impl IntoTableIdent for Raw {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::Raw(self)
    }
}

impl IntoTableIdent for Ident {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::Ident(self)
    }
}

impl IntoTableIdent for TableIdent {
    fn into_table_ident(self) -> TableIdent {
        self
    }
}

impl<T: Table> IntoTableIdent for T {
    fn into_table_ident(self) -> TableIdent {
        T::table()
    }
}

impl IntoColumns for &str {
    fn into_columns(self) -> ColumnsIdent {
        ColumnsIdent::Single(self.into_table_ident())
    }
}

impl IntoColumns for String {
    fn into_columns(self) -> ColumnsIdent {
        ColumnsIdent::Single(self.into_table_ident())
    }
}

impl IntoColumns for Raw {
    fn into_columns(self) -> ColumnsIdent {
        ColumnsIdent::Single(self.into_table_ident())
    }
}

impl IntoColumns for Ident {
    fn into_columns(self) -> ColumnsIdent {
        ColumnsIdent::Single(self.into_table_ident())
    }
}

impl IntoColumns for TableIdent {
    fn into_columns(self) -> ColumnsIdent {
        ColumnsIdent::Single(self.into_table_ident())
    }
}

impl<const N: usize> IntoColumns for [&str; N] {
    fn into_columns(self) -> ColumnsIdent {
        // cheap clone O(1)
        if N == 1 {
            ColumnsIdent::Single(self[0].into_table_ident())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table_ident()).to_vec();
            ColumnsIdent::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [String; N] {
    fn into_columns(self) -> ColumnsIdent {
        let vec: Vec<TableIdent> =
            self.map(|t| t.into_table_ident()).to_vec();
        ColumnsIdent::Many(vec)
    }
}

impl<const N: usize> IntoColumns for [Ident; N] {
    fn into_columns(self) -> ColumnsIdent {
        // cheap clone O(1)
        if N == 1 {
            ColumnsIdent::Single(self[0].clone().into_table_ident())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table_ident()).to_vec();
            ColumnsIdent::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [Raw; N] {
    fn into_columns(self) -> ColumnsIdent {
        // cheap clone O(1)
        if N == 1 {
            ColumnsIdent::Single(self[0].clone().into_table_ident())
        } else {
            let vec: Vec<TableIdent> =
                self.map(|t| t.into_table_ident()).to_vec();
            ColumnsIdent::Many(vec)
        }
    }
}

impl<const N: usize> IntoColumns for [TableIdent; N] {
    fn into_columns(self) -> ColumnsIdent {
        // cheap clone O(1)
        if N == 1 {
            ColumnsIdent::Single(self[0].clone())
        } else {
            let vec: Vec<TableIdent> = self.to_vec();
            ColumnsIdent::Many(vec)
        }
    }
}

impl IntoColumns for Vec<&str> {
    fn into_columns(self) -> ColumnsIdent {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        ColumnsIdent::Many(vec)
    }
}

impl IntoColumns for Vec<String> {
    fn into_columns(self) -> ColumnsIdent {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        ColumnsIdent::Many(vec)
    }
}

impl IntoColumns for Vec<Ident> {
    fn into_columns(self) -> ColumnsIdent {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        ColumnsIdent::Many(vec)
    }
}

impl IntoColumns for Vec<Raw> {
    fn into_columns(self) -> ColumnsIdent {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        ColumnsIdent::Many(vec)
    }
}

impl IntoColumns for Vec<TableIdent> {
    fn into_columns(self) -> ColumnsIdent {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        ColumnsIdent::Many(vec)
    }
}

impl IntoColumns for ColumnsIdent {
    fn into_columns(self) -> ColumnsIdent {
        self
    }
}

impl<T: Columns> IntoColumns for T {
    fn into_columns(self) -> ColumnsIdent {
        T::columns()
    }
}

#[cfg(test)]
mod tests {
    use crate::{col, dialect::Dialect, ident, raw, tests::format_writer};

    use super::*;

    fn select<T>(value: T) -> ColumnsIdent
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
        select(col![ident("bob"), raw("test")]);
    }

    #[test]
    fn test_format_wildcard() {
        let s = ColumnsIdent::None;
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
        let s = select(col!["id", raw("count(*)"), "username"]);
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\", count(*), \"username\"", wildcard);
    }
}
