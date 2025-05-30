use crate::scalar::ScalarExpr;

#[derive(Debug)]
pub struct InCondition {
    operator: InOperator,
    lhs: ScalarExpr,
    rhs: ScalarExpr,
}

#[derive(Debug)]
pub enum InOperator {
    In,
    NotIn,
}
