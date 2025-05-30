use crate::{bind::{Array, Bind}, operator::Operator, writer::FormatWriter, Binds, Builder, Ident, IntoBind, IntoTable, Raw, TableIdent};

// scalar should be <= 32 bytes
#[derive(Debug, Clone)]
pub enum ScalarExpr {
    Bind(Bind),
    Ident(TableIdent),
    Subquery(Box<Builder>),
}

pub trait TakeBindings {
    fn take_bindings(&mut self) -> Binds;
}

impl TakeBindings for ScalarExpr {
    fn take_bindings(&mut self) -> Binds {
        match self {
            ScalarExpr::Bind(bind) => Array::One(std::mem::replace(bind, Bind::Consumed)),
            ScalarExpr::Ident(ident) => ident.take_bindings(),
            ScalarExpr::Subquery(builder) => builder.take_bindings()
        }
    }
}

impl FormatWriter for ScalarExpr {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        match self {
            ScalarExpr::Bind(_) => context.write_placeholder(),
            ScalarExpr::Ident(ident) => ident.format_writer(context),
            ScalarExpr::Subquery(builder) => {
                context.writer.write_char('(')?;
                builder.format_writer(context)?;
                context.writer.write_char(')')
            }
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ScalarIdent(pub(crate) ScalarExpr);

#[derive(Debug)]
#[repr(transparent)]
pub struct Scalar(pub(crate) ScalarExpr);

pub trait IntoScalar {
    fn into_scalar(self) -> Scalar;
}

pub trait IntoScalarIdent {
    fn into_scalar_ident(self) -> ScalarIdent;
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
impl<T> IntoScalar for T
where
    T: IntoBind
{
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpr::Bind(self.into_bind()))
    }
}

impl IntoScalar for Builder {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpr::Subquery(Box::new(self)))
    }
}

impl IntoScalar for Raw {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpr::Ident(TableIdent::Raw(self)))
    }
}

impl IntoScalar for Ident {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpr::Ident(TableIdent::Ident(self)))
    }
}

// impl for into scalar ident
impl<T> IntoScalarIdent for T
where
    T: IntoTable
{
    fn into_scalar_ident(self) -> ScalarIdent {
        ScalarIdent(ScalarExpr::Ident(self.into_table()))
    }
}

impl IntoScalarIdent for Builder {
    fn into_scalar_ident(self) -> ScalarIdent {
        ScalarIdent(ScalarExpr::Subquery(Box::new(self)))
    }
}
