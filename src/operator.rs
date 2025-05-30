use qraft_derive::BinaryOperator;

use crate::{dialect::Dialect, writer::FormatWriter};

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
