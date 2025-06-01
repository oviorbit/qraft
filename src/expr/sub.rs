use smol_str::SmolStr;

use crate::{ident::IntoIdent, writer::{self, FormatWriter}, Builder, Ident};

use super::TakeBindings;

#[derive(Debug, Clone)]
pub struct SubqueryFn {
    keyword: SmolStr,
    sub: Box<Builder>,
    maybe_alias: Option<Ident>,
}

impl TakeBindings for SubqueryFn {
    fn take_bindings(&mut self) -> crate::Binds {
        self.sub.take_bindings()
    }
}

impl SubqueryFn {
    pub fn new<I>(keyword: I, subquery: Builder) -> Self
    where
        I: Into<SmolStr>
    {
        Self {
            keyword: keyword.into(),
            sub: Box::new(subquery),
            maybe_alias: None,
        }
    }

    pub fn alias<A: IntoIdent>(mut self, alias: A) -> Self {
        self.maybe_alias = Some(alias.into_ident());
        self
    }
}

impl FormatWriter for SubqueryFn {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        context.writer.write_str(self.keyword.as_str())?;
        context.writer.write_char('(')?;
        self.sub.format_writer(context)?;
        context.writer.write_char(')')?;
        if let Some(ref alias) = self.maybe_alias {
            context.writer.write_str(" as ")?;
            alias.format_writer(context)?;
        }
        Ok(())
    }
}
