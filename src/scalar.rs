use crate::{bind::Bind, Builder, Ident, IntoBind, IntoTable, Raw, TableIdent};

// scalar should be <= 32 bytes
#[derive(Debug)]
pub enum ScalarExpression {
    Bind(Bind),
    Ident(TableIdent),
    Subquery(Box<Builder>),
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ScalarIdent(ScalarExpression);

#[derive(Debug)]
#[repr(transparent)]
pub struct Scalar(ScalarExpression);

pub trait IntoScalar {
    fn into_scalar(self) -> Scalar;
}

pub trait IntoScalarIdent {
    fn into_scalar_ident(self) -> ScalarIdent;
}

// maybe prevent the column-like identifier for blanket impl
impl<T> IntoScalar for T
where
    T: IntoBind
{
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpression::Bind(self.into_bind()))
    }
}

impl IntoScalar for Builder {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpression::Subquery(Box::new(self)))
    }
}

impl IntoScalar for Raw {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpression::Ident(TableIdent::Raw(self)))
    }
}

impl IntoScalar for Ident {
    fn into_scalar(self) -> Scalar {
        Scalar(ScalarExpression::Ident(TableIdent::Ident(self)))
    }
}

// impl for into scalar ident
impl<T> IntoScalarIdent for T
where
    T: IntoTable
{
    fn into_scalar_ident(self) -> ScalarIdent {
        ScalarIdent(ScalarExpression::Ident(self.into_table()))
    }
}

impl IntoScalarIdent for Builder {
    fn into_scalar_ident(self) -> ScalarIdent {
        ScalarIdent(ScalarExpression::Subquery(Box::new(self)))
    }
}
