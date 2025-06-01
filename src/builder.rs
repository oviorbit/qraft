use qraft_derive::{condition_variant, or_variant, variant};

use crate::{
    IntoInList, JoinClause, JoinType, Joins,
    bind::{Binds, IntoBinds},
    col::{
        IntoProjections, IntoProjectionsWithSub, IntoTable, ProjectionSchema, Projections,
        TableSchema,
    },
    dialect::HasDialect,
    expr::{
        Expr, IntoLhsExpr, IntoOperator, IntoRhsExpr, TakeBindings,
        between::BetweenOperator,
        binary::Operator,
        cond::{Conditions, Conjunction},
        exists::ExistsOperator,
        r#in::InOperator,
        order::{Order, Ordering},
        unary::UnaryOperator,
    },
    ident::{IntoIdent, TableRef},
    raw::IntoRaw,
    writer::{FormatContext, FormatWriter},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum QueryKind {
    #[default]
    Select,
    Where,
    Having,
    Join,
}

impl TakeBindings for Builder {
    fn take_bindings(&mut self) -> Binds {
        self.binds.take_bindings()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Builder {
    query: String,
    ty: QueryKind,
    distinct: bool,
    maybe_table: Option<TableRef>,
    projections: Projections,
    binds: Binds,
    maybe_where: Option<Conditions>,
    maybe_having: Option<Conditions>,
    maybe_limit: Option<usize>,
    maybe_offset: Option<usize>,
    maybe_order: Option<Order>,
    maybe_joins: Option<Joins>,
    maybe_group_by: Option<Projections>,
}

impl Builder {
    pub fn table_schema<T: TableSchema>() -> Self {
        Self::table(T::table())
    }

    pub fn table<T: IntoTable>(table: T) -> Self {
        Self {
            query: String::new(),
            distinct: false,
            maybe_table: Some(table.into_table()),
            projections: Projections::None,
            binds: Binds::None,
            ty: QueryKind::Select,
            maybe_where: None,
            maybe_limit: None,
            maybe_offset: None,
            maybe_order: None,
            maybe_having: None,
            maybe_joins: None,
            maybe_group_by: None,
        }
    }

    pub fn from<T: IntoTable>(&mut self, table: T) -> &mut Self {
        self.maybe_table = Some(table.into_table());
        self
    }

    pub fn from_sub<F, I>(&mut self, table: F, alias: I) -> &mut Self
    where
        F: FnOnce(&mut Self),
        I: IntoIdent,
    {
        let mut inner = Self::default();
        table(&mut inner);
        self.binds.append(inner.take_bindings());
        self.maybe_table = Some(TableRef::AliasedSub(Box::new(inner), alias.into_ident()));
        self
    }

    // conditionnals

    pub fn when<F>(&mut self, condition: bool, builder: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        if condition {
            builder(self);
        }
        self
    }

    pub fn when_some<T, F>(&mut self, maybe_value: Option<T>, builder: F) -> &mut Self
    where
        F: FnOnce(&mut Self, T),
    {
        if let Some(value) = maybe_value {
            builder(self, value);
        }
        self
    }

    // joins stuff
    pub fn join<T, C, O, CC>(
        &mut self,
        table: T,
        column: C,
        operator: O,
        other_column: CC,
    ) -> &mut Self
    where
        T: IntoTable,
        C: IntoTable,
        O: IntoOperator,
        CC: IntoTable,
    {
        self.join_clause(table, |join| {
            join.on(column, operator, other_column);
        });
        self
    }

    pub fn left_join<T, C, O, CC>(
        &mut self,
        table: T,
        column: C,
        operator: O,
        other_column: CC,
    ) -> &mut Self
    where
        T: IntoTable,
        C: IntoTable,
        O: IntoOperator,
        CC: IntoTable,
    {
        self.left_join_clause(table, |join| {
            join.on(column, operator, other_column);
        });
        self
    }

    pub fn right_join<T, C, O, CC>(
        &mut self,
        table: T,
        column: C,
        operator: O,
        other_column: CC,
    ) -> &mut Self
    where
        T: IntoTable,
        C: IntoTable,
        O: IntoOperator,
        CC: IntoTable,
    {
        self.left_join_clause(table, |join| {
            join.on(column, operator, other_column);
        });
        self
    }

    pub fn cross_join<T: IntoTable>(&mut self, table: T) -> &mut Self {
        let mut join = JoinClause::new(JoinType::Cross, table.into_table());
        self.binds.append(join.take_bindings());
        let target = self.maybe_joins.get_or_insert_default();
        target.push(join);
        self
    }

    pub fn join_sub<F, A, J>(&mut self, sub: F, alias: A, clause: J) -> &mut Self
    where
        F: FnOnce(&mut Builder),
        A: IntoIdent,
        J: FnOnce(&mut JoinClause),
    {
        let mut inner = Self::default();
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        let table_ref = TableRef::AliasedSub(Box::new(inner), alias.into_ident());
        self.join_clause(table_ref, clause);
        self
    }

    pub fn left_join_sub<F, A, J>(&mut self, sub: F, alias: A, clause: J) -> &mut Self
    where
        F: FnOnce(&mut Builder),
        A: IntoIdent,
        J: FnOnce(&mut JoinClause),
    {
        let mut inner = Self::default();
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        let table_ref = TableRef::AliasedSub(Box::new(inner), alias.into_ident());
        self.left_join_clause(table_ref, clause);
        self
    }

    pub fn right_join_sub<F, A, J>(&mut self, sub: F, alias: A, clause: J) -> &mut Self
    where
        F: FnOnce(&mut Builder),
        A: IntoIdent,
        J: FnOnce(&mut JoinClause),
    {
        let mut inner = Self::default();
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        let table_ref = TableRef::AliasedSub(Box::new(inner), alias.into_ident());
        self.right_join_clause(table_ref, clause);
        self
    }

    pub fn join_clause<T, F>(&mut self, table: T, sub: F) -> &mut Self
    where
        T: IntoTable,
        F: FnOnce(&mut JoinClause),
    {
        let mut join = JoinClause::new(JoinType::Inner, table.into_table());
        sub(&mut join);
        self.binds.append(join.take_bindings());
        let target = self.maybe_joins.get_or_insert_default();
        target.push(join);
        self
    }

    pub fn left_join_clause<T, F>(&mut self, table: T, sub: F) -> &mut Self
    where
        T: IntoTable,
        F: FnOnce(&mut JoinClause),
    {
        let mut join = JoinClause::new(JoinType::Left, table.into_table());
        sub(&mut join);
        self.binds.append(join.take_bindings());
        let target = self.maybe_joins.get_or_insert_default();
        target.push(join);
        self
    }

    pub fn right_join_clause<T, F>(&mut self, table: T, sub: F) -> &mut Self
    where
        T: IntoTable,
        F: FnOnce(&mut JoinClause),
    {
        let mut join = JoinClause::new(JoinType::Right, table.into_table());
        sub(&mut join);
        self.binds.append(join.take_bindings());
        let target = self.maybe_joins.get_or_insert_default();
        target.push(join);
        self
    }

    // group by stuff

    pub fn group_by<T: IntoProjections>(&mut self, projections: T) -> &mut Self {
        let proj = projections.into_projections();
        self.maybe_group_by = Some(proj);
        self
    }

    pub fn reset_group_by(&mut self) -> &mut Self {
        self.maybe_group_by = None;
        self
    }

    // where stuff

    fn reset_where(&mut self) -> &mut Self {
        self.maybe_where = None;
        self.binds = Binds::None;
        self
    }

    #[or_variant]
    pub fn where_clause<C, O, V>(&mut self, column: C, operator: O, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        let conditions = self.maybe_where.get_or_insert_default();
        let mut lhs = column.into_lhs_expr();
        let mut rhs = value.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        conditions.push_binary(Conjunction::And, lhs, rhs, operator.into_operator());
        self
    }

    #[condition_variant]
    fn where_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = Self {
            ty: QueryKind::Where,
            ..Default::default()
        };
        sub(&mut inner);

        let binds = inner.take_bindings();
        if let Some(conds) = inner.maybe_where {
            self.binds.append(binds);
            let target = self.maybe_where.get_or_insert_default();
            target.push_group(Conjunction::And, conds);
        }

        self
    }

    #[condition_variant(not)]
    fn where_not_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = Self {
            ty: QueryKind::Where,
            ..Default::default()
        };
        sub(&mut inner);

        let binds = inner.take_bindings();
        if let Some(conds) = inner.maybe_where {
            self.binds.append(binds);
            let target = self.maybe_where.get_or_insert_default();
            target.push_group(Conjunction::AndNot, conds);
        }

        self
    }

    #[condition_variant]
    pub fn where_raw<R, B>(&mut self, raw: R, binds: B) -> &mut Self
    where
        R: IntoRaw,
        B: IntoBinds,
    {
        let raw = raw.into_raw();
        let binds = binds.into_binds();
        let target = self.maybe_where.get_or_insert_default();
        self.binds.append(binds);
        target.push_raw(Conjunction::And, raw);
        self
    }

    #[variant(Operator, Eq, eq, not_eq, like, not_like, ilike, not_ilike)]
    fn where_binary<C, V>(&mut self, column: C, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        V: IntoRhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        let mut rhs = value.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        let target = self.maybe_where.get_or_insert_default();
        target.push_binary(Conjunction::And, lhs, rhs, Operator::Eq);
        self
    }

    #[condition_variant]
    pub fn where_column<C, O, CC>(&mut self, column: C, operator: O, other_column: CC) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        CC: IntoLhsExpr,
    {
        let mut lhs = column.into_lhs_expr();
        let mut rhs = other_column.into_lhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        let target = self.maybe_where.get_or_insert_default();
        target.push_binary(Conjunction::And, lhs, rhs, operator.into_operator());
        self
    }

    #[variant(UnaryOperator, Null, null, not_null, true, false)]
    fn unary_expr<C: IntoLhsExpr>(&mut self, column: C) -> &mut Self {
        let mut column = column.into_lhs_expr();
        self.binds.append(column.take_bindings());
        let target = self.maybe_where.get_or_insert_default();
        target.push_unary(Conjunction::And, column, UnaryOperator::Null);
        self
    }

    #[variant(BetweenOperator, Between, between, not_between)]
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
        let target = self.maybe_where.get_or_insert_default();
        target.push_between(Conjunction::And, lhs, low, high, BetweenOperator::Between);
        self
    }

    #[variant(BetweenOperator, Between, between_columns Between, not_between_columns NotBetween)]
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
        let target = self.maybe_where.get_or_insert_default();
        target.push_between(Conjunction::And, lhs, low, high, BetweenOperator::Between);
        self
    }

    #[variant(ExistsOperator, Exists, exists, not_exists)]
    fn exists_expr<Q>(&mut self, sub: Q) -> &mut Self
    where
        Q: FnOnce(&mut Self),
    {
        let mut inner = Self::default();
        sub(&mut inner);
        self.binds.append(inner.take_bindings());
        let target = self.maybe_where.get_or_insert_default();
        target.push_exists(Conjunction::And, inner, ExistsOperator::Exists);
        self
    }

    #[variant(InOperator, In, in, not_in)]
    fn in_expr<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
    where
        L: IntoLhsExpr,
        R: IntoInList,
    {
        let mut lhs = lhs.into_lhs_expr();
        let mut rhs = rhs.into_in_list();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        let target = self.maybe_where.get_or_insert_default();
        target.push_in(Conjunction::And, lhs, rhs, InOperator::In);
        self
    }

    #[condition_variant]
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

    #[condition_variant]
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

    #[condition_variant(not)]
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

    // havings here

    fn reset_having(&mut self) -> &mut Self {
        // not public for now, needs impl Take binds on all conditions
        self.maybe_having = None;
        self.binds = Binds::None;
        self
    }

    #[or_variant]
    pub fn having<C, O, V>(&mut self, column: C, operator: O, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        let conditions = self.maybe_having.get_or_insert_default();
        let mut lhs = column.into_lhs_expr();
        let mut rhs = value.into_rhs_expr();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        conditions.push_binary(Conjunction::And, lhs, rhs, operator.into_operator());
        self
    }

    #[condition_variant(none)]
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
                let conditions = builder.maybe_where.get_or_insert_default();
                let mut lhs = proj.into_lhs_expr();
                let mut rhs = value.clone();
                builder.binds.append(lhs.take_bindings());
                builder.binds.append(rhs.take_bindings());
                conditions.push_binary(conj, lhs, rhs, operator);
            }
        };
        match group_conj {
            Conjunction::And => self.where_group(closure),
            Conjunction::Or => self.or_where_group(closure),
            Conjunction::AndNot => self.where_not_group(closure),
            Conjunction::OrNot => self.or_where_not_group(closure),
        }
    }

    // add order by stuff

    pub fn order_by_asc<I: IntoLhsExpr>(&mut self, column: I) -> &mut Self {
        self.order_by_expr(column.into_lhs_expr(), Ordering::Asc)
    }

    pub fn order_by_desc<I: IntoLhsExpr>(&mut self, column: I) -> &mut Self {
        self.order_by_expr(column.into_lhs_expr(), Ordering::Desc)
    }

    pub fn latest<I: IntoLhsExpr>(&mut self, column: I) -> &mut Self {
        self.order_by_desc(column)
    }

    pub fn oldest<I: IntoLhsExpr>(&mut self, column: I) -> &mut Self {
        self.order_by_asc(column)
    }

    pub fn reset_order(&mut self) -> &mut Self {
        self.maybe_order = None;
        self
    }

    pub fn order_by_raw<R, B>(&mut self, raw: R, binds: B) -> &mut Self
    where
        R: IntoRaw,
        B: IntoBinds,
    {
        let binds = binds.into_binds();
        self.binds.append(binds);
        let o = self.maybe_order.get_or_insert_default();
        let raw = raw.into_raw();
        o.push_raw(raw);
        self
    }

    pub fn order_by_random(&mut self) -> &mut Self {
        let o = self.maybe_order.get_or_insert_default();
        o.push_random();
        self
    }

    #[inline]
    pub(crate) fn order_by_expr(&mut self, ident: Expr, order: Ordering) -> &mut Self {
        let o = self.maybe_order.get_or_insert_default();
        o.push_expr(ident, order);
        self
    }

    // select stuff

    pub fn select_raw<T, B>(&mut self, value: T, binds: B) -> &mut Self
    where
        T: IntoRaw,
        B: IntoBinds,
    {
        let raw = value.into_raw();
        self.projections = Projections::One(TableRef::Raw(raw));
        self.binds.append(binds.into_binds());
        self
    }

    pub fn select_schema<T: ProjectionSchema>(&mut self) -> &mut Self {
        self.projections = T::projections();
        self
    }

    pub fn select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoProjectionsWithSub,
    {
        self.projections = cols.into_projections_with_sub();
        self
    }

    pub fn add_select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoProjectionsWithSub,
    {
        let other = cols.into_projections_with_sub();
        self.projections.append(other);
        self
    }

    fn reset_select(&mut self) -> &mut Self {
        // could be made public but needs to impl take bindings on all conds
        self.projections.reset();
        self
    }

    pub fn distinct(&mut self) -> &mut Self {
        self.distinct = true;
        self
    }

    pub fn reset_distinct(&mut self) -> &mut Self {
        self.distinct = false;
        self
    }

    // pagination stuff

    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.maybe_limit = Some(limit);
        self
    }

    pub fn reset_limit(&mut self) -> &mut Self {
        self.maybe_limit = None;
        self
    }

    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.maybe_offset = Some(offset);
        self
    }

    pub fn reset_offset(&mut self) -> &mut Self {
        self.maybe_offset = None;
        self
    }

    // building the builder

    pub fn to_sql<Database: HasDialect>(&mut self) -> &str {
        let size_hint = 64;
        let mut str = String::with_capacity(size_hint);
        let mut context = FormatContext::new(&mut str, Database::DIALECT);
        self.format_writer(&mut context)
            .expect("should not fail on a string writer");
        self.query = str;
        self.query.as_str()
    }
}

