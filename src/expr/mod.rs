use binary::BinaryCondition;
use group::GroupCondition;
use unary::UnaryCondition;

use crate::{writer::FormatWriter, Raw};

pub(crate) mod cond;
pub(crate) mod unary;
pub(crate) mod binary;
pub(crate) mod group;

pub use cond::Conjunction;

#[derive(Debug, Clone)]
pub enum ConditionKind {
    Binary(BinaryCondition),
    Group(GroupCondition),
    Raw(Raw),
    Unary(UnaryCondition),
}

impl FormatWriter for ConditionKind {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            ConditionKind::Binary(binary) => binary.format_writer(context),
            ConditionKind::Group(group) => group.format_writer(context),
            ConditionKind::Raw(raw) => raw.format_writer(context),
            ConditionKind::Unary(unary) => unary.format_writer(context),
        }
    }
}
