use crate::{
    IntoInList, Raw,
    bind::{Binds, IntoBinds},
    col::{IntoProjections, IntoTable, ProjectionSchema, Projections, TableSchema},
    dialect::HasDialect,
    expr::{
        Expr, IntoLhsExpr, IntoOperator, IntoRhsExpr, TakeBindings,
        between::{BetweenCondition, BetweenOperator},
        binary::{BinaryCondition, Operator},
        cond::{Condition, ConditionKind, Conditions, Conjunction},
        exists::{ExistsCondition, ExistsOperator},
        group::GroupCondition,
        r#in::{InCondition, InOperator},
        list::InList,
        order::{Order, Ordering},
        unary::{UnaryCondition, UnaryOperator},
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
}

impl TakeBindings for Builder {
    fn take_bindings(&mut self) -> Binds {
        std::mem::take(&mut self.binds)
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
}

macro_rules! define_binary {
    ($method:ident, $or_method:ident, $field:ident, $operator:expr) => {
        pub fn $method<C, V>(&mut self, column: C, value: V) -> &mut Self
        where
            C: IntoLhsExpr,
            V: IntoRhsExpr,
        {
            Builder::push_binary_expr(
                &mut self.binds,
                self.$field.get_or_insert_default(),
                Conjunction::And,
                column.into_lhs_expr(),
                $operator,
                value.into_rhs_expr(),
            );
            self
        }

        pub fn $or_method<C, V>(&mut self, column: C, value: V) -> &mut Self
        where
            C: IntoLhsExpr,
            V: IntoRhsExpr,
        {
            Builder::push_binary_expr(
                &mut self.binds,
                self.$field.get_or_insert_default(),
                Conjunction::Or,
                column.into_lhs_expr(),
                $operator,
                value.into_rhs_expr(),
            );
            self
        }
    };
}

macro_rules! define_between {
    ($method:ident, $or_method:ident, $operator:expr) => {
        pub fn $method<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
        where
            C: IntoLhsExpr,
            L: IntoRhsExpr,
            H: IntoRhsExpr,
        {
            self.where_between_expr(
                Conjunction::And,
                lhs.into_lhs_expr(),
                low.into_rhs_expr(),
                high.into_rhs_expr(),
                $operator,
            )
        }

        pub fn $or_method<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
        where
            C: IntoLhsExpr,
            L: IntoRhsExpr,
            H: IntoRhsExpr,
        {
            self.where_between_expr(
                Conjunction::Or,
                lhs.into_lhs_expr(),
                low.into_rhs_expr(),
                high.into_rhs_expr(),
                $operator,
            )
        }
    };
}

macro_rules! define_between_columns {
    ($method:ident, $or_method:ident, $operator:expr) => {
        pub fn $method<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
        where
            C: IntoLhsExpr,
            L: IntoLhsExpr,
            H: IntoLhsExpr,
        {
            self.where_between_expr(
                Conjunction::And,
                lhs.into_lhs_expr(),
                low.into_lhs_expr(),
                high.into_lhs_expr(),
                $operator,
            )
        }

        pub fn $or_method<C, L, H>(&mut self, lhs: C, low: L, high: H) -> &mut Self
        where
            C: IntoLhsExpr,
            L: IntoLhsExpr,
            H: IntoLhsExpr,
        {
            self.where_between_expr(
                Conjunction::Or,
                lhs.into_lhs_expr(),
                low.into_lhs_expr(),
                high.into_lhs_expr(),
                $operator,
            )
        }
    };
}

macro_rules! define_in {
    ($method:ident, $or_method:ident, $operator:expr) => {
        pub fn $method<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
        where
            L: IntoLhsExpr,
            R: IntoInList,
        {
            self.where_in_expr(
                Conjunction::And,
                lhs.into_lhs_expr(),
                rhs.into_in_list(),
                $operator,
            )
        }

        pub fn $or_method<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
        where
            L: IntoLhsExpr,
            R: IntoInList,
        {
            self.where_in_expr(
                Conjunction::Or,
                lhs.into_lhs_expr(),
                rhs.into_in_list(),
                $operator,
            )
        }
    };
}

macro_rules! define_exists {
    ($method:ident, $or_method:ident, $operator:expr) => {
        pub fn $method<Q>(&mut self, sub: Q) -> &mut Self
        where
            Q: FnOnce(&mut Self),
        {
            let mut inner = Self::default();
            sub(&mut inner);
            self.where_exists_expr(Conjunction::And, $operator, inner)
        }

        pub fn $or_method<Q>(&mut self, sub: Q) -> &mut Self
        where
            Q: FnOnce(&mut crate::Builder),
        {
            let mut inner = crate::Builder::default();
            sub(&mut inner);
            self.where_exists_expr(Conjunction::Or, $operator, inner)
        }
    };
}

macro_rules! define_unary {
    ($method:ident, $or_method:ident, $operator:expr) => {
        pub fn $method<C>(&mut self, column: C) -> &mut Self
        where
            C: IntoLhsExpr,
        {
            self.where_unary_expr(Conjunction::And, column.into_lhs_expr(), $operator)
        }

        pub fn $or_method<C>(&mut self, column: C) -> &mut Self
        where
            C: IntoLhsExpr,
        {
            self.where_unary_expr(Conjunction::Or, column.into_lhs_expr(), $operator)
        }
    };
}

macro_rules! define_filter {
    ($method:ident, $c1:expr, $c2:expr) => {
        pub fn $method<C, O, V>(&mut self, columns: C, operator: O, rhs: V) -> &mut Self
        where
            C: IntoProjections,
            O: IntoOperator,
            V: IntoRhsExpr,
        {
            self.where_grouped_expr(
                $c1,
                $c2,
                columns.into_projections(),
                rhs.into_rhs_expr(),
                operator.into_operator(),
            )
        }
    };
}

impl Builder {
    pub fn table_as<T: TableSchema>() -> Self {
        Self::table(T::table())
    }

    pub fn table<T>(table: T) -> Self
    where
        T: IntoTable,
    {
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
        }
    }

    pub fn from<T: IntoTable>(&mut self, table: T) -> &mut Self {
        self.maybe_table = Some(table.into_table());
        self
    }

    pub fn from_sub<I, F>(&mut self, alias: I, table: F) -> &mut Self
    where
        I: IntoIdent,
        F: FnOnce(&mut Self),
    {
        let mut inner = Self::default();
        table(&mut inner);
        self.maybe_table = Some(TableRef::AliasedSub(alias.into_ident(), Box::new(inner)));
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

    // where stuff

    pub fn reset_where(&mut self) -> &mut Self {
        self.maybe_where = None;
        self
    }

    pub fn filter<C, O, V>(&mut self, column: C, operator: O, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        Builder::push_binary_expr(
            &mut self.binds,
            self.maybe_where.get_or_insert_default(),
            Conjunction::And,
            column.into_lhs_expr(),
            operator.into_operator(),
            value.into_rhs_expr(),
        );
        self
    }

    pub fn or_where<C, O, V>(&mut self, column: C, operator: O, value: V) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        V: IntoRhsExpr,
    {
        Builder::push_binary_expr(
            &mut self.binds,
            self.maybe_where.get_or_insert_default(),
            Conjunction::Or,
            column.into_lhs_expr(),
            operator.into_operator(),
            value.into_rhs_expr(),
        );
        self
    }

    pub fn where_not_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.where_group_expr(Conjunction::AndNot, sub)
    }

    pub fn or_where_not_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.where_group_expr(Conjunction::OrNot, sub)
    }

    pub fn where_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.where_group_expr(Conjunction::And, sub)
    }

    pub fn or_where_group<F>(&mut self, sub: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        self.where_group_expr(Conjunction::Or, sub)
    }

    pub fn where_raw<R, B>(&mut self, raw: R, binds: B) -> &mut Self
    where
        R: IntoRaw,
        B: IntoBinds,
    {
        self.where_raw_expr(Conjunction::And, raw.into_raw(), binds.into_binds())
    }

    pub fn or_where_raw<R, B>(&mut self, raw: R, binds: B) -> &mut Self
    where
        R: IntoRaw,
        B: IntoBinds,
    {
        self.where_raw_expr(Conjunction::Or, raw.into_raw(), binds.into_binds())
    }

    pub fn where_column<C, O, CC>(&mut self, column: C, operator: O, other_column: CC) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        CC: IntoLhsExpr,
    {
        Builder::push_binary_expr(
            &mut self.binds,
            self.maybe_where.get_or_insert_default(),
            Conjunction::And,
            column.into_lhs_expr(),
            operator.into_operator(),
            other_column.into_lhs_expr(),
        );
        self
    }

    pub fn or_where_column<C, O, CC>(
        &mut self,
        column: C,
        operator: O,
        other_column: CC,
    ) -> &mut Self
    where
        C: IntoLhsExpr,
        O: IntoOperator,
        CC: IntoLhsExpr,
    {
        Builder::push_binary_expr(
            &mut self.binds,
            self.maybe_where.get_or_insert_default(),
            Conjunction::Or,
            column.into_lhs_expr(),
            operator.into_operator(),
            other_column.into_lhs_expr(),
        );
        self
    }

    define_unary!(where_null, or_where_null, UnaryOperator::Null);
    define_unary!(where_false, or_where_false, UnaryOperator::False);
    define_unary!(where_true, or_where_true, UnaryOperator::True);
    define_unary!(where_not_null, or_where_not_null, UnaryOperator::NotNull);

    define_binary!(where_eq, or_where_eq, maybe_where, Operator::Eq);
    define_binary!(where_gt, or_where_gt, maybe_where, Operator::Gt);
    define_binary!(where_gte, or_where_gte, maybe_where, Operator::Gte);
    define_binary!(where_lt, or_where_lt, maybe_where, Operator::Lt);
    define_binary!(where_lte, or_where_lte, maybe_where, Operator::Lte);
    define_binary!(where_like, or_where_like, maybe_where, Operator::Like);
    define_binary!(where_not_eq, or_where_not_eq, maybe_where, Operator::NotEq);
    define_binary!(
        where_not_like,
        or_where_not_like,
        maybe_where,
        Operator::NotLike
    );
    define_binary!(where_ilike, or_where_ilike, maybe_where, Operator::Ilike);
    define_binary!(
        where_not_ilike,
        or_where_not_ilike,
        maybe_where,
        Operator::NotIlike
    );

    define_between!(where_between, or_where_between, BetweenOperator::Between);
    define_between!(
        where_not_between,
        or_where_not_between,
        BetweenOperator::NotBetween
    );

    define_between_columns!(
        where_between_columns,
        or_where_between_columns,
        BetweenOperator::Between
    );
    define_between_columns!(
        where_not_between_columns,
        or_where_not_between_columns,
        BetweenOperator::NotBetween
    );

    define_exists!(where_exists, or_where_exists, ExistsOperator::Exists);
    define_exists!(
        where_not_exists,
        or_where_not_exists,
        ExistsOperator::NotExists
    );

    define_in!(where_in, or_where_in, InOperator::In);
    define_in!(where_not_in, or_where_not_in, InOperator::NotIn);

    define_filter!(where_all, Conjunction::And, Conjunction::And);
    define_filter!(where_any, Conjunction::And, Conjunction::Or);
    define_filter!(where_none, Conjunction::AndNot, Conjunction::And);
    define_filter!(or_where_all, Conjunction::Or, Conjunction::And);
    define_filter!(or_where_any, Conjunction::Or, Conjunction::Or);
    define_filter!(or_where_none, Conjunction::OrNot, Conjunction::And);

    // havings here

    define_binary!(having_eq, or_having_eq, maybe_having, Operator::Eq);
    define_binary!(having_gt, or_having_gt, maybe_having, Operator::Gt);
    define_binary!(having_gte, or_having_gte, maybe_having, Operator::Gte);
    define_binary!(having_lt, or_having_lt, maybe_having, Operator::Lt);
    define_binary!(having_lte, or_having_lte, maybe_having, Operator::Lte);
    define_binary!(having_like, or_having_like, maybe_having, Operator::Like);
    define_binary!(
        having_not_eq,
        or_having_not_eq,
        maybe_having,
        Operator::NotEq
    );
    define_binary!(
        having_not_like,
        or_having_not_like,
        maybe_having,
        Operator::NotLike
    );
    define_binary!(having_ilike, or_having_ilike, maybe_having, Operator::Ilike);
    define_binary!(
        having_not_ilike,
        or_having_not_ilike,
        maybe_having,
        Operator::NotIlike
    );

    #[inline]
    pub(crate) fn where_grouped_expr(
        &mut self,
        group_conj: Conjunction,
        conj: Conjunction,
        projections: Projections,
        value: Expr,
        operator: Operator,
    ) -> &mut Self {
        self.where_group_expr(group_conj, |builder| {
            for proj in projections {
                // todo: instead of cloning, i could put the same placeholder value (in pg and
                // sqlite) and refer to the same.
                Builder::push_binary_expr(
                    &mut builder.binds,
                    builder.maybe_where.get_or_insert_default(),
                    conj,
                    Expr::Ident(proj),
                    operator,
                    value.clone(),
                );
            }
        });
        self
    }

    #[inline]
    pub(crate) fn where_exists_expr(
        &mut self,
        conj: Conjunction,
        operator: ExistsOperator,
        mut rhs: Builder,
    ) -> &mut Self {
        let expr = self.maybe_where.get_or_insert_default();
        self.binds.append(rhs.take_bindings());
        let cond = ExistsCondition {
            operator,
            subquery: Box::new(rhs),
        };
        let kind = ConditionKind::Exists(cond);
        let cond = Condition::new(conj, kind);
        expr.push(cond);
        self
    }

    #[inline]
    pub(crate) fn where_in_expr(
        &mut self,
        conj: Conjunction,
        mut lhs: Expr,
        mut rhs: InList,
        operator: InOperator,
    ) -> &mut Self {
        let expr = self.maybe_where.get_or_insert_default();
        self.binds.append(lhs.take_bindings());
        self.binds.append(rhs.take_bindings());
        let cond = InCondition { operator, lhs, rhs };
        let kind = ConditionKind::In(cond);
        let cond = Condition::new(conj, kind);
        expr.push(cond);
        self
    }

    #[inline]
    pub(crate) fn where_between_expr(
        &mut self,
        conj: Conjunction,
        mut lhs: Expr,
        mut low: Expr,
        mut high: Expr,
        operator: BetweenOperator,
    ) -> &mut Self {
        let expr = self.maybe_where.get_or_insert_default();
        self.binds.append(lhs.take_bindings());
        self.binds.append(low.take_bindings());
        self.binds.append(high.take_bindings());
        let cond = BetweenCondition {
            lhs,
            low,
            high,
            operator,
        };
        let kind = ConditionKind::Between(cond);
        let cond = Condition::new(conj, kind);
        expr.push(cond);
        self
    }

    #[inline]
    pub(crate) fn where_unary_expr(
        &mut self,
        conj: Conjunction,
        mut lhs: Expr,
        operator: UnaryOperator,
    ) -> &mut Self {
        self.binds.append(lhs.take_bindings());
        let expr = self.maybe_where.get_or_insert_default();
        let cond = UnaryCondition { lhs, operator };
        let kind = ConditionKind::Unary(cond);
        let cond = Condition::new(conj, kind);
        expr.push(cond);
        self
    }

    #[inline]
    pub(crate) fn where_raw_expr(
        &mut self,
        conj: Conjunction,
        rhs: Raw,
        binds: Binds,
    ) -> &mut Self {
        let expr = self.maybe_where.get_or_insert_default();
        self.binds.append(binds);
        let cond = ConditionKind::Raw(rhs);
        let cond = Condition::new(conj, cond);
        expr.push(cond);
        self
    }

    #[inline]
    pub(crate) fn push_binary_expr(
        binds: &mut Binds,
        target: &mut Conditions,
        conjunction: Conjunction,
        mut lhs: Expr,
        operator: Operator,
        mut rhs: Expr,
    ) {
        binds.append(lhs.take_bindings());
        binds.append(rhs.take_bindings());
        let binary = BinaryCondition { lhs, operator, rhs };
        let expr = ConditionKind::Binary(binary);
        let condition = Condition::new(conjunction, expr);
        target.push(condition);
    }

    #[inline]
    pub(crate) fn where_group_expr<F>(&mut self, conjunction: Conjunction, closure: F) -> &mut Self
    where
        F: FnOnce(&mut Self),
    {
        // with a type of where, we should ignore most actions
        let mut inner = Self {
            ty: QueryKind::Where,
            ..Default::default()
        };
        // modify the internal states with wheres
        closure(&mut inner);

        let binds = inner.take_bindings();
        if let Some(inner_conds) = inner.maybe_where {
            self.binds.append(binds);

            let group = GroupCondition {
                conditions: inner_conds,
            };
            let kind = ConditionKind::Group(group);

            let ws = self.maybe_where.get_or_insert_default();
            ws.push(Condition::new(conjunction, kind));
        }

        self
    }

    // add order by stuff

    pub fn order_by_asc<I: IntoTable>(&mut self, column: I) -> &mut Self {
        self.order_by_expr(column.into_table(), Ordering::Asc)
    }

    pub fn order_by_desc<I: IntoTable>(&mut self, column: I) -> &mut Self {
        self.order_by_expr(column.into_table(), Ordering::Desc)
    }

    pub fn latest<I: IntoTable>(&mut self, column: I) -> &mut Self {
        self.order_by_desc(column)
    }

    pub fn oldest<I: IntoTable>(&mut self, column: I) -> &mut Self {
        self.order_by_asc(column)
    }

    pub fn reset_order(&mut self) -> &mut Self {
        self.maybe_order = None;
        self
    }

    pub fn order_by_raw<R: IntoRaw>(&mut self, raw: R) -> &mut Self {
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
    pub(crate) fn order_by_expr(&mut self, ident: TableRef, order: Ordering) -> &mut Self {
        let o = self.maybe_order.get_or_insert_default();
        o.push_proj(ident, order);
        self
    }

    // start of expr stuff

    // select stuff

    pub fn select_raw<T: IntoRaw, B: IntoBinds>(&mut self, value: T, binds: B) -> &mut Self {
        let raw = value.into_raw();
        self.projections = Projections::One(TableRef::Raw(raw));
        self.binds.append(binds.into_binds());
        self
    }

    pub fn select_as<T: ProjectionSchema>(&mut self) -> &mut Self {
        self.projections = T::projections();
        self
    }

    pub fn select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoProjections,
    {
        self.projections = cols.into_projections();
        self
    }

    pub fn add_select<T>(&mut self, cols: T) -> &mut Self
    where
        T: IntoProjections,
    {
        let other = cols.into_projections();
        self.projections.append(other);
        self
    }

    pub fn reset_select(&mut self) -> &mut Self {
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
        context.writer.write_str("select ")?;
        if self.distinct {
            context.writer.write_str(" distinct ")?;
        }
        self.projections.format_writer(context)?;
        if let Some(ref table) = self.maybe_table {
            context.writer.write_str(" from ")?;
            table.format_writer(context)?;
        }

        if let Some(ref w) = self.maybe_where {
            // if we are not in a where group
            if !w.0.is_empty() && matches!(self.ty, QueryKind::Select) {
                context.writer.write_str(" where ")?;
            }
            w.format_writer(context)?;
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
        expr::{IntoLhsExpr, IntoRhsExpr},
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
        let mut builder = Builder::table_as::<User>();
        builder.select_as::<User>();
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
        builder.from_sub("foo", |builder| {
            builder.where_eq("username", "foo").from("bar");
        });
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
        assert_eq!(
            "select * from \"users\" limit 1",
            builder.to_sql::<Postgres>()
        );
    }

    #[test]
    fn test_offset_clause() {
        let mut builder = Builder::table("users");
        builder.offset(42);
        assert_eq!(
            "select * from \"users\" offset 42",
            builder.to_sql::<Postgres>()
        );
    }
}
