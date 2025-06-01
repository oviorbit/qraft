use std::fmt;

use crate::{
    Binds, Raw, TableRef,
    bind::Array,
    dialect::Dialect,
    writer::{self, FormatWriter},
};

use super::{Expr, TakeBindings};

#[derive(Debug, Clone)]
pub enum OrderExpr {
    Column(Expr, Ordering),
    Raw(Raw),
    Random,
}

impl TakeBindings for OrderExpr {
    fn take_bindings(&mut self) -> Binds {
        match self {
            OrderExpr::Column(table_ref, _) => table_ref.take_bindings(),
            OrderExpr::Raw(_) => Binds::None,
            OrderExpr::Random => Binds::None,
        }
    }
}

pub type OrderProjections = Array<OrderExpr>;

impl TakeBindings for OrderProjections {
    fn take_bindings(&mut self) -> Binds {
        self.iter_mut()
            .map(|v| v.take_bindings())
            .fold(Binds::None, |mut acc, next| {
                acc.append(next);
                acc
            })
    }
}

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
            OrderExpr::Random => match context.dialect {
                Dialect::Postgres | Dialect::Sqlite => context.writer.write_str("random()"),
                Dialect::MySql => context.writer.write_str("rand()"),
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Order {
    projections: OrderProjections,
}

impl TakeBindings for Order {
    fn take_bindings(&mut self) -> Binds {
        self.projections.take_bindings()
    }
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
    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }

    pub fn push_expr(&mut self, ident: Expr, ordering: Ordering) {
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
