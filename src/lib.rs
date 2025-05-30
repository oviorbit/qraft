#![allow(dead_code)]

mod bind;
mod builder;
mod col;
mod dialect;
pub mod expr;
mod ident;
mod raw;
mod scalar;
mod set;
mod writer;

use bind::Bind;
pub use col::ColumnSchema;
pub use col::Columns;
pub use col::IntoColumns;
pub use col::IntoTable;
pub use col::TableSchema;

pub use bind::Binds;
pub use bind::IntoBind;
pub use bind::IntoBinds;

pub use ident::Ident;
pub use ident::TableIdent;
pub use raw::IntoRaw;
pub use raw::Raw;

pub use builder::Builder;

pub use scalar::IntoOperator;
pub use scalar::IntoScalar;
pub use scalar::IntoScalarIdent;

pub use set::IntoSet;

pub fn column_static(value: &'static str) -> Ident {
    Ident::new_static(value)
}

pub fn column(value: &str) -> Ident {
    Ident::new(value)
}

pub fn value_static(value: &'static str) -> Bind {
    Bind::new_static_str(value)
}

pub fn value<V: IntoBind>(value: V) -> Bind {
    Bind::new(value)
}

pub fn raw_static(value: &'static str) -> Raw {
    Raw::new_static(value)
}

pub fn raw(value: &str) -> Raw {
    Raw::new(value)
}

pub fn sub<F>(query: F) -> Builder
where
    F: FnOnce(&mut Builder),
{
    let mut builder = Builder::default();
    query(&mut builder);
    builder
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{dialect, writer};

    pub(crate) fn format_writer<W: writer::FormatWriter>(
        writer: W,
        dialect: dialect::Dialect,
    ) -> String {
        let mut str = String::new();
        let mut context = writer::FormatContext::new(&mut str, dialect);
        writer.format_writer(&mut context).unwrap();
        str
    }
}
