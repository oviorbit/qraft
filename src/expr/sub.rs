use smol_str::SmolStr;

use crate::{
    Builder, Ident,
    col::AliasSub,
    ident::IntoIdent,
    writer::{self, FormatWriter},
};

use super::TakeBindings;

#[derive(Debug, Clone)]
pub struct AliasSubFn {
    keyword: SmolStr,
    inner: AliasSub,
}

impl TakeBindings for AliasSubFn {
    fn take_bindings(&mut self) -> crate::Binds {
        self.inner.take_bindings()
    }
}

impl AliasSubFn {
    pub fn new<I, T>(keyword: I, inner: Builder, alias: T) -> Self
    where
        I: Into<SmolStr>,
        T: IntoIdent,
    {
        Self {
            keyword: keyword.into(),
            inner: AliasSub::new(inner, alias),
        }
    }
}

impl FormatWriter for AliasSubFn {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        context.writer.write_str(self.keyword.as_str())?;
        self.inner.format_writer(context)?;
        Ok(())
    }
}
