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

pub trait IntoColumns {
    fn into_columns(self) -> Columns;
}

trait IntoTableIdent {
    fn into_table_ident(self) -> TableIdent;
}

impl IntoTableIdent for &str {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::Ident(Ident::new(self))
    }
}

impl IntoTableIdent for String {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::Ident(Ident::new(self))
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

impl<T> IntoColumns for T
where
    T: IntoTableIdent,
{
    fn into_columns(self) -> Columns {
        Columns::Single(self.into_table_ident())
    }
}

impl<T, const N: usize> IntoColumns for [T; N]
where
    T: IntoTableIdent,
{
    fn into_columns(self) -> Columns {
        let vec: Vec<TableIdent> =
            self.map(|t| t.into_table_ident()).to_vec();
        Columns::Many(vec)
    }
}

impl<T> IntoColumns for Vec<T>
where
    T: IntoTableIdent,
{
    fn into_columns(self) -> Columns {
        let vec = self.into_iter().map(|t| t.into_table_ident()).collect();
        Columns::Many(vec)
    }
}

impl IntoColumns for Columns {
    fn into_columns(self) -> Columns {
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{dialect::Dialect, tests::format_writer};

    use super::*;

    fn test<T>(value: T) -> Columns
    where
        T: IntoColumns
    {
        value.into_columns()
    }

    #[test]
    fn test_into_columns() {
        test("hello");
        test(String::from("hello"));
        test(Ident::new("test?"));
        test(["hello"]);
        test([TableIdent::Ident(Ident::new_static("bob")), TableIdent::Raw(Raw::new_static("test"))]);
    }

    #[test]
    fn test_format_wildcard() {
        let s = Columns::None;
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("*", wildcard);
    }

    #[test]
    fn test_single_column() {
        let s = test("id");
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\"", wildcard);
    }

    #[test]
    fn test_multi_column() {
        let s = test([TableIdent::Ident(Ident::new_static("id")), TableIdent::Raw(Raw::new_static("count(*)")), TableIdent::Ident(Ident::new_static("username"))]);
        let wildcard = format_writer(s, Dialect::Postgres);
        assert_eq!("\"id\", count(*), \"username\"", wildcard);
    }
}
