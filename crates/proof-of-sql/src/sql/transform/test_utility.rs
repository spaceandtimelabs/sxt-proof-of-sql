use super::*;
use proof_of_sql_parser::intermediate_ast::*;

pub fn lit_i64(literal: i64) -> Box<Expression> {
    Box::new(Expression::Literal(Literal::BigInt(literal)))
}

pub fn lit<L: Into<Literal>>(literal: L) -> Box<Expression> {
    Box::new(Expression::Literal(literal.into()))
}
pub trait ToLit {
    fn to_lit(self) -> Box<Expression>;
}
impl ToLit for i64 {
    fn to_lit(self) -> Box<Expression> {
        lit_i64(self)
    }
}
pub fn col(name: &str) -> Box<Expression> {
    Box::new(Expression::Column(name.parse().unwrap()))
}

pub(crate) fn select(result_schema: &[impl ToPolarsExpr]) -> Box<dyn RecordBatchExpr> {
    #[allow(deprecated)]
    Box::new(SelectExpr::new(result_schema))
}

pub fn schema(columns: &[(&str, &str)]) -> Vec<AliasedResultExpr> {
    columns
        .iter()
        .map(|(name, alias)| col(name).alias(alias))
        .collect()
}

pub fn result(columns: &[(&str, &str)]) -> ResultExpr {
    let mut composition = CompositionExpr::default();
    composition.add(Box::new(SelectExpr::new_from_aliased_result_exprs(
        &schema(columns),
    )));
    ResultExpr::new(Box::new(composition))
}

pub fn slice(limit: u64, offset: i64) -> Box<dyn RecordBatchExpr> {
    Box::new(SliceExpr::new(limit, offset))
}

pub fn composite_result(transformations: Vec<Box<dyn RecordBatchExpr>>) -> ResultExpr {
    let mut composition = CompositionExpr::default();

    for transformation in transformations {
        composition.add(transformation);
    }

    ResultExpr::new(Box::new(composition))
}

pub fn orders(cols: &[&str], directions: &[OrderByDirection]) -> Box<dyn RecordBatchExpr> {
    let by_exprs = cols
        .iter()
        .zip(directions.iter())
        .map(|(col, direction)| OrderBy {
            expr: col.parse().unwrap(),
            direction: *direction,
        })
        .collect();

    Box::new(OrderByExprs::new(by_exprs))
}

pub fn groupby<
    T: IntoIterator<Item = Box<Expression>>,
    A: IntoIterator<Item = AliasedResultExpr>,
>(
    by_exprs: T,
    agg_exprs: A,
) -> Box<dyn RecordBatchExpr> {
    Box::new(GroupByExpr::new(
        &Vec::from_iter(by_exprs.into_iter().map(|expr| match *expr {
            Expression::Column(c) => c,
            _ => panic!("Expected column expression"),
        })),
        &Vec::from_iter(agg_exprs),
    ))
}
