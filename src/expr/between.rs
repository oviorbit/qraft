use std::fmt::Write;

use crate::writer::{FormatContext, FormatWriter};

use super::{Expr, TakeBindings};

#[derive(Debug, Clone, Copy)]
pub enum BetweenOperator {
    Between,
    NotBetween,
}

#[derive(Debug, Clone)]
pub struct BetweenCondition {
    pub(crate) lhs: Expr,
    pub(crate) low: Expr,
    pub(crate) high: Expr,
    pub(crate) operator: BetweenOperator,
}

impl TakeBindings for BetweenCondition {
    fn take_bindings(&mut self) -> crate::Binds {
        let mut binds = self.lhs.take_bindings();
        binds.append(self.low.take_bindings());
        binds.append(self.high.take_bindings());
        binds
    }
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
