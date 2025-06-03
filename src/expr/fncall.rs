use crate::{Ident, writer::FormatWriter};

#[derive(Debug, Copy, Clone)]
pub enum Aggregate {
    Avg,
    Sum,
    Max,
    Min,
}

impl FormatWriter for Aggregate {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            Aggregate::Avg => context.writer.write_str("avg("),
            Aggregate::Sum => context.writer.write_str("sum("),
            Aggregate::Max => context.writer.write_str("max("),
            Aggregate::Min => context.writer.write_str("min("),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AggregateCall {
    agg: Aggregate,
    column: Ident,
    alias: Option<Ident>,
}

impl AggregateCall {
    pub fn new(agg: Aggregate, column: Ident, alias: Option<Ident>) -> Self {
        Self { agg, column, alias }
    }
}

impl FormatWriter for AggregateCall {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.agg.format_writer(context)?;
        self.column.format_writer(context)?;
        context.writer.write_char(')')?;
        context.write_alias(self.alias.as_ref())?;
        Ok(())
    }
}
