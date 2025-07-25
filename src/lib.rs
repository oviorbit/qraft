pub mod bind;
mod builder;
pub mod col;
pub mod dialect;
pub mod expr;
pub mod ident;
mod insert;
pub mod join;
pub mod raw;
pub mod writer;
pub mod row;

pub use builder::Builder;
pub use insert::InsertBuilder;
pub use row::Row;

use bind::{Bind, IntoBind};
use col::AliasSub;
use expr::sub::AliasSubFn;
use expr::Expr;
use ident::{Ident, IntoIdent};
use raw::Raw;
use smol_str::SmolStr;

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

pub fn lit(value: &'static str) -> Raw {
    Raw::new(smol_str::format_smolstr!("'{}'", value))
}

pub fn sub_as<F, I>(table: F, alias: I) -> AliasSub
where
    F: FnOnce(&mut Builder),
    I: IntoIdent,
{
    let mut inner = Builder::default();
    table(&mut inner);
    AliasSub::new(inner, alias)
}

pub fn fn_sub_as<T, F, A>(keyword: T, subquery: F, alias: A) -> AliasSubFn
where
    T: Into<SmolStr>,
    F: FnOnce(&mut Builder),
    A: IntoIdent,
{
    let mut builder = Builder::default();
    subquery(&mut builder);
    AliasSubFn::new(keyword, builder, alias)
}

pub fn sub<F>(query: F) -> Expr
where
    F: FnOnce(&mut Builder),
{
    let mut builder = Builder::default();
    query(&mut builder);
    Expr::Subquery(Box::new(builder))
}

#[macro_export]
macro_rules! row {
    ( $( $key:ident => $val:expr ),* $(,)? ) => {{
        let mut __row = $crate::Row::new();
        $(
            __row.field(stringify!($key), $val);
        )*
        __row
    }};
    ( $( $key:expr => $val:expr ),* $(,)? ) => {{
        let mut __row = $crate::Row::new();
        $(
            __row.field($key, $val);
        )*
        __row
    }};
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
