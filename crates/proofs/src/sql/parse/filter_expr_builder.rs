use crate::base::database::{ColumnRef, TableRef};
use crate::sql::ast::{
    AndExpr, BoolExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr,
    TableExpr,
};

use proofs_sql::intermediate_ast::{BinaryOperator, Expression, Literal, UnaryOperator};
use proofs_sql::Identifier;

use std::collections::{HashMap, HashSet};

pub struct FilterExprBuilder {
    table: Option<TableExpr>,
    where_clause: Option<Box<dyn BoolExpr>>,
    filter_result_expr_list: Vec<FilterResultExpr>,
    column_mapping: HashMap<Identifier, ColumnRef>,
}

// Public interface
impl FilterExprBuilder {
    pub fn new(column_mapping: HashMap<Identifier, ColumnRef>) -> Self {
        Self {
            table: None,
            where_clause: None,
            filter_result_expr_list: vec![],
            column_mapping,
        }
    }

    pub fn set_table(mut self, table: TableRef) -> Self {
        self.table = Some(TableExpr { table_ref: table });
        self
    }

    pub fn set_where_clause(mut self, where_clause: Option<Box<Expression>>) -> Self {
        if let Some(where_clause) = where_clause {
            self.where_clause = Some(self.visit_expression(*where_clause));
        }
        self
    }

    pub fn add_referenced_result_columns(mut self, columns: HashSet<Identifier>) -> Self {
        // Sorting is required to make the relative order of the columns deterministic
        let mut columns = columns.into_iter().collect::<Vec<_>>();
        columns.sort();

        columns.into_iter().for_each(|column| {
            let column = *self.column_mapping.get(&column).unwrap();
            self.filter_result_expr_list
                .push(FilterResultExpr::new(column));
        });

        self
    }

    pub fn build(self) -> FilterExpr {
        FilterExpr::new(
            self.filter_result_expr_list,
            self.table.expect("table is required"),
            self.where_clause
                .unwrap_or_else(|| Box::new(ConstBoolExpr::new(true))),
        )
    }
}

// Private interface
impl FilterExprBuilder {
    fn visit_expression(
        &self,
        expr: proofs_sql::intermediate_ast::Expression,
    ) -> Box<dyn BoolExpr> {
        match expr {
            Expression::Binary { op, left, right } => {
                self.visit_binary_expression(op, *left, *right)
            }
            Expression::Unary { op, expr } => self.visit_unary_expression(op, *expr),
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    fn visit_unary_expression(&self, op: UnaryOperator, expr: Expression) -> Box<dyn BoolExpr> {
        let expr = self.visit_expression(expr);

        match op {
            UnaryOperator::Not => Box::new(NotExpr::new(expr)),
        }
    }

    fn visit_binary_expression(
        &self,
        op: BinaryOperator,
        left: Expression,
        right: Expression,
    ) -> Box<dyn BoolExpr> {
        match op {
            BinaryOperator::And => {
                let left = self.visit_expression(left);
                let right = self.visit_expression(right);
                Box::new(AndExpr::new(left, right))
            }
            BinaryOperator::Or => {
                let left = self.visit_expression(left);
                let right = self.visit_expression(right);
                Box::new(OrExpr::new(left, right))
            }
            BinaryOperator::Equal => self.visit_equal_expression(left, right),
            _ => panic!("The parser must ensure that the expression is a boolean expression"),
        }
    }

    fn visit_equal_expression(&self, left: Expression, right: Expression) -> Box<dyn BoolExpr> {
        let left = match left {
            Expression::Column(identifier) => *self.column_mapping.get(&identifier).unwrap(),
            _ => panic!("The parser must ensure that the left side is a column"),
        };

        let right = match right {
            Expression::Literal(literal) => match literal {
                Literal::Int128(value) => value.into(),
                Literal::VarChar(value) => value.into(),
            },
            _ => panic!("The parser must ensure that the left side is a literal"),
        };

        Box::new(EqualsExpr::new(left, right))
    }
}
