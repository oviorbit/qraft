use crate::{bind::Binds, writer::FormatWriter, Builder, Raw};

use super::{
    Expr, TakeBindings,
    between::{BetweenCondition, BetweenOperator},
    binary::{BinaryCondition, Operator},
    exists::{ExistsExpr, ExistsOperator},
    group::GroupCondition,
    r#in::{InExpr, InOperator},
    list::InList,
    unary::{UnaryCondition, UnaryOperator},
};

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
pub enum ConditionKind {
    Group(GroupCondition),
    Raw(Raw),
    Binary(BinaryCondition),
    Unary(UnaryCondition),
    Between(BetweenCondition),
    In(InExpr),
    Exists(ExistsExpr),
}

impl TakeBindings for ConditionKind {
    fn take_bindings(&mut self) -> Binds {
        match self {
            ConditionKind::Binary(condition) => condition.take_bindings(),
            ConditionKind::Group(condition) => condition.take_bindings(),
            ConditionKind::Raw(_) => Binds::None,
            ConditionKind::Unary(condition) => condition.take_bindings(),
            ConditionKind::Between(condition) => condition.take_bindings(),
            ConditionKind::In(condition) => condition.take_bindings(),
            ConditionKind::Exists(condition) => condition.take_bindings(),
        }
    }
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

impl TakeBindings for Condition {
    fn take_bindings(&mut self) -> Binds {
        self.kind.take_bindings()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Conditions(pub(crate) Vec<Condition>);

impl TakeBindings for Conditions {
    fn take_bindings(&mut self) -> Binds {
        self.0
            .iter_mut()
            .map(|v| v.take_bindings())
            .fold(Binds::None, |mut acc, next| {
                acc.append(next);
                acc
            })
    }
}

impl Conditions {
    pub fn push(&mut self, other: Condition) {
        self.0.push(other);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push_unary(&mut self, conjunction: Conjunction, lhs: Expr, operator: UnaryOperator) {
        let cond = UnaryCondition { lhs, operator };
        let kind = ConditionKind::Unary(cond);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
    }

    pub fn push_in(
        &mut self,
        conjunction: Conjunction,
        lhs: Expr,
        rhs: InList,
        operator: InOperator,
    ) {
        let inc = InExpr::new(operator, lhs, rhs, None);
        let kind = ConditionKind::In(inc);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
    }

    pub fn push_exists(
        &mut self,
        conjunction: Conjunction,
        rhs: Builder,
        operator: ExistsOperator,
    ) {
        let exists = ExistsExpr::new(operator, rhs, None);
        let kind = ConditionKind::Exists(exists);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
    }

    pub fn push_group(&mut self, conjunction: Conjunction, conditions: Conditions) {
        let group = GroupCondition { conditions };
        let kind = ConditionKind::Group(group);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
    }

    pub fn push_binary(
        &mut self,
        conjunction: Conjunction,
        lhs: Expr,
        rhs: Expr,
        operator: Operator,
    ) {
        let binary = BinaryCondition { lhs, operator, rhs };
        let kind = ConditionKind::Binary(binary);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
    }

    pub fn push_between(
        &mut self,
        conjunction: Conjunction,
        lhs: Expr,
        low: Expr,
        high: Expr,
        operator: BetweenOperator,
    ) {
        let cond = BetweenCondition {
            lhs,
            low,
            high,
            operator,
        };
        let kind = ConditionKind::Between(cond);
        self.push(Condition::new(conjunction, kind));
    }

    pub fn push_raw(&mut self, conjunction: Conjunction, raw: Raw) {
        let kind = ConditionKind::Raw(raw);
        let cond = Condition::new(conjunction, kind);
        self.push(cond);
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
