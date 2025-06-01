use qraft_derive::{variant, or_variant};

use crate::{builder::QueryKind, expr::{between::BetweenOperator, binary::Operator, cond::Conditions, exists::ExistsOperator, r#in::InOperator, unary::UnaryOperator, Conjunction, Expr, TakeBindings}, Binds, Builder, IntoBinds, IntoInList, IntoLhsExpr, IntoOperator, IntoProjections, IntoRaw, IntoRhsExpr, Projections, TableRef};

pub enum JoinType {
    Inner,
    Left,
    Right,
}

pub struct JoinClause {
    kind: QueryKind,
    ty: JoinType,
    table: Option<TableRef>,
    conditions: Conditions,
    binds: Binds,
}

impl Default for JoinClause {
    fn default() -> Self {
        Self {
            kind: QueryKind::Join,
            ty: JoinType::Inner,
            table: None,
            conditions: Conditions::default(),
            binds: Binds::None,
        }
    }
}

impl TakeBindings for JoinClause {
    fn take_bindings(&mut self) -> crate::Binds {
        self.binds.take_bindings()
    }
}

impl JoinClause {
    pub fn new(ty: JoinType, table: TableRef) -> Self {
        Self {
            ty,
            table: Some(table),
            conditions: Conditions::default(),
            kind: QueryKind::Join,
            binds: Binds::None,
        }
    }

    // do stuff here ?

    #[or_variant]
    pub fn on<C, O, CC>(&mut self, column: C, operator: O, other_column: CC) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        CC: IntoLhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        let mut rhs = other_column.into_lhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        self.conditions.push_binary(Conjunction::And, lhs, rhs, operator.into_operator());
        self
    }

    #[or_variant]
    pub fn where_clause<C, O, V>(&mut self, column: C, operator: O, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        let mut rhs = value.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        self.conditions.push_binary(Conjunction::And, lhs, rhs, operator.into_operator());
        self
    }

    #[or_variant]
    fn where_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = Self {
            kind: QueryKind::Where,
            ..Default::default()
        };
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        self.conditions.push_group(Conjunction::And, inner.conditions);
        self
    }

    #[or_variant(not)]
    fn where_not_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = Self {
            kind: QueryKind::Where,
            ..Default::default()
        };
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        self.conditions.push_group(Conjunction::And, inner.conditions);
        self
    }

    #[or_variant]
    pub fn where_raw<R, B>(&mut self, raw: R, binds: B) -> &mut Self
    where
        R: IntoRaw,
        B: IntoBinds,
    {
        let raw = raw.into_raw();
        let binds = binds.into_binds();
        self.binds.append(binds);
        self.conditions.push_raw(Conjunction::And, raw);
        self
    }

    #[variant(join, Operator, Eq, eq, not_eq, like, not_like, ilike, not_ilike)]
    fn where_binary<C, V>(&mut self, column: C, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        V: IntoRhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        let mut rhs = value.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        self.conditions.push_binary(Conjunction::And, lhs, rhs, Operator::Eq);
        self
    }

    #[variant(join, UnaryOperator, Null, null, not_null, true, false)]
    fn unary_expr<C>(&mut self, column: C) -> &mut Self
    where
        C: IntoLhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        self.binds.append(lhs.take_bindings());
        self.conditions.push_unary(Conjunction::And, lhs, UnaryOperator::Null);
        self
    }

    #[variant(join, BetweenOperator, Between, between, not_between)]
    fn between_expr<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
    where
        C: IntoLhsExpr,
        L: IntoRhsExpr,
        H: IntoRhsExpr,
    {
        let mut lhs = lhs.into_lhs_expr();
        let mut low = low.into_rhs_expr();
        let mut high = high.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(low.take_bindings());
        self.binds.append(high.take_bindings());
        self.conditions.push_between(Conjunction::And, lhs, low, high, BetweenOperator::Between);
        self
    }

    #[variant(join, BetweenOperator, Between, between_columns Between, not_between_columns NotBetween)]
    fn between_columns_expr<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
    where
        C: IntoLhsExpr,
        L: IntoLhsExpr,
        H: IntoLhsExpr,
    {
        let mut lhs = lhs.into_lhs_expr();
        let mut low = low.into_lhs_expr();
        let mut high = high.into_lhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(low.take_bindings());
        self.binds.append(high.take_bindings());
        self.conditions.push_between(Conjunction::And, lhs, low, high, BetweenOperator::Between);
        self
    }

    #[variant(join, ExistsOperator, Exists, exists, not_exists)]
    fn exists_expr<Q>(&mut self, sub: Q) -> &mut Self
    where
        Q: FnOnce(&mut Builder),
    {
        let mut inner = Builder::default();
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        self.conditions.push_exists(Conjunction::And, inner, ExistsOperator::Exists);
        self
    }

    #[variant(join, InOperator, In, in, not_in)]
    fn in_expr<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
    where
        L: IntoLhsExpr,
        R: IntoInList,
    {
        let mut lhs = lhs.into_lhs_expr();
        let mut rhs = rhs.into_in_list();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        self.conditions.push_in(Conjunction::And, lhs, rhs, InOperator::In);
        self
    }

    #[or_variant]
    pub fn where_all<C, O, V>(&mut self, columns: C, operator: O, rhs: V) -> &mut Self
    where
        C: IntoProjections,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        self.where_grouped_expr(
            Conjunction::And,
            Conjunction::And,
            columns.into_projections(),
            rhs.into_rhs_expr(),
            operator.into_operator(),
        )
    }

    #[or_variant]
    pub fn where_any<C, O, V>(&mut self, columns: C, operator: O, rhs: V) -> &mut Self
    where
        C: IntoProjections,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        self.where_grouped_expr(
            Conjunction::And,
            Conjunction::Or,
            columns.into_projections(),
            rhs.into_rhs_expr(),
            operator.into_operator(),
        )
    }

    #[or_variant(not)]
    pub fn where_none<C, O, V>(&mut self, columns: C, operator: O, rhs: V) -> &mut Self
    where
        C: IntoProjections,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        self.where_grouped_expr(
            Conjunction::AndNot,
            Conjunction::Or,
            columns.into_projections(),
            rhs.into_rhs_expr(),
            operator.into_operator(),
        )
    }


    fn where_grouped_expr(
        &mut self,
        group_conj: Conjunction,
        conj: Conjunction,
        projections: Projections,
        value: Expr,
        operator: Operator,
    ) -> &mut Self {
        let closure = |builder: &mut Self| {
            for proj in projections {
                let mut lhs = proj.into_lhs_expr();
                let mut rhs = value.clone();
                builder.binds.append(lhs.take_bindings());
                builder.binds.append(rhs.take_bindings());
                builder.conditions.push_binary(conj, lhs, rhs, operator);
            }
        };
        match group_conj {
            Conjunction::And => self.where_group(closure),
            Conjunction::Or => self.or_where_group(closure),
            Conjunction::AndNot => self.where_not_group(closure),
            Conjunction::OrNot => self.or_where_not_group(closure),
        }
    }
}
