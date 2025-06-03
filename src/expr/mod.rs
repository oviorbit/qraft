use crate::{Raw, writer::FormatWriter};

pub(crate) mod between;
pub(crate) mod binary;
pub(crate) mod cond;
pub(crate) mod exists;
pub(crate) mod fncall;
pub(crate) mod group;
pub(crate) mod r#in;
pub(crate) mod list;
pub(crate) mod order;
pub(crate) mod sub;
pub(crate) mod unary;

use between::{BetweenCondition, BetweenOperator};
use binary::BinaryCondition;
pub use cond::Conjunction;
use exists::ExistsExpr;
use fncall::AggregateCall;
use r#in::InExpr;
use qraft_derive::variant;
use unary::{UnaryCondition, UnaryOperator};

use crate::{
    Binds, Builder, Ident, IntoBind, IntoTable, TableRef,
    bind::{Array, Bind},
    expr::binary::Operator,
};

// expr should be <= 64 bytes
#[derive(Debug, Clone)]
pub enum Expr {
    Bind(Bind),
    Ident(TableRef),
    Subquery(Box<Builder>),
    Exists(ExistsExpr),
    In(Box<InExpr>),
    AggregateCall(AggregateCall),
    Binary(Box<BinaryCondition>),
    Unary(Box<UnaryCondition>),
    Between(Box<BetweenCondition>),
}

impl Expr {
    #[variant(
        none, Operator, Eq, not_eq, gt, lt, gte, lte, like, not_like, ilike, not_ilike
    )]
    pub fn eq<R>(self, other: R) -> Self
    where
        R: IntoRhsExpr,
    {
        let rhs_expr = other.into_rhs_expr();
        let bin = BinaryCondition {
            lhs: self,
            operator: Operator::Eq,
            rhs: rhs_expr,
        };
        Expr::Binary(Box::new(bin))
    }

    #[variant(none, UnaryOperator, Null, is_not_null NotNull, is_true True, is_false False)]
    pub fn is_null<R>(self) -> Self {
        let unary = UnaryCondition {
            lhs: self,
            operator: UnaryOperator::Null,
        };
        Expr::Unary(Box::new(unary))
    }

    #[variant(none, BetweenOperator, Between, not_between)]
    fn between<L, H>(self, low: L, high: H) -> Self
    where
        L: IntoRhsExpr,
        H: IntoRhsExpr,
    {
        let low = low.into_rhs_expr();
        let high = high.into_rhs_expr();
        let btw = BetweenCondition {
            lhs: self,
            low,
            high,
            operator: BetweenOperator::Between,
        };
        Expr::Between(Box::new(btw))
    }
}

pub(crate) trait TakeBindings {
    fn take_bindings(&mut self) -> Binds;
}

impl TakeBindings for Expr {
    fn take_bindings(&mut self) -> Binds {
        match self {
            Expr::Bind(bind) => Array::One(std::mem::replace(bind, Bind::Consumed)),
            Expr::Ident(ident) => ident.take_bindings(),
            Expr::Subquery(builder) => builder.take_bindings(),
            Expr::Exists(condition) => condition.take_bindings(),
            Expr::In(condition) => condition.take_bindings(),
            Expr::AggregateCall(_) => Binds::None,
            Expr::Binary(condition) => condition.take_bindings(),
            Expr::Unary(condition) => condition.take_bindings(),
            Expr::Between(condition) => condition.take_bindings(),
        }
    }
}

impl FormatWriter for Expr {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            Expr::Bind(_) => context.write_placeholder(),
            Expr::Ident(ident) => ident.format_writer(context),
            Expr::Subquery(builder) => {
                context.writer.write_char('(')?;
                builder.format_writer(context)?;
                context.writer.write_char(')')
            }
            Expr::Exists(condition) => condition.format_writer(context),
            Expr::In(condition) => condition.format_writer(context),
            Expr::AggregateCall(aggregate) => aggregate.format_writer(context),
            Expr::Binary(condition) => condition.format_writer(context),
            Expr::Unary(condition) => condition.format_writer(context),
            Expr::Between(condition) => condition.format_writer(context)
        }
    }
}

pub trait IntoRhsExpr {
    fn into_rhs_expr(self) -> Expr;
}

pub trait IntoLhsExpr {
    fn into_lhs_expr(self) -> Expr;
}

impl IntoRhsExpr for Expr {
    fn into_rhs_expr(self) -> Expr {
        self
    }
}

impl IntoLhsExpr for Expr {
    fn into_lhs_expr(self) -> Expr {
        self
    }
}

pub trait IntoOperator {
    fn into_operator(self) -> Operator;
}

impl IntoOperator for Operator {
    fn into_operator(self) -> Operator {
        self
    }
}

impl IntoOperator for char {
    fn into_operator(self) -> Operator {
        match self {
            '=' => Operator::Eq,
            '>' => Operator::Gt,
            '<' => Operator::Lt,
            other => {
                debug_assert!(false, "Invalid operator char in Builder {:?}", other);
                tracing::warn!("Invalid operator '{:?}', defaulting to =", other);
                Operator::Eq
            }
        }
    }
}

impl IntoOperator for &'static str {
    fn into_operator(self) -> Operator {
        match self.to_ascii_lowercase().as_str() {
            "=" => Operator::Eq,
            "!=" | "<>" => Operator::NotEq,
            "<" => Operator::Lt,
            "<=" => Operator::Lte,
            ">" => Operator::Gt,
            ">=" => Operator::Gte,
            "like" => Operator::Like,
            "not like" => Operator::NotLike,
            "ilike" => Operator::Ilike,
            "not ilike" => Operator::NotIlike,
            other => {
                debug_assert!(false, "Invalid operator string in Builder {:?}", other);
                tracing::warn!("Invalid operator \"{:?}\", defaulting to =", other);
                Operator::Eq
            }
        }
    }
}

// maybe prevent the column-like identifier for blanket impl
impl<T> IntoRhsExpr for T
where
    T: IntoBind,
{
    fn into_rhs_expr(self) -> Expr {
        Expr::Bind(self.into_bind())
    }
}

impl IntoRhsExpr for Builder {
    fn into_rhs_expr(self) -> Expr {
        Expr::Subquery(Box::new(self))
    }
}

impl IntoRhsExpr for Raw {
    fn into_rhs_expr(self) -> Expr {
        Expr::Ident(TableRef::Raw(self))
    }
}

impl IntoRhsExpr for Ident {
    fn into_rhs_expr(self) -> Expr {
        Expr::Ident(TableRef::Ident(self))
    }
}

// impl for into scalar ident
impl<T> IntoLhsExpr for T
where
    T: IntoTable,
{
    fn into_lhs_expr(self) -> Expr {
        Expr::Ident(self.into_table())
    }
}

impl IntoLhsExpr for Builder {
    fn into_lhs_expr(self) -> Expr {
        Expr::Subquery(Box::new(self))
    }
}
