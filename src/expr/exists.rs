use std::fmt::Write;

use crate::{writer::{FormatContext, FormatWriter}, Builder};

#[derive(Debug, Clone)]
pub struct ExistsCondition {
    pub(crate) operator: ExistsOperator,
    pub(crate) subquery: Box<Builder>,
}

impl ExistsCondition {
    pub fn new(operator: ExistsOperator, subquery: Builder) -> Self {
        Self {
            operator,
            subquery: Box::new(subquery),
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

impl FormatWriter for ExistsCondition {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        self.operator.format_writer(context)?;
        context.writer.write_str(" (")?;
        self.subquery.format_writer(context)?;
        context.writer.write_char(')')?;
        Ok(())
    }
}
