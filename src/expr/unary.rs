use std::fmt::Write;

use crate::{bind::Binds, writer::{FormatContext, FormatWriter}};

use super::{Expr, TakeBindings};

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Null,
    NotNull,
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct UnaryCondition {
    pub(crate) lhs: Expr,
    pub(crate) operator: UnaryOperator,
}

impl TakeBindings for UnaryCondition {
    fn take_bindings(&mut self) -> Binds {
        self.lhs.take_bindings()
    }
}

impl FormatWriter for UnaryCondition {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.lhs.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.operator.format_writer(context)
    }
}

impl FormatWriter for UnaryOperator {
    fn format_writer<W: Write>(&self, context: &mut FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            UnaryOperator::Null => context.writer.write_str("is null"),
            UnaryOperator::NotNull => context.writer.write_str("is not null"),
            // Supported with mysql 8.0+ and of course postgres sqlite
            UnaryOperator::True => context.writer.write_str("is true"),
            UnaryOperator::False => context.writer.write_str("is false"),
        }
    }
}
