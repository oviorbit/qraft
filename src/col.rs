use std::fmt;

use crate::{ident::{Ident, TableIdent}, writer::FormatWriter, Raw};

#[derive(Debug, Clone)]
pub enum Columns {
    None,
    Single(TableIdent),
    Many(Vec<TableIdent>),
}

impl Columns {
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

impl IntoTableIdent for &'static str {
    fn into_table_ident(self) -> TableIdent {
        TableIdent::Ident(Ident::new_static(self))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test<T>(_: T)
    where
        T: IntoColumns
    {
    }

    #[test]
    fn test_into_columns() {
        test("hello");
        test(String::from("hello"));
        test(Ident::new("test?"));
        test(["hello"]);
        test([TableIdent::Ident(Ident::new_static("bob")), TableIdent::Raw(Raw::new_static("test"))]);
    }
}
