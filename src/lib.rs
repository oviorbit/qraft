#![allow(dead_code)]

mod dialect;
mod writer;
mod ident;
mod operator;
mod raw;
mod builder;
mod col;
mod bind;

pub use col::TableSchema;
pub use col::ColumnSchema;
pub use col::Columns;
pub use col::IntoColumns;
pub use col::IntoTable;

pub use ident::TableIdent;
pub use ident::Ident;
pub use raw::Raw;

pub use builder::Builder;

pub fn ident_static(value: &'static str) -> Ident {
    Ident::new_static(value)
}

pub fn ident(value: &str) -> Ident {
    Ident::new(value)
}

pub fn raw_static(value: &'static str) -> Raw {
    Raw::new_static(value)
}

pub fn raw(value: &str) -> Raw {
    Raw::new(value)
}

#[macro_export]
macro_rules! col {
    () => {
        []
    };
    ( $($col:expr),+ $(,)? ) => {
        [$( $col.into_table() ),+]
    };
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{dialect, writer};

    pub(crate) fn format_writer<W: writer::FormatWriter>(writer: W, dialect: dialect::Dialect) -> String {
        let mut str = String::new();
        let mut context = writer::FormatContext::new(&mut str, dialect);
        writer.format_writer(&mut context).unwrap();
        str
    }
}
