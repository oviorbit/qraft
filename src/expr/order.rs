use std::fmt;

use crate::{
    bind::Array, dialect::Dialect, writer::{self, FormatWriter}, Raw, TableRef
};

#[derive(Debug, Clone)]
pub enum OrderExpr {
    Column(TableRef, Ordering),
    Raw(Raw),
    Random,
}

pub type OrderProjections = Array<OrderExpr>;

impl FormatWriter for OrderExpr {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> fmt::Result {
        match self {
            OrderExpr::Column(ident, ordering) => {
                ident.format_writer(context)?;
                context.writer.write_char(' ')?;
                ordering.format_writer(context)
            }
            OrderExpr::Raw(raw) => raw.format_writer(context),
            OrderExpr::Random => {
                match context.dialect {
                    Dialect::Postgres | Dialect::Sqlite => context.writer.write_str("random()"),
                    Dialect::MySql => context.writer.write_str("rand()"),
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Order {
    projections: OrderProjections,
}

impl FormatWriter for Order {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        for (index, proj) in self.projections.iter().enumerate() {
            if index > 0 {
                context.writer.write_str(", ")?;
            }
            proj.format_writer(context)?;
        }
        Ok(())
    }
}

impl Order {
    pub fn new() -> Self {
        Self {
            projections: OrderProjections::None,
        }
    }

    pub fn push_proj(&mut self, ident: TableRef, ordering: Ordering) {
        let order_expr = OrderExpr::Column(ident, ordering);
        self.projections.push(order_expr);
    }

    pub fn push_raw(&mut self, raw: Raw) {
        self.projections.push(OrderExpr::Raw(raw));
    }

    pub fn push_random(&mut self) {
        self.projections.push(OrderExpr::Random);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Ordering {
    Asc,
    Desc,
}

impl FormatWriter for Ordering {
    fn format_writer<W: fmt::Write>(
        &self,
        context: &mut writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        match self {
            Ordering::Asc => context.writer.write_str("asc"),
            Ordering::Desc => context.writer.write_str("desc"),
        }
    }
}
