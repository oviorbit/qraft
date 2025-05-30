use crate::{Binds, Builder, IntoBinds, scalar::TakeBindings, writer::FormatWriter};

#[derive(Debug, Clone)]
pub enum SetExpr {
    Binds(Binds),
    Subquery(Box<Builder>),
}

impl TakeBindings for SetExpr {
    fn take_bindings(&mut self) -> Binds {
        match self {
            SetExpr::Binds(array) => array.take_bindings(),
            SetExpr::Subquery(builder) => builder.take_bindings(),
        }
    }
}

pub trait IntoSet {
    fn into_set(self) -> SetExpr;
}

impl IntoSet for Builder {
    fn into_set(self) -> SetExpr {
        SetExpr::Subquery(Box::new(self))
    }
}

impl<T> IntoSet for T
where
    T: IntoBinds,
{
    fn into_set(self) -> SetExpr {
        SetExpr::Binds(self.into_binds())
    }
}

impl FormatWriter for SetExpr {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            SetExpr::Binds(array) => array.format_writer(context),
            SetExpr::Subquery(builder) => builder.format_writer(context),
        }
    }
}
