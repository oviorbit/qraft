use crate::{writer::{self, FormatWriter}, Ident};

use super::{list::InList, Expr, TakeBindings};

#[derive(Debug, Clone)]
pub struct InExpr {
    pub(crate) operator: InOperator,
    pub(crate) lhs: Expr,
    pub(crate) rhs: InList,
    pub(crate) alias: Option<Ident>,
}

impl InExpr {
    pub fn new(operator: InOperator, lhs: Expr, rhs: InList, alias: Option<Ident>) -> Self {
        Self {
            operator,
            lhs,
            rhs,
            alias,
        }
    }
}

impl TakeBindings for InExpr {
    fn take_bindings(&mut self) -> crate::Binds {
        let mut binds = self.lhs.take_bindings();
        binds.append(self.rhs.take_bindings());
        binds
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InOperator {
    In,
    NotIn,
}

impl FormatWriter for InExpr {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.lhs.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.operator.format_writer(context)?;
        context.writer.write_str(" (")?;
        self.rhs.format_writer(context)?;
        context.writer.write_char(')')?;
        context.write_alias(self.alias.as_ref())?;
        Ok(())
    }
}

impl FormatWriter for InOperator {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            InOperator::In => context.writer.write_str("in"),
            InOperator::NotIn => context.writer.write_str("not in"),
        }
    }
}
