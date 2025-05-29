mod dialect;
mod writer;
mod ident;
mod operator;
mod raw;
mod builder;
pub mod col;

pub use ident::TableIdent;
pub use ident::Ident;
pub use raw::Raw;

pub use builder::Builder;

pub fn ident(value: &'static str) -> Ident {
    Ident::new_static(value)
}

pub fn ident_by_ref(value: &str) -> Ident {
    Ident::new(value)
}

pub fn raw(value: &'static str) -> Raw {
    Raw::new_static(value)
}

pub fn raw_by_ref(value: &str) -> Raw {
    Raw::new(value)
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

#[macro_export]
macro_rules! columns {
    () => {
        []
    };
    ( $($col:expr),+ $(,)? ) => {
        [$( $col.into_table_ident() ),+]
    };
}
