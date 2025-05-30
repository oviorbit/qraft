use std::fmt::Write;

use crate::{scalar::ScalarExpr, writer::{FormatContext, FormatWriter}};

#[derive(Debug, Clone, Copy, qraft_derive::BetweenOperator)]
pub enum BetweenOperator {
    Between,
    NotBetween,
}

#[derive(Debug, Clone)]
pub struct BetweenCondition {
    pub(crate) lhs: ScalarExpr,
    pub(crate) low: ScalarExpr,
    pub(crate) high: ScalarExpr,
    pub(crate) operator: BetweenOperator,
}

impl FormatWriter for BetweenOperator {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            BetweenOperator::Between => context.writer.write_str("between"),
            BetweenOperator::NotBetween => context.writer.write_str("not between"),
        }
    }
}

impl FormatWriter for BetweenCondition {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        self.lhs.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.operator.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.low.format_writer(context)?;
        context.writer.write_str(" and ")?;
        self.high.format_writer(context)
    }
}
