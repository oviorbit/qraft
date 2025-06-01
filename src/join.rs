use crate::{expr::cond::Conditions, TableRef};

pub enum JoinType {
}

pub struct JoinClause {
    ty: JoinType,
    table: TableRef,
    conditions: Conditions,
}

impl JoinClause {
}
