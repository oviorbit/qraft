mod bind;
mod builder;
mod col;
mod dialect;
pub mod expr;
mod ident;
mod insert;
mod join;
mod raw;
mod writer;

use bind::Bind;
use col::AliasSub;
pub use col::IntoGroupProj;
pub use col::IntoTable;
pub use col::ProjectionSchema;
pub use col::Projections;
pub use col::TableSchema;
use expr::sub::AliasSubFn;
use expr::Expr;
use ident::IntoIdent;
pub use join::*;

pub use bind::Binds;
pub use bind::IntoBind;
pub use bind::IntoBinds;

pub use ident::Ident;
pub use ident::TableRef;
pub use raw::IntoRaw;
pub use raw::Raw;

pub use builder::Builder;

pub use expr::IntoLhsExpr;
pub use expr::IntoOperator;
pub use expr::IntoRhsExpr;

pub use expr::list::IntoInList;

pub use dialect::*;
use smol_str::SmolStr;

// Can use { ... } instead of |builder| ...

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
