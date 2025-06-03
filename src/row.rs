use std::collections::HashMap;

use crate::{
    expr::{Expr, TakeBindings}, ident::IntoIdent, Binds, Ident, IntoRhsExpr
};

#[derive(Debug, Default)]
pub struct Row {
    values: HashMap<Ident, Expr>,
    binds: Binds,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
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
}

impl TakeBindings for Row {
    fn take_bindings(&mut self) -> Binds {
        self.binds.take()
    }
}
