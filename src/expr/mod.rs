use crate::{Raw, writer::FormatWriter};

pub(crate) mod between;
pub(crate) mod binary;
pub(crate) mod cond;
pub(crate) mod exists;
pub(crate) mod group;
pub(crate) mod r#in;
pub(crate) mod list;
pub(crate) mod order;
pub(crate) mod unary;

pub use cond::Conjunction;

use crate::{
    Binds, Builder, Ident, IntoBind, IntoTable, TableRef,
    bind::{Array, Bind},
    expr::binary::Operator,
};

// expr should be <= 32 bytes
#[derive(Debug, Clone)]
pub enum Expr {
    Bind(Bind),
    Ident(TableRef),
    Subquery(Box<Builder>),
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
