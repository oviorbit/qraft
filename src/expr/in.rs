use qraft_derive::InOperator;

use crate::{scalar::ScalarExpr, set::SetExpr, writer::FormatWriter};

#[derive(Debug, Clone)]
pub struct InCondition {
    pub(crate) operator: InOperator,
    pub(crate) lhs: ScalarExpr,
    pub(crate) rhs: SetExpr,
}

#[derive(Debug, Clone, Copy, InOperator)]
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
