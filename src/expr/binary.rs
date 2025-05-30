use qraft_derive::BinaryOperator;

use crate::{dialect::Dialect, scalar::ScalarExpr, writer::FormatWriter};

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


#[derive(Debug, Clone, Copy, BinaryOperator)]
pub enum Operator {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    NotLike,
    Ilike,
    NotIlike,
}

impl FormatWriter for Operator {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            Operator::Eq => context.writer.write_char('='),
            Operator::NotEq => context.writer.write_str("!="),
            Operator::Lt => context.writer.write_char('<'),
            Operator::Lte => context.writer.write_str("<="),
            Operator::Gt => context.writer.write_char('>'),
            Operator::Gte => context.writer.write_str(">="),
            Operator::Like => match context.dialect {
                Dialect::Postgres => context.writer.write_str("like"),
                Dialect::MySql => context.writer.write_str("like binary"),
                Dialect::Sqlite => context.writer.write_str("glob"),
            },
            Operator::NotLike => match context.dialect {
                Dialect::Postgres => context.writer.write_str("not like"),
                Dialect::MySql => context.writer.write_str("not like binary"),
                Dialect::Sqlite => context.writer.write_str("not glob"),
            },
            Operator::Ilike => match context.dialect {
                Dialect::Postgres => context.writer.write_str("ilike"),
                Dialect::MySql | Dialect::Sqlite => context.writer.write_str("like"),
            },
            Operator::NotIlike => match context.dialect {
                Dialect::Postgres => context.writer.write_str("not ilike"),
                Dialect::MySql | Dialect::Sqlite => context.writer.write_str("not like"),
            },
        }
    }
}
