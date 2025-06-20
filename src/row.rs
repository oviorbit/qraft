use crate::{
    expr::{Expr, TakeBindings}, ident::IntoIdent, writer::FormatWriter, Binds, Ident, IntoRhsExpr
};

#[derive(Debug, Default)]
pub struct Row {
    values: Vec<(Ident, Expr)>,
    binds: Binds,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
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
        self.values.push((col_ident, expr));
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
