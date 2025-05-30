use crate::{expr::ConditionKind, writer::FormatWriter};

#[derive(Debug, Clone, Copy)]
pub enum LogicalOperator {
    And,
    Or,
}

impl FormatWriter for LogicalOperator {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            LogicalOperator::And => context.writer.write_str("and"),
            LogicalOperator::Or => context.writer.write_str("or"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    logic: LogicalOperator,
    kind: ConditionKind,
}

impl FormatWriter for Condition {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        self.kind.format_writer(context)
    }
}

impl Condition {
    pub fn new(logic: LogicalOperator, kind: ConditionKind) -> Self {
        Self {
            logic,
            kind,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Conditions(pub(crate) Vec<Condition>);

impl FormatWriter for Conditions {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        for (index, condition) in self.0.iter().enumerate() {
            if index > 0 {
                context.writer.write_char(' ')?;
                condition.logic.format_writer(context)?;
                context.writer.write_char(' ')?;
            }
            condition.format_writer(context)?;
        }
        Ok(())
    }
}
