use cond::{Conditions, Conjunction};

use crate::{dialect::Dialect, operator::Operator, scalar::ScalarExpr, writer::FormatWriter};

pub mod cond;

#[derive(Debug, Clone)]
pub struct Binary {
    pub(crate) lhs: ScalarExpr,
    pub(crate) operator: Operator,
    pub(crate) rhs: ScalarExpr,
}

#[derive(Debug, Clone)]
pub enum ConditionKind {
    Binary(Binary),
    Group(Group),
}

#[derive(Debug, Clone)]
pub struct Group {
    pub(crate) conditions: Conditions,
    pub(crate) conjunction: Conjunction,
}

impl FormatWriter for Group {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        context.writer.write_char('(')?;
        self.conditions.format_writer(context)?;
        context.writer.write_char(')')?;
        Ok(())
    }
}

impl FormatWriter for Binary {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.lhs.format_writer(context)?;
        if let (
            Dialect::Postgres,
            Operator::Like | Operator::Ilike | Operator::NotLike | Operator::NotIlike,
        ) = (context.dialect, self.operator)
        {
           context.writer.write_str("::text")?;
        };
        context.writer.write_char(' ')?;
        self.operator.format_writer(context)?;
        context.writer.write_char(' ')?;
        self.rhs.format_writer(context)
    }
}

impl FormatWriter for ConditionKind {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            ConditionKind::Binary(binary) => binary.format_writer(context),
            ConditionKind::Group(group) => group.format_writer(context),
        }
    }
}
