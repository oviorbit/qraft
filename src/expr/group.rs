use crate::writer::FormatWriter;

use super::cond::Conditions;

#[derive(Debug, Clone)]
pub struct GroupCondition {
    pub(crate) conditions: Conditions,
}

impl FormatWriter for GroupCondition {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        context.writer.write_char('(')?;
        self.conditions.format_writer(context)?;
        context.writer.write_char(')')?;
        Ok(())
    }
}

