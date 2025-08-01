use std::fmt::Write;

use crate::{
    bind::Binds, writer::{FormatContext, FormatWriter}, Builder, Ident
};

use super::TakeBindings;

#[derive(Debug, Clone)]
pub struct ExistsExpr {
    pub(crate) operator: ExistsOperator,
    pub(crate) subquery: Box<Builder>,
    pub(crate) alias: Option<Ident>,
}

impl TakeBindings for ExistsExpr {
    fn take_bindings(&mut self) -> Binds {
        self.subquery.take_bindings()
    }
}

impl ExistsExpr {
    pub fn new(operator: ExistsOperator, subquery: Builder, alias: Option<Ident>) -> Self {
        Self {
            operator,
            subquery: Box::new(subquery),
            alias,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExistsOperator {
    Exists,
    NotExists,
}

impl FormatWriter for ExistsOperator {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            ExistsOperator::Exists => context.writer.write_str("exists"),
            ExistsOperator::NotExists => context.writer.write_str("not exists"),
        }
    }
}

impl FormatWriter for ExistsExpr {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        self.operator.format_writer(context)?;
        context.writer.write_str(" (")?;
        self.subquery.format_writer(context)?;
        context.writer.write_char(')')?;
        context.write_alias(self.alias.as_ref())?;
        Ok(())
    }
}
