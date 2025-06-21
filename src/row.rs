use indexmap::IndexMap;

use crate::{
    expr::{Expr, TakeBindings}, ident::IntoIdent, writer::FormatWriter, Binds, Ident, IntoRhsExpr
};

#[derive(Debug, Default)]
pub struct Row {
    values: IndexMap<Ident, Expr>,
    binds: Binds,
}

pub trait IntoRow {
    fn into_row(self) -> Row;
}

impl IntoRow for Row {
    fn into_row(self) -> Row {
        self
    }
}

impl<F> IntoRow for F
where
    F: FnOnce(&mut Row),
{
    fn into_row(self) -> Row {
        let mut row = Row::new();
        self(&mut row);
        row
    }
}

impl IntoRow for Vec<Row> {
    fn into_row(self) -> Row {
        let mut row = Row::new();
        for r in self {
            row.append(r);
        }
        row
    }
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: IndexMap::new(),
            binds: Binds::None,
        }
    }

    pub fn field<K, V>(&mut self, columns: K, value: V) -> &mut Self
    where
        K: IntoIdent,
        V: IntoRhsExpr,
    {
        let col_ident = columns.into_ident();
        let mut expr = value.into_rhs_expr();
        self.binds.append(expr.take_bindings());
        self.values.insert(col_ident, expr);
        self
    }

    pub(crate) fn append(&mut self, other: Self) {
        self.binds.append(other.binds);
        self.values.extend(other.values);
    }

    pub(crate) fn format_idents<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        for (i, (ident, _)) in self.values.iter().enumerate() {
            if i > 0 {
                context.writer.write_str(", ")?;
            }
            ident.format_writer(context)?;
        }
        Ok(())
    }

    pub(crate) fn format_values<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        for (i, (_, expr)) in self.values.iter().enumerate() {
            if i > 0 {
                context.writer.write_str(", ")?;
            }
            expr.format_writer(context)?;
        }
        Ok(())
    }

    pub fn build(&mut self) -> Self {
        // take mem
        Row {
            values: std::mem::take(&mut self.values),
            binds: self.binds.take(),
        }
    }
}

impl TakeBindings for Row {
    fn take_bindings(&mut self) -> Binds {
        self.binds.take()
    }
}

impl FormatWriter for Row {
    fn format_writer<W: std::fmt::Write>(&self, context: &mut crate::writer::FormatContext<'_, W>) -> std::fmt::Result {
        // format for all dialect insert values (<rows>)
        for (i, (_, expr)) in self.values.iter().enumerate() {
            if i > 0 {
                context.writer.write_str(", ")?;
            }
            expr.format_writer(context)?;
        }
        Ok(())
    }
}
