#![feature(prelude_import)]
#![allow(dead_code)]
#[prelude_import]
use std::prelude::rust_2024::*;
#[macro_use]
extern crate std;
mod dialect {
    pub enum Dialect {
        Postgres,
        MySql,
        Sqlite,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Dialect {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    Dialect::Postgres => "Postgres",
                    Dialect::MySql => "MySql",
                    Dialect::Sqlite => "Sqlite",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Dialect {
        #[inline]
        fn clone(&self) -> Dialect {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for Dialect {}
    pub trait HasDialect {
        const DIALECT: Dialect;
    }
    pub struct Postgres;
    impl HasDialect for Postgres {
        const DIALECT: Dialect = Dialect::Postgres;
    }
    pub struct MySql;
    impl HasDialect for MySql {
        const DIALECT: Dialect = Dialect::MySql;
    }
    pub struct Sqlite;
    impl HasDialect for Sqlite {
        const DIALECT: Dialect = Dialect::Sqlite;
    }
}
mod writer {
    use std::{fmt::Write, ops::Deref};
    use crate::dialect::Dialect;
    pub(crate) trait FormatWriter {
        fn format_writer<W: Write>(
            &self,
            context: &mut FormatContext<'_, W>,
        ) -> std::fmt::Result;
    }
    pub(crate) struct FormatContext<'a, W: Write> {
        pub(crate) writer: &'a mut W,
        pub(crate) dialect: Dialect,
        pub(crate) placeholder: u16,
    }
    impl<'a, W: Write> FormatContext<'a, W> {
        pub fn new(writer: &'a mut W, dialect: Dialect) -> Self {
            Self {
                writer,
                dialect,
                placeholder: 0,
            }
        }
        pub(crate) fn write_table(&mut self, ident: &str) -> std::fmt::Result {
            for (i, part) in ident.split('.').enumerate() {
                if i > 0 {
                    self.writer.write_char('.')?;
                }
                self.write_ident(part)?;
            }
            Ok(())
        }
        pub(crate) fn write_ident(&mut self, part: &str) -> std::fmt::Result {
            let quote = match self.dialect {
                Dialect::Postgres | Dialect::Sqlite => '"',
                Dialect::MySql => '`',
            };
            self.writer.write_char(quote)?;
            let dbl = if quote == '"' { "\"\"" } else { "``" };
            let mut last = 0;
            for (index, char) in part.char_indices() {
                if char == quote {
                    if index != last {
                        self.writer.write_str(&part[last..index])?;
                    }
                    self.writer.write_str(dbl)?;
                    last = index + char.len_utf8();
                }
            }
            if last < part.len() {
                self.writer.write_str(&part[last..])?;
            }
            self.writer.write_char(quote)?;
            Ok(())
        }
        pub(crate) fn write_placeholder(&mut self) -> std::fmt::Result {
            self.placeholder += 1;
            self.writer.write_fmt(format_args!("${0}", self.placeholder))?;
            Ok(())
        }
    }
    impl<D> FormatWriter for D
    where
        D: Deref,
        D::Target: FormatWriter,
    {
        fn format_writer<W: std::fmt::Write>(
            &self,
            ctx: &mut FormatContext<'_, W>,
        ) -> std::fmt::Result {
            self.deref().format_writer(ctx)
        }
    }
}
mod ident {
    use smol_str::SmolStr;
    use crate::{
        bind::Array, raw::Raw, scalar::TakeBindings, writer::{self, FormatWriter},
    };
    pub enum TableIdent {
        Ident(Ident),
        Raw(Raw),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for TableIdent {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                TableIdent::Ident(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Ident",
                        &__self_0,
                    )
                }
                TableIdent::Raw(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Raw",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for TableIdent {
        #[inline]
        fn clone(&self) -> TableIdent {
            match self {
                TableIdent::Ident(__self_0) => {
                    TableIdent::Ident(::core::clone::Clone::clone(__self_0))
                }
                TableIdent::Raw(__self_0) => {
                    TableIdent::Raw(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    impl TakeBindings for TableIdent {
        fn take_bindings(&mut self) -> crate::Binds {
            match self {
                TableIdent::Ident(_) => Array::None,
                TableIdent::Raw(_) => Array::None,
            }
        }
    }
    impl TableIdent {
        pub fn ident_static(value: &'static str) -> Self {
            Self::Ident(Ident::new_static(value))
        }
        pub fn ident<T>(value: T) -> Self
        where
            T: Into<SmolStr>,
        {
            Self::Ident(Ident::new(value))
        }
        pub fn raw<T>(value: T) -> Self
        where
            T: Into<SmolStr>,
        {
            Self::Raw(Raw::new(value))
        }
        pub fn raw_static(value: &'static str) -> Self {
            Self::Raw(Raw::new_static(value))
        }
    }
    impl FormatWriter for TableIdent {
        fn format_writer<W: std::fmt::Write>(
            &self,
            context: &mut writer::FormatContext<'_, W>,
        ) -> std::fmt::Result {
            match self {
                TableIdent::Ident(ident) => ident.format_writer(context),
                TableIdent::Raw(raw) => raw.format_writer(context),
            }
        }
    }
    pub struct Ident(SmolStr);
    #[automatically_derived]
    impl ::core::fmt::Debug for Ident {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Ident", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Ident {
        #[inline]
        fn clone(&self) -> Ident {
            Ident(::core::clone::Clone::clone(&self.0))
        }
    }
    impl Ident {
        #[inline]
        pub fn new<T>(value: T) -> Self
        where
            T: Into<SmolStr>,
        {
            Self(value.into())
        }
        #[inline]
        pub fn new_static(value: &'static str) -> Self {
            Self(SmolStr::new_static(value))
        }
    }
    impl FormatWriter for Ident {
        fn format_writer<W: std::fmt::Write>(
            &self,
            context: &mut writer::FormatContext<'_, W>,
        ) -> std::fmt::Result {
            let table = self.0.as_str();
            if let Some(index) = find_as(table.as_bytes()) {
                let (lhs, rhs) = table.split_at(index);
                let alias = &rhs[4..];
                context.write_table(lhs)?;
                context.writer.write_str(" as ")?;
                context.write_ident(alias)?;
                return Ok(());
            }
            context.write_table(table)?;
            Ok(())
        }
    }
    /// Return the index of " as " in bytes case insensitive with no allocations.
    fn find_as(h: &[u8]) -> Option<usize> {
        if h.len() < 4 {
            return None;
        }
        for (i, w) in h.windows(4).enumerate() {
            if w[0] == b' ' && w[3] == b' ' && (w[1] | 0x20) == b'a'
                && (w[2] | 0x20) == b's'
            {
                return Some(i);
            }
        }
        None
    }
}
mod raw {
    use smol_str::SmolStr;
    use crate::{dialect::Dialect, writer::{self, FormatWriter}};
    pub struct Raw(SmolStr);
    #[automatically_derived]
    impl ::core::fmt::Debug for Raw {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Raw", &&self.0)
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Raw {
        #[inline]
        fn clone(&self) -> Raw {
            Raw(::core::clone::Clone::clone(&self.0))
        }
    }
    impl Raw {
        pub fn new<T>(value: T) -> Self
        where
            T: Into<SmolStr>,
        {
            Self(value.into())
        }
        pub fn new_static(value: &'static str) -> Self {
            Self(SmolStr::new_static(value))
        }
    }
    impl FormatWriter for Raw {
        fn format_writer<W: std::fmt::Write>(
            &self,
            context: &mut writer::FormatContext<'_, W>,
        ) -> std::fmt::Result {
            let sql = self.0.as_str();
            if !match context.dialect {
                Dialect::Postgres => true,
                _ => false,
            } {
                return context.writer.write_str(sql);
            }
            enum State {
                Normal,
                Ident,
                Lit,
                Tag,
            }
            let mut state = State::Normal;
            let mut span_start = 0;
            let mut lit_start = 0;
            let mut lit_end = 0;
            let mut ident_start = 0;
            let mut ident_end = 0;
            let mut tag_start = 0;
            let mut tag_end = 0;
            let max_len = sql.len();
            let mut chars = sql.char_indices().peekable();
            while let Some((index, char)) = chars.next() {
                match state {
                    State::Normal => {
                        match char {
                            '\'' => {
                                context.writer.write_str(&sql[span_start..index])?;
                                lit_start = index;
                                lit_end = index + char.len_utf8();
                                span_start = index;
                                state = State::Lit;
                            }
                            '"' => {
                                context.writer.write_str(&sql[span_start..index])?;
                                ident_start = index;
                                ident_end = index + char.len_utf8();
                                span_start = index;
                                state = State::Ident;
                            }
                            '?' => {
                                let is_placeholder = if let Some(&(_, next_ch)) = chars
                                    .peek()
                                {
                                    next_ch != '?' && next_ch != '|' && next_ch != '&'
                                } else {
                                    true
                                };
                                if let Some(&(_, next_ch)) = chars.peek() {
                                    if next_ch == '?' || next_ch == '|' || next_ch == '&' {
                                        let _ = chars.next();
                                        continue;
                                    }
                                }
                                if is_placeholder {
                                    context.writer.write_str(&sql[span_start..index])?;
                                    context.write_placeholder()?;
                                    span_start = index + char.len_utf8();
                                }
                            }
                            '$' => {
                                context.writer.write_str(&sql[span_start..index])?;
                                tag_start = index;
                                tag_end = index + char.len_utf8();
                                span_start = index;
                                state = State::Tag;
                            }
                            _ => {}
                        }
                    }
                    State::Ident => {
                        while let Some(&(next_idx, next_ch)) = chars.peek() {
                            let w = next_ch.len_utf8();
                            chars.next();
                            ident_end = next_idx + w;
                            if next_ch == '"' {
                                if let Some(&(_, '"')) = chars.peek() {
                                    if let Some((esc_idx, _)) = chars.next() {
                                        ident_end = esc_idx + w;
                                        continue;
                                    }
                                }
                                state = State::Normal;
                                break;
                            }
                        }
                        context.writer.write_str(&sql[ident_start..ident_end])?;
                        span_start = ident_end;
                    }
                    State::Lit => {
                        while let Some(&(next_idx, next_ch)) = chars.peek() {
                            let w = next_ch.len_utf8();
                            chars.next();
                            lit_end = next_idx + w;
                            if next_ch == '\'' {
                                if let Some(&(_, '\'')) = chars.peek() {
                                    if let Some((esc_idx, _)) = chars.next() {
                                        lit_end = esc_idx + w;
                                        continue;
                                    }
                                }
                                state = State::Normal;
                                break;
                            }
                        }
                        context.writer.write_str(&sql[lit_start..lit_end])?;
                        span_start = lit_end;
                    }
                    State::Tag => {
                        while let Some(&(next_idx, next_ch)) = chars.peek() {
                            let w = next_ch.len_utf8();
                            chars.next();
                            tag_end = next_idx + w;
                            if next_ch == '$' {
                                state = State::Normal;
                                break;
                            }
                        }
                        context.writer.write_str(&sql[tag_start..tag_end])?;
                        span_start = tag_end;
                    }
                }
            }
            if span_start < max_len {
                context.writer.write_str(&sql[span_start..])?;
            }
            Ok(())
        }
    }
    pub trait IntoRaw {
        fn into_raw(self) -> Raw;
    }
    impl<T> IntoRaw for T
    where
        T: Into<SmolStr>,
    {
        fn into_raw(self) -> Raw {
            Raw::new(self.into())
        }
    }
    impl IntoRaw for Raw {
        fn into_raw(self) -> Raw {
            self
        }
    }
}
mod builder {
    use crate::{
        bind::{Binds, IntoBinds},
        col::{ColumnSchema, Columns, IntoColumns, IntoTable, TableSchema},
        dialect::HasDialect,
        expr::{
            between::{BetweenCondition, BetweenOperator},
            binary::{BinaryCondition, Operator},
            cond::{Condition, Conditions, Conjunction},
            group::GroupCondition, unary::{UnaryCondition, UnaryOperator},
            ConditionKind,
        },
        ident::TableIdent, raw::IntoRaw,
        scalar::{IntoOperator, IntoScalar, IntoScalarIdent, ScalarExpr, TakeBindings},
        writer::{FormatContext, FormatWriter},
        Raw,
    };
    pub enum QueryKind {
        #[default]
        Select,
        Where,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for QueryKind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    QueryKind::Select => "Select",
                    QueryKind::Where => "Where",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for QueryKind {
        #[inline]
        fn default() -> QueryKind {
            Self::Select
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for QueryKind {
        #[inline]
        fn clone(&self) -> QueryKind {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for QueryKind {}
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for QueryKind {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for QueryKind {
        #[inline]
        fn eq(&self, other: &QueryKind) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for QueryKind {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_receiver_is_total_eq(&self) -> () {}
    }
    impl TakeBindings for Builder {
        fn take_bindings(&mut self) -> Binds {
            std::mem::take(&mut self.binds)
        }
    }
    pub struct Builder {
        query: String,
        ty: QueryKind,
        distinct: bool,
        maybe_table: Option<TableIdent>,
        columns: Columns,
        binds: Binds,
        maybe_where: Option<Conditions>,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Builder {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "query",
                "ty",
                "distinct",
                "maybe_table",
                "columns",
                "binds",
                "maybe_where",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.query,
                &self.ty,
                &self.distinct,
                &self.maybe_table,
                &self.columns,
                &self.binds,
                &&self.maybe_where,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "Builder",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::default::Default for Builder {
        #[inline]
        fn default() -> Builder {
            Builder {
                query: ::core::default::Default::default(),
                ty: ::core::default::Default::default(),
                distinct: ::core::default::Default::default(),
                maybe_table: ::core::default::Default::default(),
                columns: ::core::default::Default::default(),
                binds: ::core::default::Default::default(),
                maybe_where: ::core::default::Default::default(),
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Builder {
        #[inline]
        fn clone(&self) -> Builder {
            Builder {
                query: ::core::clone::Clone::clone(&self.query),
                ty: ::core::clone::Clone::clone(&self.ty),
                distinct: ::core::clone::Clone::clone(&self.distinct),
                maybe_table: ::core::clone::Clone::clone(&self.maybe_table),
                columns: ::core::clone::Clone::clone(&self.columns),
                binds: ::core::clone::Clone::clone(&self.binds),
                maybe_where: ::core::clone::Clone::clone(&self.maybe_where),
            }
        }
    }
    impl Builder {
        pub fn table_as<T: TableSchema>() -> Self {
            Self {
                query: String::new(),
                distinct: false,
                maybe_table: Some(T::table()),
                columns: Columns::None,
                binds: Binds::None,
                ty: QueryKind::Select,
                maybe_where: None,
            }
        }
        pub fn table<T>(table: T) -> Self
        where
            T: IntoTable,
        {
            Self {
                query: String::new(),
                distinct: false,
                maybe_table: Some(table.into_table()),
                columns: Columns::None,
                binds: Binds::None,
                ty: QueryKind::Select,
                maybe_where: None,
            }
        }
        pub fn from<T: IntoTable>(&mut self, table: T) -> &mut Self {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            self.maybe_table = Some(table.into_table());
            self
        }
        pub fn when<F>(&mut self, condition: bool, builder: F) -> &mut Self
        where
            F: FnOnce(&mut Self),
        {
            if condition {
                builder(self);
            }
            self
        }
        pub fn when_some<T, F>(
            &mut self,
            maybe_value: Option<T>,
            builder: F,
        ) -> &mut Self
        where
            F: FnOnce(&mut Self, T),
        {
            if let Some(value) = maybe_value {
                builder(self, value);
            }
            self
        }
        pub fn where_operator<C, O, V>(
            &mut self,
            column: C,
            operator: O,
            value: V,
        ) -> &mut Self
        where
            C: IntoScalarIdent,
            O: IntoOperator,
            V: IntoScalar,
        {
            self.where_binary_expr(
                Conjunction::And,
                column.into_scalar_ident().0,
                operator.into_operator(),
                value.into_scalar().0,
            )
        }
        pub fn or_where_operator<C, O, V>(
            &mut self,
            column: C,
            operator: O,
            value: V,
        ) -> &mut Self
        where
            C: IntoScalarIdent,
            O: IntoOperator,
            V: IntoScalar,
        {
            self.where_binary_expr(
                Conjunction::Or,
                column.into_scalar_ident().0,
                operator.into_operator(),
                value.into_scalar().0,
            )
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
        #[inline]
        pub(crate) fn where_between_expr(
            &mut self,
            conj: Conjunction,
            mut lhs: ScalarExpr,
            mut low: ScalarExpr,
            mut high: ScalarExpr,
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
            self
        }
        #[inline]
        pub(crate) fn where_unary_expr(
            &mut self,
            conj: Conjunction,
            mut lhs: ScalarExpr,
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
        pub(crate) fn where_binary_expr(
            &mut self,
            conjunction: Conjunction,
            mut lhs: ScalarExpr,
            operator: Operator,
            mut rhs: ScalarExpr,
        ) -> &mut Self {
            self.binds.append(lhs.take_bindings());
            self.binds.append(rhs.take_bindings());
            let binary = BinaryCondition {
                lhs,
                operator,
                rhs,
            };
            let expr = ConditionKind::Binary(binary);
            let condition = Condition::new(conjunction, expr);
            let ws = self.maybe_where.get_or_insert_default();
            ws.push(condition);
            self
        }
        #[inline]
        pub(crate) fn where_group_expr<F>(
            &mut self,
            conjunction: Conjunction,
            closure: F,
        ) -> &mut Self
        where
            F: FnOnce(&mut Self),
        {
            let mut inner = Self {
                ty: QueryKind::Where,
                ..Default::default()
            };
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
        pub fn select_raw<T: IntoRaw, B: IntoBinds>(
            &mut self,
            value: T,
            binds: B,
        ) -> &mut Self {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            let raw = value.into_raw();
            self.columns = Columns::One(TableIdent::Raw(raw));
            self.binds.append(binds.into_binds());
            self
        }
        pub fn select_as<T: ColumnSchema>(&mut self) -> &mut Self {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            self.columns = T::columns();
            self
        }
        pub fn select<T>(&mut self, cols: T) -> &mut Self
        where
            T: IntoColumns,
        {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            self.columns = cols.into_columns();
            self
        }
        pub fn add_select<T>(&mut self, cols: T) -> &mut Self
        where
            T: IntoColumns,
        {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            let other = cols.into_columns();
            self.columns.append(other);
            self
        }
        pub fn reset_select(&mut self) -> &mut Self {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            self.columns.reset();
            self
        }
        pub fn distinct(&mut self) -> &mut Self {
            if match self.ty {
                QueryKind::Where => true,
                _ => false,
            } {
                return self;
            }
            self.distinct = true;
            self
        }
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
            self.columns.format_writer(context)?;
            if let Some(ref table) = self.maybe_table {
                context.writer.write_str(" from ")?;
                table.format_writer(context)?;
            }
            if let Some(ref w) = self.maybe_where {
                if !w.0.is_empty()
                    && match self.ty {
                        QueryKind::Select => true,
                        _ => false,
                    }
                {
                    context.writer.write_str(" where ")?;
                }
                w.format_writer(context)?;
            }
            Ok(())
        }
    }
}
mod col {
    use std::fmt;
    use crate::{
        bind::Array, ident::{Ident, TableIdent},
        writer::FormatWriter, Raw,
    };
    pub type Columns = Array<TableIdent>;
    impl FormatWriter for Columns {
        fn format_writer<W: fmt::Write>(
            &self,
            context: &mut crate::writer::FormatContext<'_, W>,
        ) -> fmt::Result {
            match self {
                Columns::None => context.writer.write_char('*')?,
                Columns::One(ident) => ident.format_writer(context)?,
                Columns::Many(idents) => {
                    for (index, elem) in idents.iter().enumerate() {
                        if index > 0 {
                            context.writer.write_str(", ")?;
                        }
                        elem.format_writer(context)?;
                    }
                }
            };
            Ok(())
        }
    }
    pub trait TableSchema {
        fn table() -> TableIdent;
    }
    pub trait ColumnSchema {
        fn columns() -> Columns;
    }
    pub trait IntoColumns {
        fn into_columns(self) -> Columns;
    }
    pub trait IntoTable {
        fn into_table(self) -> TableIdent;
    }
    impl IntoTable for &str {
        fn into_table(self) -> TableIdent {
            TableIdent::ident(self)
        }
    }
    impl IntoTable for String {
        fn into_table(self) -> TableIdent {
            TableIdent::ident(self)
        }
    }
    impl IntoTable for Raw {
        fn into_table(self) -> TableIdent {
            TableIdent::Raw(self)
        }
    }
    impl IntoTable for Ident {
        fn into_table(self) -> TableIdent {
            TableIdent::Ident(self)
        }
    }
    impl IntoTable for TableIdent {
        fn into_table(self) -> TableIdent {
            self
        }
    }
    impl<T: TableSchema> IntoTable for T {
        fn into_table(self) -> TableIdent {
            T::table()
        }
    }
    impl IntoColumns for &str {
        fn into_columns(self) -> Columns {
            Columns::One(self.into_table())
        }
    }
    impl IntoColumns for String {
        fn into_columns(self) -> Columns {
            Columns::One(self.into_table())
        }
    }
    impl IntoColumns for Raw {
        fn into_columns(self) -> Columns {
            Columns::One(self.into_table())
        }
    }
    impl IntoColumns for Ident {
        fn into_columns(self) -> Columns {
            Columns::One(self.into_table())
        }
    }
    impl IntoColumns for TableIdent {
        fn into_columns(self) -> Columns {
            Columns::One(self.into_table())
        }
    }
    impl<const N: usize> IntoColumns for [&str; N] {
        fn into_columns(self) -> Columns {
            if N == 1 {
                Columns::One(self[0].into_table())
            } else {
                let vec: Vec<TableIdent> = self.map(|t| t.into_table()).to_vec();
                Columns::Many(vec)
            }
        }
    }
    impl<const N: usize> IntoColumns for [String; N] {
        fn into_columns(self) -> Columns {
            let vec: Vec<TableIdent> = self.map(|t| t.into_table()).to_vec();
            Columns::Many(vec)
        }
    }
    impl<const N: usize> IntoColumns for [Ident; N] {
        fn into_columns(self) -> Columns {
            if N == 1 {
                Columns::One(self[0].clone().into_table())
            } else {
                let vec: Vec<TableIdent> = self.map(|t| t.into_table()).to_vec();
                Columns::Many(vec)
            }
        }
    }
    impl<const N: usize> IntoColumns for [Raw; N] {
        fn into_columns(self) -> Columns {
            if N == 1 {
                Columns::One(self[0].clone().into_table())
            } else {
                let vec: Vec<TableIdent> = self.map(|t| t.into_table()).to_vec();
                Columns::Many(vec)
            }
        }
    }
    impl<const N: usize> IntoColumns for [TableIdent; N] {
        fn into_columns(self) -> Columns {
            if N == 1 {
                Columns::One(self[0].clone())
            } else {
                let vec: Vec<TableIdent> = self.to_vec();
                Columns::Many(vec)
            }
        }
    }
    impl IntoColumns for Vec<&str> {
        fn into_columns(self) -> Columns {
            let vec = self.into_iter().map(|t| t.into_table()).collect();
            Columns::Many(vec)
        }
    }
    impl IntoColumns for Vec<String> {
        fn into_columns(self) -> Columns {
            let vec = self.into_iter().map(|t| t.into_table()).collect();
            Columns::Many(vec)
        }
    }
    impl IntoColumns for Vec<Ident> {
        fn into_columns(self) -> Columns {
            let vec = self.into_iter().map(|t| t.into_table()).collect();
            Columns::Many(vec)
        }
    }
    impl IntoColumns for Vec<Raw> {
        fn into_columns(self) -> Columns {
            let vec = self.into_iter().map(|t| t.into_table()).collect();
            Columns::Many(vec)
        }
    }
    impl IntoColumns for Vec<TableIdent> {
        fn into_columns(self) -> Columns {
            let vec = self.into_iter().map(|t| t.into_table()).collect();
            Columns::Many(vec)
        }
    }
    impl IntoColumns for Columns {
        fn into_columns(self) -> Columns {
            self
        }
    }
    impl<T: ColumnSchema> IntoColumns for T {
        fn into_columns(self) -> Columns {
            T::columns()
        }
    }
}
mod bind {
    pub enum Bind {
        Null,
        Consumed,
        String(String),
        StaticString(&'static str),
        Bool(bool),
        F32(f32),
        F64(f64),
        I8(i8),
        I16(i16),
        I32(i32),
        I64(i64),
        U8(u8),
        U16(u16),
        U32(u32),
        U64(u64),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Bind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Bind::Null => ::core::fmt::Formatter::write_str(f, "Null"),
                Bind::Consumed => ::core::fmt::Formatter::write_str(f, "Consumed"),
                Bind::String(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "String",
                        &__self_0,
                    )
                }
                Bind::StaticString(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "StaticString",
                        &__self_0,
                    )
                }
                Bind::Bool(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Bool",
                        &__self_0,
                    )
                }
                Bind::F32(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "F32",
                        &__self_0,
                    )
                }
                Bind::F64(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "F64",
                        &__self_0,
                    )
                }
                Bind::I8(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I8", &__self_0)
                }
                Bind::I16(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "I16",
                        &__self_0,
                    )
                }
                Bind::I32(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "I32",
                        &__self_0,
                    )
                }
                Bind::I64(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "I64",
                        &__self_0,
                    )
                }
                Bind::U8(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U8", &__self_0)
                }
                Bind::U16(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "U16",
                        &__self_0,
                    )
                }
                Bind::U32(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "U32",
                        &__self_0,
                    )
                }
                Bind::U64(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "U64",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Bind {
        #[inline]
        fn clone(&self) -> Bind {
            match self {
                Bind::Null => Bind::Null,
                Bind::Consumed => Bind::Consumed,
                Bind::String(__self_0) => {
                    Bind::String(::core::clone::Clone::clone(__self_0))
                }
                Bind::StaticString(__self_0) => {
                    Bind::StaticString(::core::clone::Clone::clone(__self_0))
                }
                Bind::Bool(__self_0) => Bind::Bool(::core::clone::Clone::clone(__self_0)),
                Bind::F32(__self_0) => Bind::F32(::core::clone::Clone::clone(__self_0)),
                Bind::F64(__self_0) => Bind::F64(::core::clone::Clone::clone(__self_0)),
                Bind::I8(__self_0) => Bind::I8(::core::clone::Clone::clone(__self_0)),
                Bind::I16(__self_0) => Bind::I16(::core::clone::Clone::clone(__self_0)),
                Bind::I32(__self_0) => Bind::I32(::core::clone::Clone::clone(__self_0)),
                Bind::I64(__self_0) => Bind::I64(::core::clone::Clone::clone(__self_0)),
                Bind::U8(__self_0) => Bind::U8(::core::clone::Clone::clone(__self_0)),
                Bind::U16(__self_0) => Bind::U16(::core::clone::Clone::clone(__self_0)),
                Bind::U32(__self_0) => Bind::U32(::core::clone::Clone::clone(__self_0)),
                Bind::U64(__self_0) => Bind::U64(::core::clone::Clone::clone(__self_0)),
            }
        }
    }
    impl Bind {
        pub fn new<V>(value: V) -> Bind
        where
            V: IntoBind,
        {
            value.into_bind()
        }
        pub fn new_static_str(value: &'static str) -> Bind {
            Bind::StaticString(value)
        }
    }
    pub type Binds = Array<Bind>;
    impl IntoBinds for Binds {
        fn into_binds(self) -> Binds {
            self
        }
    }
    pub enum Array<T> {
        #[default]
        None,
        One(T),
        Many(Vec<T>),
    }
    #[automatically_derived]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for Array<T> {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                Array::None => ::core::fmt::Formatter::write_str(f, "None"),
                Array::One(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "One",
                        &__self_0,
                    )
                }
                Array::Many(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Many",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl<T> ::core::default::Default for Array<T> {
        #[inline]
        fn default() -> Array<T> {
            Self::None
        }
    }
    #[automatically_derived]
    impl<T: ::core::clone::Clone> ::core::clone::Clone for Array<T> {
        #[inline]
        fn clone(&self) -> Array<T> {
            match self {
                Array::None => Array::None,
                Array::One(__self_0) => Array::One(::core::clone::Clone::clone(__self_0)),
                Array::Many(__self_0) => {
                    Array::Many(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    pub enum ArrayIter<'a, T> {
        None,
        One(Option<&'a T>),
        Many(std::slice::Iter<'a, T>),
    }
    impl<'a, T> Iterator for ArrayIter<'a, T> {
        type Item = &'a T;
        fn next(&mut self) -> Option<Self::Item> {
            match self {
                ArrayIter::None => None,
                ArrayIter::One(o) => o.take(),
                ArrayIter::Many(i) => i.next(),
            }
        }
    }
    pub enum ArrayIterMut<'a, T> {
        None,
        One(Option<&'a mut T>),
        Many(std::slice::IterMut<'a, T>),
    }
    impl<'a, T> Iterator for ArrayIterMut<'a, T> {
        type Item = &'a mut T;
        fn next(&mut self) -> Option<Self::Item> {
            match self {
                ArrayIterMut::None => None,
                ArrayIterMut::One(o) => o.take(),
                ArrayIterMut::Many(i) => i.next(),
            }
        }
    }
    impl<'a, T> IntoIterator for &'a Array<T> {
        type Item = &'a T;
        type IntoIter = ArrayIter<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }
    impl<'a, T> IntoIterator for &'a mut Array<T> {
        type Item = &'a mut T;
        type IntoIter = ArrayIterMut<'a, T>;
        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    }
    impl<T> Array<T> {
        pub fn iter(&self) -> ArrayIter<'_, T> {
            match self {
                Array::None => ArrayIter::None,
                Array::One(x) => ArrayIter::One(Some(x)),
                Array::Many(xs) => ArrayIter::Many(xs.iter()),
            }
        }
        pub fn iter_mut(&mut self) -> ArrayIterMut<'_, T> {
            match self {
                Array::None => ArrayIterMut::None,
                Array::One(x) => ArrayIterMut::One(Some(x)),
                Array::Many(xs) => ArrayIterMut::Many(xs.iter_mut()),
            }
        }
        pub fn append(&mut self, other: Self) {
            let combined = match (std::mem::replace(self, Self::None), other) {
                (Self::None, cols) | (cols, Self::None) => cols,
                (Self::One(a), Self::One(b)) => {
                    Self::Many(
                        <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([a, b])),
                    )
                }
                (Self::One(a), Self::Many(mut b)) => {
                    b.insert(0, a);
                    Self::Many(b)
                }
                (Self::Many(mut a), Self::One(b)) => {
                    a.push(b);
                    Self::Many(a)
                }
                (Self::Many(mut a), Self::Many(mut b)) => {
                    a.append(&mut b);
                    Self::Many(a)
                }
            };
            *self = combined;
        }
        pub fn len(&self) -> usize {
            match self {
                Array::None => 0,
                Array::One(_) => 1,
                Array::Many(items) => items.len(),
            }
        }
        pub fn reset(&mut self) {
            *self = Self::None;
        }
        pub fn into_vec(self) -> Vec<T> {
            match self {
                Self::None => Vec::new(),
                Self::One(one) => Vec::from([one]),
                Self::Many(many) => many,
            }
        }
    }
    pub trait IntoBind {
        fn into_bind(self) -> Bind;
    }
    pub trait IntoBinds {
        fn into_binds(self) -> Binds;
    }
    impl<T> IntoBinds for T
    where
        T: IntoBind,
    {
        fn into_binds(self) -> Binds {
            Binds::One(self.into_bind())
        }
    }
    impl<T> IntoBinds for Vec<T>
    where
        T: IntoBind,
    {
        fn into_binds(self) -> Binds {
            Binds::Many(self.into_iter().map(IntoBind::into_bind).collect())
        }
    }
    impl<T, const N: usize> IntoBinds for [T; N]
    where
        T: IntoBind,
    {
        fn into_binds(self) -> Binds {
            let mut iter = self.into_iter().map(IntoBind::into_bind);
            match N {
                0 => Binds::None,
                1 => {
                    let one = iter.next().expect("safe since N is 1");
                    Binds::One(one)
                }
                _ => Binds::Many(iter.collect()),
            }
        }
    }
    impl<T> IntoBind for Option<T>
    where
        T: IntoBind,
    {
        fn into_bind(self) -> Bind {
            if let Some(value) = self { value.into_bind() } else { Bind::Null }
        }
    }
    impl IntoBind for i32 {
        fn into_bind(self) -> Bind {
            Bind::I32(self)
        }
    }
}
mod scalar {
    use crate::{
        bind::{Array, Bind},
        expr::binary::Operator, writer::FormatWriter, Binds, Builder, Ident, IntoBind,
        IntoTable, Raw, TableIdent,
    };
    pub enum ScalarExpr {
        Bind(Bind),
        Ident(TableIdent),
        Subquery(Box<Builder>),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ScalarExpr {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ScalarExpr::Bind(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Bind",
                        &__self_0,
                    )
                }
                ScalarExpr::Ident(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Ident",
                        &__self_0,
                    )
                }
                ScalarExpr::Subquery(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Subquery",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ScalarExpr {
        #[inline]
        fn clone(&self) -> ScalarExpr {
            match self {
                ScalarExpr::Bind(__self_0) => {
                    ScalarExpr::Bind(::core::clone::Clone::clone(__self_0))
                }
                ScalarExpr::Ident(__self_0) => {
                    ScalarExpr::Ident(::core::clone::Clone::clone(__self_0))
                }
                ScalarExpr::Subquery(__self_0) => {
                    ScalarExpr::Subquery(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    pub trait TakeBindings {
        fn take_bindings(&mut self) -> Binds;
    }
    impl TakeBindings for ScalarExpr {
        fn take_bindings(&mut self) -> Binds {
            match self {
                ScalarExpr::Bind(bind) => {
                    Array::One(std::mem::replace(bind, Bind::Consumed))
                }
                ScalarExpr::Ident(ident) => ident.take_bindings(),
                ScalarExpr::Subquery(builder) => builder.take_bindings(),
            }
        }
    }
    impl FormatWriter for ScalarExpr {
        fn format_writer<W: std::fmt::Write>(
            &self,
            context: &mut crate::writer::FormatContext<'_, W>,
        ) -> std::fmt::Result {
            match self {
                ScalarExpr::Bind(_) => context.write_placeholder(),
                ScalarExpr::Ident(ident) => ident.format_writer(context),
                ScalarExpr::Subquery(builder) => {
                    context.writer.write_char('(')?;
                    builder.format_writer(context)?;
                    context.writer.write_char(')')
                }
            }
        }
    }
    #[repr(transparent)]
    pub struct ScalarIdent(pub(crate) ScalarExpr);
    #[automatically_derived]
    impl ::core::fmt::Debug for ScalarIdent {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "ScalarIdent", &&self.0)
        }
    }
    #[repr(transparent)]
    pub struct Scalar(pub(crate) ScalarExpr);
    #[automatically_derived]
    impl ::core::fmt::Debug for Scalar {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Scalar", &&self.0)
        }
    }
    pub trait IntoScalar {
        fn into_scalar(self) -> Scalar;
    }
    pub trait IntoScalarIdent {
        fn into_scalar_ident(self) -> ScalarIdent;
    }
    pub trait IntoOperator {
        fn into_operator(self) -> Operator;
    }
    impl IntoOperator for Operator {
        fn into_operator(self) -> Operator {
            self
        }
    }
    impl<T> IntoScalar for T
    where
        T: IntoBind,
    {
        fn into_scalar(self) -> Scalar {
            Scalar(ScalarExpr::Bind(self.into_bind()))
        }
    }
    impl IntoScalar for Builder {
        fn into_scalar(self) -> Scalar {
            Scalar(ScalarExpr::Subquery(Box::new(self)))
        }
    }
    impl IntoScalar for Raw {
        fn into_scalar(self) -> Scalar {
            Scalar(ScalarExpr::Ident(TableIdent::Raw(self)))
        }
    }
    impl IntoScalar for Ident {
        fn into_scalar(self) -> Scalar {
            Scalar(ScalarExpr::Ident(TableIdent::Ident(self)))
        }
    }
    impl<T> IntoScalarIdent for T
    where
        T: IntoTable,
    {
        fn into_scalar_ident(self) -> ScalarIdent {
            ScalarIdent(ScalarExpr::Ident(self.into_table()))
        }
    }
    impl IntoScalarIdent for Builder {
        fn into_scalar_ident(self) -> ScalarIdent {
            ScalarIdent(ScalarExpr::Subquery(Box::new(self)))
        }
    }
}
pub mod expr {
    use between::BetweenCondition;
    use binary::BinaryCondition;
    use group::GroupCondition;
    use unary::UnaryCondition;
    use crate::{writer::FormatWriter, Raw};
    pub(crate) mod cond {
        use crate::{expr::ConditionKind, writer::FormatWriter};
        pub enum Conjunction {
            And,
            Or,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Conjunction {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Conjunction::And => "And",
                        Conjunction::Or => "Or",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Conjunction {
            #[inline]
            fn clone(&self) -> Conjunction {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Conjunction {}
        impl FormatWriter for Conjunction {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                match self {
                    Conjunction::And => context.writer.write_str("and"),
                    Conjunction::Or => context.writer.write_str("or"),
                }
            }
        }
        pub struct Condition {
            conjunction: Conjunction,
            kind: ConditionKind,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Condition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "Condition",
                    "conjunction",
                    &self.conjunction,
                    "kind",
                    &&self.kind,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Condition {
            #[inline]
            fn clone(&self) -> Condition {
                Condition {
                    conjunction: ::core::clone::Clone::clone(&self.conjunction),
                    kind: ::core::clone::Clone::clone(&self.kind),
                }
            }
        }
        impl FormatWriter for Condition {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                self.kind.format_writer(context)
            }
        }
        impl Condition {
            pub fn new(conjunction: Conjunction, kind: ConditionKind) -> Self {
                Self { conjunction, kind }
            }
        }
        pub struct Conditions(pub(crate) Vec<Condition>);
        #[automatically_derived]
        impl ::core::fmt::Debug for Conditions {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "Conditions",
                    &&self.0,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for Conditions {
            #[inline]
            fn default() -> Conditions {
                Conditions(::core::default::Default::default())
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Conditions {
            #[inline]
            fn clone(&self) -> Conditions {
                Conditions(::core::clone::Clone::clone(&self.0))
            }
        }
        impl Conditions {
            pub fn push(&mut self, other: Condition) {
                self.0.push(other);
            }
        }
        impl FormatWriter for Conditions {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                for (index, condition) in self.0.iter().enumerate() {
                    if index > 0 {
                        context.writer.write_char(' ')?;
                        condition.conjunction.format_writer(context)?;
                        context.writer.write_char(' ')?;
                    }
                    condition.format_writer(context)?;
                }
                Ok(())
            }
        }
    }
    pub(crate) mod unary {
        use std::fmt::Write;
        use crate::{scalar::ScalarExpr, writer::{FormatContext, FormatWriter}};
        use qraft_derive::UnaryOperator;
        pub enum UnaryOperator {
            Null,
            NotNull,
            True,
            False,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UnaryOperator {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        UnaryOperator::Null => "Null",
                        UnaryOperator::NotNull => "NotNull",
                        UnaryOperator::True => "True",
                        UnaryOperator::False => "False",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UnaryOperator {
            #[inline]
            fn clone(&self) -> UnaryOperator {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for UnaryOperator {}
        impl crate::Builder {
        }
        pub struct UnaryCondition {
            pub(crate) lhs: ScalarExpr,
            pub(crate) operator: UnaryOperator,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UnaryCondition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field2_finish(
                    f,
                    "UnaryCondition",
                    "lhs",
                    &self.lhs,
                    "operator",
                    &&self.operator,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UnaryCondition {
            #[inline]
            fn clone(&self) -> UnaryCondition {
                UnaryCondition {
                    lhs: ::core::clone::Clone::clone(&self.lhs),
                    operator: ::core::clone::Clone::clone(&self.operator),
                }
            }
        }
        impl FormatWriter for UnaryCondition {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                self.lhs.format_writer(context)?;
                context.writer.write_char(' ')?;
                self.operator.format_writer(context)
            }
        }
        impl FormatWriter for UnaryOperator {
            fn format_writer<W: Write>(
                &self,
                context: &mut FormatContext<'_, W>,
            ) -> std::fmt::Result {
                match self {
                    UnaryOperator::Null => context.writer.write_str("is null"),
                    UnaryOperator::NotNull => context.writer.write_str("is not null"),
                    UnaryOperator::True => context.writer.write_str("is true"),
                    UnaryOperator::False => context.writer.write_str("is false"),
                }
            }
        }
    }
    pub(crate) mod binary {
        use qraft_derive::BinaryOperator;
        use crate::{dialect::Dialect, scalar::ScalarExpr, writer::FormatWriter};
        pub struct BinaryCondition {
            pub(crate) lhs: ScalarExpr,
            pub(crate) operator: Operator,
            pub(crate) rhs: ScalarExpr,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BinaryCondition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "BinaryCondition",
                    "lhs",
                    &self.lhs,
                    "operator",
                    &self.operator,
                    "rhs",
                    &&self.rhs,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BinaryCondition {
            #[inline]
            fn clone(&self) -> BinaryCondition {
                BinaryCondition {
                    lhs: ::core::clone::Clone::clone(&self.lhs),
                    operator: ::core::clone::Clone::clone(&self.operator),
                    rhs: ::core::clone::Clone::clone(&self.rhs),
                }
            }
        }
        impl FormatWriter for BinaryCondition {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                self.lhs.format_writer(context)?;
                if let (
                    Dialect::Postgres,
                    Operator::Like
                    | Operator::Ilike
                    | Operator::NotLike
                    | Operator::NotIlike,
                ) = (context.dialect, self.operator) {
                    context.writer.write_str("::text")?;
                }
                context.writer.write_char(' ')?;
                self.operator.format_writer(context)?;
                context.writer.write_char(' ')?;
                self.rhs.format_writer(context)
            }
        }
        pub enum Operator {
            Eq,
            NotEq,
            Lt,
            Lte,
            Gt,
            Gte,
            Like,
            NotLike,
            Ilike,
            NotIlike,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for Operator {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        Operator::Eq => "Eq",
                        Operator::NotEq => "NotEq",
                        Operator::Lt => "Lt",
                        Operator::Lte => "Lte",
                        Operator::Gt => "Gt",
                        Operator::Gte => "Gte",
                        Operator::Like => "Like",
                        Operator::NotLike => "NotLike",
                        Operator::Ilike => "Ilike",
                        Operator::NotIlike => "NotIlike",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for Operator {
            #[inline]
            fn clone(&self) -> Operator {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for Operator {}
        impl crate::Builder {
        }
        impl FormatWriter for Operator {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                match self {
                    Operator::Eq => context.writer.write_char('='),
                    Operator::NotEq => context.writer.write_str("!="),
                    Operator::Lt => context.writer.write_char('<'),
                    Operator::Lte => context.writer.write_str("<="),
                    Operator::Gt => context.writer.write_char('>'),
                    Operator::Gte => context.writer.write_str(">="),
                    Operator::Like => {
                        match context.dialect {
                            Dialect::Postgres => context.writer.write_str("like"),
                            Dialect::MySql => context.writer.write_str("like binary"),
                            Dialect::Sqlite => context.writer.write_str("glob"),
                        }
                    }
                    Operator::NotLike => {
                        match context.dialect {
                            Dialect::Postgres => context.writer.write_str("not like"),
                            Dialect::MySql => context.writer.write_str("not like binary"),
                            Dialect::Sqlite => context.writer.write_str("not glob"),
                        }
                    }
                    Operator::Ilike => {
                        match context.dialect {
                            Dialect::Postgres => context.writer.write_str("ilike"),
                            Dialect::MySql | Dialect::Sqlite => {
                                context.writer.write_str("like")
                            }
                        }
                    }
                    Operator::NotIlike => {
                        match context.dialect {
                            Dialect::Postgres => context.writer.write_str("not ilike"),
                            Dialect::MySql | Dialect::Sqlite => {
                                context.writer.write_str("not like")
                            }
                        }
                    }
                }
            }
        }
    }
    pub(crate) mod group {
        use crate::writer::FormatWriter;
        use super::cond::Conditions;
        pub struct GroupCondition {
            pub(crate) conditions: Conditions,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for GroupCondition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field1_finish(
                    f,
                    "GroupCondition",
                    "conditions",
                    &&self.conditions,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for GroupCondition {
            #[inline]
            fn clone(&self) -> GroupCondition {
                GroupCondition {
                    conditions: ::core::clone::Clone::clone(&self.conditions),
                }
            }
        }
        impl FormatWriter for GroupCondition {
            fn format_writer<W: std::fmt::Write>(
                &self,
                context: &mut crate::writer::FormatContext<'_, W>,
            ) -> std::fmt::Result {
                context.writer.write_char('(')?;
                self.conditions.format_writer(context)?;
                context.writer.write_char(')')?;
                Ok(())
            }
        }
    }
    pub(crate) mod between {
        use std::fmt::Write;
        use crate::{scalar::ScalarExpr, writer::{FormatContext, FormatWriter}};
        pub use qraft_derive::BetweenOperator;
        pub enum BetweenOperator {
            Between,
            NotBetween,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BetweenOperator {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        BetweenOperator::Between => "Between",
                        BetweenOperator::NotBetween => "NotBetween",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BetweenOperator {
            #[inline]
            fn clone(&self) -> BetweenOperator {
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for BetweenOperator {}
        impl crate::Builder {
            pub fn where_between<C, L, H>(
                &mut self,
                lhs: C,
                low: L,
                high: H,
            ) -> &mut Self
            where
                C: crate::IntoScalarIdent,
                L: crate::IntoScalar,
                H: crate::IntoScalar,
            {
                self.where_between_expr(
                    crate::expr::Conjunction::And,
                    lhs.into_scalar_ident().0,
                    low.into_scalar().0,
                    high.into_scalar().0,
                    BetweenOperator::Between,
                )
            }
            pub fn or_where_between<C, L, H>(
                &mut self,
                lhs: C,
                low: L,
                high: H,
            ) -> &mut Self
            where
                C: crate::IntoScalarIdent,
                L: crate::IntoScalar,
                H: crate::IntoScalar,
            {
                self.where_between_expr(
                    crate::expr::Conjunction::Or,
                    lhs.into_scalar_ident().0,
                    low.into_scalar().0,
                    high.into_scalar().0,
                    BetweenOperator::Between,
                )
            }
            pub fn where_not_between<C, L, H>(
                &mut self,
                lhs: C,
                low: L,
                high: H,
            ) -> &mut Self
            where
                C: crate::IntoScalarIdent,
                L: crate::IntoScalar,
                H: crate::IntoScalar,
            {
                self.where_between_expr(
                    crate::expr::Conjunction::And,
                    lhs.into_scalar_ident().0,
                    low.into_scalar().0,
                    high.into_scalar().0,
                    BetweenOperator::NotBetween,
                )
            }
            pub fn or_where_not_between<C, L, H>(
                &mut self,
                lhs: C,
                low: L,
                high: H,
            ) -> &mut Self
            where
                C: crate::IntoScalarIdent,
                L: crate::IntoScalar,
                H: crate::IntoScalar,
            {
                self.where_between_expr(
                    crate::expr::Conjunction::Or,
                    lhs.into_scalar_ident().0,
                    low.into_scalar().0,
                    high.into_scalar().0,
                    BetweenOperator::NotBetween,
                )
            }
        }
        pub struct BetweenCondition {
            pub(crate) lhs: ScalarExpr,
            pub(crate) low: ScalarExpr,
            pub(crate) high: ScalarExpr,
            pub(crate) operator: BetweenOperator,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for BetweenCondition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "BetweenCondition",
                    "lhs",
                    &self.lhs,
                    "low",
                    &self.low,
                    "high",
                    &self.high,
                    "operator",
                    &&self.operator,
                )
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for BetweenCondition {
            #[inline]
            fn clone(&self) -> BetweenCondition {
                BetweenCondition {
                    lhs: ::core::clone::Clone::clone(&self.lhs),
                    low: ::core::clone::Clone::clone(&self.low),
                    high: ::core::clone::Clone::clone(&self.high),
                    operator: ::core::clone::Clone::clone(&self.operator),
                }
            }
        }
        impl FormatWriter for BetweenOperator {
            fn format_writer<W: Write>(
                &self,
                context: &mut FormatContext<'_, W>,
            ) -> std::fmt::Result {
                match self {
                    BetweenOperator::Between => context.writer.write_str("between"),
                    BetweenOperator::NotBetween => {
                        context.writer.write_str("not between")
                    }
                }
            }
        }
        impl FormatWriter for BetweenCondition {
            fn format_writer<W: Write>(
                &self,
                context: &mut FormatContext<'_, W>,
            ) -> std::fmt::Result {
                self.lhs.format_writer(context)?;
                context.writer.write_char(' ')?;
                self.operator.format_writer(context)?;
                context.writer.write_char(' ')?;
                self.low.format_writer(context)?;
                context.writer.write_str(" and ")?;
                self.high.format_writer(context)
            }
        }
    }
    pub(crate) mod r#in {
        use crate::scalar::ScalarExpr;
        pub struct InCondition {
            operator: InOperator,
            lhs: ScalarExpr,
            rhs: ScalarExpr,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InCondition {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "InCondition",
                    "operator",
                    &self.operator,
                    "lhs",
                    &self.lhs,
                    "rhs",
                    &&self.rhs,
                )
            }
        }
        pub enum InOperator {
            In,
            NotIn,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for InOperator {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        InOperator::In => "In",
                        InOperator::NotIn => "NotIn",
                    },
                )
            }
        }
    }
    pub use cond::Conjunction;
    pub enum ConditionKind {
        Binary(BinaryCondition),
        Group(GroupCondition),
        Raw(Raw),
        Unary(UnaryCondition),
        Between(BetweenCondition),
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for ConditionKind {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match self {
                ConditionKind::Binary(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Binary",
                        &__self_0,
                    )
                }
                ConditionKind::Group(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Group",
                        &__self_0,
                    )
                }
                ConditionKind::Raw(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Raw",
                        &__self_0,
                    )
                }
                ConditionKind::Unary(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Unary",
                        &__self_0,
                    )
                }
                ConditionKind::Between(__self_0) => {
                    ::core::fmt::Formatter::debug_tuple_field1_finish(
                        f,
                        "Between",
                        &__self_0,
                    )
                }
            }
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for ConditionKind {
        #[inline]
        fn clone(&self) -> ConditionKind {
            match self {
                ConditionKind::Binary(__self_0) => {
                    ConditionKind::Binary(::core::clone::Clone::clone(__self_0))
                }
                ConditionKind::Group(__self_0) => {
                    ConditionKind::Group(::core::clone::Clone::clone(__self_0))
                }
                ConditionKind::Raw(__self_0) => {
                    ConditionKind::Raw(::core::clone::Clone::clone(__self_0))
                }
                ConditionKind::Unary(__self_0) => {
                    ConditionKind::Unary(::core::clone::Clone::clone(__self_0))
                }
                ConditionKind::Between(__self_0) => {
                    ConditionKind::Between(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    impl FormatWriter for ConditionKind {
        fn format_writer<W: std::fmt::Write>(
            &self,
            context: &mut crate::writer::FormatContext<'_, W>,
        ) -> std::fmt::Result {
            match self {
                ConditionKind::Binary(binary) => binary.format_writer(context),
                ConditionKind::Group(group) => group.format_writer(context),
                ConditionKind::Raw(raw) => raw.format_writer(context),
                ConditionKind::Unary(unary) => unary.format_writer(context),
                ConditionKind::Between(between) => between.format_writer(context),
            }
        }
    }
}
use bind::Bind;
pub use col::TableSchema;
pub use col::ColumnSchema;
pub use col::Columns;
pub use col::IntoColumns;
pub use col::IntoTable;
pub use bind::Binds;
pub use bind::IntoBind;
pub use bind::IntoBinds;
pub use ident::TableIdent;
pub use ident::Ident;
pub use raw::Raw;
pub use raw::IntoRaw;
pub use builder::Builder;
pub use scalar::IntoScalar;
pub use scalar::IntoScalarIdent;
pub use scalar::IntoOperator;
pub fn column_static(value: &'static str) -> Ident {
    Ident::new_static(value)
}
pub fn column(value: &str) -> Ident {
    Ident::new(value)
}
pub fn value_static(value: &'static str) -> Bind {
    Bind::new_static_str(value)
}
pub fn value<V: IntoBind>(value: V) -> Bind {
    Bind::new(value)
}
pub fn raw_static(value: &'static str) -> Raw {
    Raw::new_static(value)
}
pub fn raw(value: &str) -> Raw {
    Raw::new(value)
}
pub fn sub<F>(query: F) -> Builder
where
    F: FnOnce(&mut Builder),
{
    let mut builder = Builder::default();
    query(&mut builder);
    builder
}
