use crate::{expr::ConditionKind, writer::FormatWriter};

#[derive(Debug, Clone, Copy)]
pub enum Conjunction {
    And,
    Or,
    AndNot,
    OrNot,
}

impl FormatWriter for Conjunction {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            Conjunction::And => context.writer.write_str("and"),
            Conjunction::Or => context.writer.write_str("or"),
            Conjunction::AndNot => context.writer.write_str("and not"),
            Conjunction::OrNot => context.writer.write_str("or not"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    conjunction: Conjunction,
    kind: ConditionKind,
}

impl FormatWriter for Condition {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        self.kind.format_writer(context)
    }
}

impl Condition {
    pub fn new(conjunction: Conjunction, kind: ConditionKind) -> Self {
        Self { conjunction, kind }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Conditions(pub(crate) Vec<Condition>);

impl Conditions {
    pub fn push(&mut self, other: Condition) {
        self.0.push(other);
    }
}

impl FormatWriter for Conditions {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        for (index, condition) in self.0.iter().enumerate() {
            if index > 0 {
                context.writer.write_char(' ')?;
                condition.conjunction.format_writer(context)?;
                context.writer.write_char(' ')?;
            }
            condition.format_writer(context)?;
        }
        Ok(())
    }
}