impl FormatWriter for Builder {
    fn format_writer<W: std::fmt::Write>(
        &self,
        context: &mut crate::writer::FormatContext<'_, W>,
    ) -> std::fmt::Result {
        if self.distinct {
            context.writer.write_str("select distinct ")?;
        } else {
            context.writer.write_str("select ")?;
        }
        self.projections.format_writer(context)?;
        if let Some(ref table) = self.maybe_table {
            context.writer.write_str(" from ")?;
            table.format_writer(context)?;
        }

        // joins here
        if let Some(ref joins) = self.maybe_joins {
            context.writer.write_char(' ')?;
            for (index, join) in joins.iter().enumerate() {
                if index > 0 {
                    context.writer.write_char(' ')?;
                }
                join.format_writer(context)?;
            }
        }

        if let Some(ref w) = self.maybe_where {
            // if we are not in a where group
            if !w.is_empty() {
                if matches!(self.ty, QueryKind::Select) {
                    context.writer.write_str(" where ")?;
                }
                w.format_writer(context)?;
            }
        }

        // group by
        if let Some(ref group_by) = self.maybe_group_by {
            if matches!(self.ty, QueryKind::Select) {
                context.writer.write_str(" group by ")?;
            }
            group_by.format_writer(context)?;
        }

        if let Some(ref h) = self.maybe_having {
            // if we are not in a having group
            if !h.is_empty() {
                if matches!(self.ty, QueryKind::Select) {
                    context.writer.write_str(" having ")?;
                }
                h.format_writer(context)?;
            }
        }

        if let Some(ref order) = self.maybe_order {
            if !order.is_empty() {
                context.writer.write_str(" order by ")?;
                order.format_writer(context)?;
            }
        }

        if let Some(limit) = self.maybe_limit {
            write!(context.writer, " limit {}", limit)?;
        }

        if let Some(offset) = self.maybe_offset {
            write!(context.writer, " offset {}", offset)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bind::{self, Bind},
        col::ProjectionSchema,
        column_static,
        dialect::Postgres,
        raw, sub,
    };

    use super::*;

    #[test]
    fn test_basic_select() {
        let mut builder = Builder::table("users");
        builder.select("i\"d");
        assert_eq!(
            "select \"i\"\"d\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
        builder.select("username");
        assert_eq!(
            "select \"username\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
        builder.add_select("id");
        assert_eq!(
            "select \"username\", \"id\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
        builder.reset_select();
        assert_eq!("select * from \"users\"", builder.to_sql::<Postgres>());
    }

    #[derive(Debug, Clone)]
    struct User {
        id: i64,
        admin: bool,
    }

    // generated ?
    impl TableSchema for User {
        fn table() -> TableRef {
            TableRef::ident_static("users")
        }
    }

    // generated ?
    impl ProjectionSchema for User {
        fn projections() -> Projections {
            [column_static("id"), column_static("admin")].into_projections()
        }
    }

    #[test]
    fn test_select_into_ident() {
        let mut builder = Builder::table_schema::<User>();
        builder.select_schema::<User>();
        assert_eq!(
            "select \"id\", \"admin\" from \"users\"",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_select_raw() {
        let mut builder = Builder::table("users");
        builder.select_raw("id, count(*)", Binds::None);
        assert_eq!(
            "select id, count(*) from \"users\"",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_select_raw_bound() {
        let mut builder = Builder::table("users");
        builder.select_raw("price + ? as fee", [5]);
        assert_eq!(
            "select price + $1 as fee from \"users\"",
            builder.to_sql::<Postgres>()
        );
        assert_eq!(1, builder.binds.len());
        let value = match builder.binds {
            bind::Array::None => panic!("should have one value"),
            bind::Array::One(value) => value,
            bind::Array::Many(_) => panic!("wrong size"),
        };
        assert!(matches!(value, Bind::I32(5)));
    }

    #[test]
    fn test_expr_where() {
        let mut builder = Builder::table("users");
        builder.where_eq(
            "id",
            sub(|builder| {
                builder.select("id").from("roles");
            }),
        );
        assert_eq!(
            "select * from \"users\" where \"id\" = (select \"id\" from \"roles\")",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_expr_and_conds() {
        let mut builder = Builder::table("users");
        builder.where_like("username", 3);
        builder.where_eq("id", 1);
        builder.or_where_eq("name", 3);
        builder.where_eq(
            "foo",
            sub(|builder| {
                builder.select("id").from("bar");
            }),
        );
        assert_eq!(
            "select * from \"users\" where \"username\"::text like $1 and \"id\" = $2 or \"name\" = $3 and \"foo\" = (select \"id\" from \"bar\")",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_expr_value_column() {
        let mut builder = Builder::table("users");
        builder.where_like(
            sub(|builder| {
                builder.select("foo").from("bar");
            }),
            3,
        );
        assert_eq!(
            "select * from \"users\" where (select \"foo\" from \"bar\")::text like $1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_expr_like() {
        let mut builder = Builder::table("users");
        builder.where_like("username", 3);
        assert_eq!(
            "select * from \"users\" where \"username\"::text like $1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_where_group_expr() {
        let mut builder = Builder::table("users");
        builder.where_group(|builder| {
            builder
                .where_eq("foo", 3)
                .or_where_like("foo", column_static("bar"));
        });
        assert_eq!(
            "select * from \"users\" where (\"foo\" = $1 or \"foo\"::text like \"bar\")",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_in_expr() {
        let mut builder = Builder::table("users");
        builder.where_in("id", 3);
        assert_eq!(
            "select * from \"users\" where \"id\" in ($1)",
            builder.to_sql::<Postgres>()
        );
        builder.reset_where();
        builder.where_in("id", [1, 2, 3]);
        assert_eq!(
            "select * from \"users\" where \"id\" in ($1, $2, $3)",
            builder.to_sql::<Postgres>()
        );
        builder.reset_where();
        builder.where_in("id", [1, 2, 3]);
        assert_eq!(
            "select * from \"users\" where \"id\" in ($1, $2, $3)",
            builder.to_sql::<Postgres>()
        );
        builder.reset_where();
        builder.where_in("id", [1, 2, 3]);
        builder.where_eq("foo", 1);
        assert_eq!(
            "select * from \"users\" where \"id\" in ($1, $2, $3) and \"foo\" = $4",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_exists_expr() {
        let mut builder = Builder::table("users");
        builder.where_exists(|builder| {
            builder.select(raw("1")).from("users").where_eq("id", 1);
        });
        builder.where_eq("foo", "bar");
        assert_eq!(
            "select * from \"users\" where exists (select 1 from \"users\" where \"id\" = $1) and \"foo\" = $2",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_where_not_group() {
        let mut builder = Builder::table("users");
        builder.where_eq("value", "bar");
        builder.where_not_group(|builder| {
            builder
                .where_eq("foo", "bar")
                .select("id")
                .where_like("bar", "foo");
        });
        builder.where_eq("baz", "bar");
        assert_eq!(
            "select * from \"users\" where \"value\" = $1 and not (\"foo\" = $2 and \"bar\"::text like $3) and \"baz\" = $4",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_from_sub() {
        let mut builder = Builder::table("users");
        builder.from_sub(
            |builder| {
                builder.where_eq("username", "foo").from("bar");
            },
            "foo",
        );
        assert_eq!(
            "select * from (select * from \"bar\" where \"username\" = $1) as \"foo\"",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_limit_clause() {
        let mut builder = Builder::table("users");
        builder.limit(42);
        builder.limit(1);
        assert!(builder.binds.is_empty());
        assert_eq!(
            "select * from \"users\" limit 1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_offset_clause() {
        let mut builder = Builder::table("users");
        builder.offset(42);
        assert!(builder.binds.is_empty());
        assert_eq!(
            "select * from \"users\" offset 42",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_order_clause() {
        let mut builder = Builder::table("users");
        builder.order_by_asc("id");

        assert_eq!(
            "select * from \"users\" order by \"id\" asc",
            builder.to_sql::<Postgres>()
        );

        builder.order_by_desc("username");

        assert!(builder.binds.is_empty());
        assert_eq!(
            "select * from \"users\" order by \"id\" asc, \"username\" desc",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_where_any() {
        let mut builder = Builder::table("users");
        builder
            .where_eq("id", 1)
            .where_any(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(builder.binds.len(), 4);
        assert_eq!(
            "select * from \"users\" where \"id\" = $1 and (\"id\" = $2 or \"foo\" = $3 or \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );

        assert_eq!(builder.binds.len(), 4);

        builder.reset_where();

        builder
            .where_eq("id", 1)
            .or_where_any(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(builder.binds.len(), 4);
        assert_eq!(
            "select * from \"users\" where \"id\" = $1 or (\"id\" = $2 or \"foo\" = $3 or \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_where_all() {
        let mut builder = Builder::table("users");
        builder
            .where_eq("id", 1)
            .where_all(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(
            "select * from \"users\" where \"id\" = $1 and (\"id\" = $2 and \"foo\" = $3 and \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );
        assert_eq!(builder.binds.len(), 4);
        builder.reset_where();

        builder
            .where_eq("id", 1)
            .or_where_all(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(
            "select * from \"users\" where \"id\" = $1 or (\"id\" = $2 and \"foo\" = $3 and \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );
        assert_eq!(builder.binds.len(), 4);
    }

    #[test]
    fn test_where_none() {
        let mut builder = Builder::table("users");
        builder
            .where_eq("id", 1)
            .where_none(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(
            "select * from \"users\" where \"id\" = $1 and not (\"id\" = $2 or \"foo\" = $3 or \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );
        assert_eq!(builder.binds.len(), 4);
        builder.reset_where();

        builder
            .where_eq("id", 1)
            .or_where_none(["id", "foo", "bar"], Operator::Eq, "baz");

        assert_eq!(
            "select * from \"users\" where \"id\" = $1 or not (\"id\" = $2 or \"foo\" = $3 or \"bar\" = $4)",
            builder.to_sql::<Postgres>()
        );
        assert_eq!(builder.binds.len(), 4);
    }

    #[test]
    fn test_join() {
        let mut builder = Builder::table("users");
        builder
            .join("contacts", "users.id", "=", "contacts.user_id")
            .join("orders", "users.id", '=', "orders.user_id")
            .select(["users.*", "contacts.phone", "orders.price"]);
        let result = r#"select "users".*, "contacts"."phone", "orders"."price" from "users" inner join "contacts" on "users"."id" = "contacts"."user_id" inner join "orders" on "users"."id" = "orders"."user_id""#;
        assert_eq!(result, builder.to_sql::<Postgres>());
    }

    #[test]
    fn test_join_clause() {
        let mut builder = Builder::table("users");
        builder.join_clause("orders", |clause| {
            clause.where_eq("foo", "bar");
        });
        let result = r#"select * from "users" inner join "orders" on "foo" = $1"#;
        assert_eq!(result, builder.to_sql::<Postgres>());
        assert_eq!(builder.binds.len(), 1);
    }

    #[test]
    fn test_group_by() {
        let mut builder = Builder::table("users");
        let result = r#"select * from "users" inner join "orders" on "foo" = $1"#;
        assert_eq!(result, builder.to_sql::<Postgres>());
    }
}
