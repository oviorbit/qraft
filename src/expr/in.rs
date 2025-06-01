use crate::writer::FormatWriter;

use super::{Expr, list::InList};

#[derive(Debug, Clone)]
pub struct InCondition {
    pub(crate) operator: InOperator,
    pub(crate) lhs: Expr,
    pub(crate) rhs: InList,
}

#[derive(Debug, Clone, Copy)]
pub enum InOperator {
    In,
    NotIn,
}

impl FormatWriter for InCondition {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.lhs.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.operator.format_writer(context)?;
        context.writer.write_str(" (")?;
        self.rhs.format_writer(context)?;
        context.writer.write_char(')')?;
        Ok(())
    }
}

impl FormatWriter for InOperator {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            InOperator::In => context.writer.write_str("in"),
            InOperator::NotIn => context.writer.write_str("not in"),
        }
    }
}
