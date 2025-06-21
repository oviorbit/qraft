use std::{borrow::Cow, sync::Arc};

use smol_str::SmolStr;

use crate::{
    bind::{Array, Binds},
    col::AliasSub,
    expr::TakeBindings,
    raw::Raw,
    writer::{self, FormatWriter},
};

#[derive(Debug, Clone)]
pub enum TableRef {
    Ident(Ident),
    Raw(Raw),
    AliasSub(AliasSub),
}

#[derive(Debug, Clone)]
pub enum RawOrIdent {
    Ident(Ident),
    Raw(Raw),
}

impl RawOrIdent {
    pub fn table_name(&self) -> &str {
        match self {
            Self::Ident(ident) => {
                let res = split_alias(ident.0.as_str());
                res.1.unwrap_or(res.0)
            }
            Self::Raw(raw) => raw.0.as_str(),
        }
    }
}

impl Default for TableRef {
    fn default() -> Self {
        TableRef::Ident(Ident::new_static(""))
    }
}

impl TakeBindings for TableRef {
    fn take_bindings(&mut self) -> Binds {
        match self {
            TableRef::Ident(_) => Array::None,
            TableRef::Raw(_) => Array::None,
            TableRef::AliasSub(builder) => builder.take_bindings(),
        }
    }
}

impl TableRef {
    pub fn ident_static(value: &'static str) -> Self {
        Self::Ident(Ident::new_static(value))
    }

    pub fn ident<T>(value: T) -> Self
    where
        T: Into<SmolStr>,
    {
        Self::Ident(Ident::new(value))
    }

    pub fn raw<T>(value: T) -> Self
    where
        T: Into<SmolStr>,
    {
        Self::Raw(Raw::new(value))
    }

    pub fn raw_static(value: &'static str) -> Self {
        Self::Raw(Raw::new_static(value))
    }

    pub fn table_name(&self) -> &str {
        match self {
            TableRef::Ident(ident) => {
                let res = split_alias(ident.0.as_str());
                res.1.unwrap_or(res.0)
            }
            TableRef::Raw(raw) => raw.0.as_str(),
            TableRef::AliasSub(alias) => alias.alias.0.as_str(),
        }
    }
}

impl FormatWriter for TableRef {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            TableRef::Ident(ident) => ident.format_writer(context),
            TableRef::Raw(raw) => raw.format_writer(context),
            TableRef::AliasSub(builder) => builder.format_writer(context),
        }
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident(SmolStr);

pub trait IntoIdent {
    fn into_ident(self) -> Ident;
}

impl IntoIdent for Ident {
    fn into_ident(self) -> Ident {
        self
    }
}

impl IntoIdent for char {
    fn into_ident(self) -> Ident {
        Ident::new(smol_str::format_smolstr!("{}", self))
    }
}

impl IntoIdent for &str {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for &mut str {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for &String {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for String {
    #[inline(always)]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for Box<str> {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for Arc<str> {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl<'a> IntoIdent for Cow<'a, str> {
    #[inline]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

impl IntoIdent for SmolStr {
    #[inline(always)]
    fn into_ident(self) -> Ident {
        Ident::new(self)
    }
}

pub fn split_alias(s: &str) -> (&str, Option<&str>) {
    if let Some(idx) = find_as(s.as_bytes()) {
        let left = &s[..idx];
        let right = &s[idx + 4..];
        (left, Some(right))
    } else {
        (s, None)
    }
}

impl Ident {
    #[inline]
    pub fn new<T>(value: T) -> Self
    where
        T: Into<SmolStr>,
    {
        Self(value.into())
    }

    #[inline]
    pub fn new_static(value: &'static str) -> Self {
        Self(SmolStr::new_static(value))
    }

    pub fn split_alias(&self) -> (Ident, Option<Ident>) {
        let s = self.0.as_str();
        if let Some(idx) = find_as(s.as_bytes()) {
            let left = &s[..idx];
            let right = &s[idx + 4..];
            (Ident::new(left), Some(Ident::new(right)))
        } else {
            (self.clone(), None)
        }
    }
}

impl FormatWriter for Ident {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        let table = self.0.as_str();
        if let Some(index) = find_as(table.as_bytes()) {
            let (lhs, rhs) = table.split_at(index);
            let alias = &rhs[4..];
            context.write_table(lhs)?;
            context.writer.write_str(" as ")?;
            context.write_ident(alias)?;
            return Ok(());
        }

        context.write_table(table)?;
        Ok(())
    }
}

/// Return the index of " as " in bytes case insensitive with no allocations.
fn find_as(h: &[u8]) -> Option<usize> {
    if h.len() < 4 {
        return None;
    }
    for (i, w) in h.windows(4).enumerate() {
        if w[0] == b' ' && w[3] == b' ' && (w[1] | 0x20) == b'a' && (w[2] | 0x20) == b's' {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::{dialect::Dialect, tests::format_writer};

    use super::*;

    #[test]
    fn test_find_as() {
        let matches = "users as u";
        let index = find_as(matches.as_bytes());
        assert_eq!(index, Some(5));
        let no_match = "users";
        let index = find_as(no_match.as_bytes());
        assert_eq!(index, None);
        let first_match = "users as u as bob";
        let index = find_as(first_match.as_bytes());
        assert_eq!(index, Some(5));
    }

    #[test]
    fn test_format_ident_simple() {
        let ident = Ident::new_static("users");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"users\"", ident);
        let ident = Ident::new_static("users");
        let ident = format_writer(ident, Dialect::MySql);
        assert_eq!("`users`", ident)
    }

    #[test]
    fn test_format_writer_spaces() {
        let ident = Ident::new_static("an sql table");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"an sql table\"", ident);
    }

    #[test]
    fn test_format_writer_alias() {
        let ident = Ident::new_static("users as foo");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"users\" as \"foo\"", ident);
    }

    #[test]
    fn test_format_writer_quote() {
        let ident = Ident::new_static("us\"ers");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"us\"\"ers\"", ident);
        let ident = Ident::new_static("us`ers");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"us`ers\"", ident);
        let ident = Ident::new_static("us`ers");
        let ident = format_writer(ident, Dialect::MySql);
        assert_eq!("`us``ers`", ident);
    }

    #[test]
    fn test_format_writer_dot() {
        let ident = Ident::new_static("x.y");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"x\".\"y\"", ident);
        let ident = Ident::new_static("x.y");
        let ident = format_writer(ident, Dialect::MySql);
        assert_eq!("`x`.`y`", ident);
    }

    #[test]
    fn test_format_writer_space_dot() {
        let ident = Ident::new_static("some space.x.y as some.table");
        let ident = format_writer(ident, Dialect::Postgres);
        assert_eq!("\"some space\".\"x\".\"y\" as \"some.table\"", ident);
        let ident = Ident::new_static("some space.x.y as some.table");
        let ident = format_writer(ident, Dialect::MySql);
        assert_eq!("`some space`.`x`.`y` as `some.table`", ident);
    }
}
