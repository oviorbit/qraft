use crate::{dialect::Dialect, operator::Operator, scalar::ScalarExpr, writer::FormatWriter};

#[derive(Debug, Clone)]
pub struct BinaryCondition {
    pub(crate) lhs: ScalarExpr,
    pub(crate) operator: Operator,
    pub(crate) rhs: ScalarExpr,
}

impl FormatWriter for BinaryCondition {
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

