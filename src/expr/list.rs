use crate::{expr::TakeBindings, writer::FormatWriter, Binds, Builder, IntoBinds};

#[derive(Debug, Clone)]
pub enum InList {
    Binds(Binds),
    Subquery(Box<Builder>),
}

impl TakeBindings for InList {
    fn take_bindings(&mut self) -> Binds {
        match self {
            InList::Binds(array) => array.take_bindings(),
            InList::Subquery(builder) => builder.take_bindings(),
        }
    }
}

pub trait IntoInList {
    fn into_in_list(self) -> InList;
}

impl IntoInList for Builder {
    fn into_in_list(self) -> InList {
        InList::Subquery(Box::new(self))
    }
}

impl<T> IntoInList for T
where
    T: IntoBinds,
{
    fn into_in_list(self) -> InList {
        InList::Binds(self.into_binds())
    }
}

impl FormatWriter for InList {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            InList::Binds(array) => array.format_writer(context),
            InList::Subquery(builder) => builder.format_writer(context),
        }
    }
}
