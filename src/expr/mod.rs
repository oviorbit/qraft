use between::BetweenCondition;
use binary::BinaryCondition;
use exists::ExistsCondition;
use group::GroupCondition;
use r#in::InCondition;
use unary::UnaryCondition;

use crate::{Raw, writer::FormatWriter};

pub(crate) mod between;
pub(crate) mod binary;
pub(crate) mod cond;
pub(crate) mod exists;
pub(crate) mod group;
pub(crate) mod r#in;
pub(crate) mod unary;

pub use cond::Conjunction;

#[derive(Debug, Clone)]
pub enum ConditionKind {
    Binary(BinaryCondition),
    Group(GroupCondition),
    Raw(Raw),
    Unary(UnaryCondition),
    Between(BetweenCondition),
    In(InCondition),
    Exists(ExistsCondition),
}

impl FormatWriter for ConditionKind {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            ConditionKind::Binary(binary) => binary.format_writer(context),
            ConditionKind::Group(group) => group.format_writer(context),
            ConditionKind::Raw(raw) => raw.format_writer(context),
            ConditionKind::Unary(unary) => unary.format_writer(context),
            ConditionKind::Between(between) => between.format_writer(context),
            ConditionKind::In(inc) => inc.format_writer(context),
            ConditionKind::Exists(exists) => exists.format_writer(context),
        }
    }
}
