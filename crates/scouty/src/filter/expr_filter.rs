//! Expression-backed filter implementation.

use crate::filter::eval;
use crate::filter::expr::Expr;
use crate::record::LogRecord;
use crate::traits::LogFilter;

/// A filter backed by a parsed expression.
#[derive(Debug)]
pub struct ExprFilter {
    expr: Expr,
    description: String,
}

impl ExprFilter {
    pub fn new(expr: Expr, description: impl Into<String>) -> Self {
        Self {
            expr,
            description: description.into(),
        }
    }
}

impl LogFilter for ExprFilter {
    fn matches(&self, record: &LogRecord) -> bool {
        eval::eval(&self.expr, record)
    }

    fn description(&self) -> &str {
        &self.description
    }
}
