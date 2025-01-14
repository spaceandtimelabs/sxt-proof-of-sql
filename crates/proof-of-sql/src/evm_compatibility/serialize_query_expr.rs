use super::dyn_proof_plan_serializer::{DynProofPlanSerializer, ProofPlanSerializationError};
use crate::{
    base::scalar::Scalar,
    sql::{parse::QueryExpr, proof::ProofPlan},
};
use alloc::vec::Vec;
/// Serializes a `QueryExpr` into a vector of bytes.
///
/// This function takes a `QueryExpr` and attempts to serialize it into a vector of bytes.
/// The serialization is done in a manner that is compatible with efficient proof verification
/// on the EVM.
///
/// # Arguments
///
/// * `query_expr` - A reference to the `QueryExpr` to be serialized.
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - A vector of bytes representing the serialized query expression.
/// * `Err(ProofPlanSerializationError)` - An error indicating why the serialization failed.
///
/// # Errors
///
/// This function returns a `ProofPlanSerializationError::NotSupported` error if the query
/// expression contains postprocessing steps or if the proof plan cannot be serialized.
pub fn serialize_query_expr<S: Scalar>(
    query_expr: &QueryExpr,
) -> Result<Vec<u8>, ProofPlanSerializationError> {
    let plan = query_expr
        .postprocessing()
        .is_empty()
        .then(|| query_expr.proof_expr())
        .ok_or(ProofPlanSerializationError::NotSupported)?;
    let bytes = DynProofPlanSerializer::<S>::try_new(
        plan.get_table_references(),
        plan.get_column_references(),
    )?
    .serialize_dyn_proof_plan(plan)?
    .into_bytes();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType},
            map::indexset,
            scalar::test_scalar::TestScalar,
        },
        evm_compatibility::{
            dyn_proof_plan_serializer::{
                constants::*, DynProofPlanSerializer, ProofPlanSerializationError,
            },
            serialize_query_expr,
        },
        sql::{
            parse::QueryExpr,
            postprocessing::{OwnedTablePostprocessing, SlicePostprocessing},
            proof_exprs::{
                AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr, TableExpr,
            },
            proof_plans::{DynProofPlan, EmptyExec, FilterExec},
        },
    };
    use core::iter;
    use itertools::Itertools;
    use sqlparser::ast::{Expr as SqlExpr, Value};

    #[test]
    fn we_can_generate_serialized_proof_plan_for_query_expr() {
        let table_ref = "namespace.table".parse().unwrap();
        let identifier_alias = "alias".into();

        let plan = DynProofPlan::Filter(FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Literal(LiteralExpr::new(SqlExpr::Value(Value::Number(
                    "1001".to_string(),
                    false,
                )))),
                alias: identifier_alias,
            }],
            TableExpr { table_ref },
            DynProofExpr::Literal(LiteralExpr::new(SqlExpr::Value(Value::Number(
                "1001".to_string(),
                false,
            )))),
        ));

        // Serializing a query expression without postprocessing steps should succeed and
        // return the serialization of the proof plan.
        let query_expr = QueryExpr::new(plan.clone(), vec![]);
        let expected_bytes =
            DynProofPlanSerializer::<TestScalar>::try_new(indexset! { table_ref }, indexset! {})
                .unwrap()
                .serialize_dyn_proof_plan(&plan)
                .unwrap()
                .into_bytes();

        let bytes = serialize_query_expr::<TestScalar>(&query_expr).unwrap();
        assert_eq!(bytes, expected_bytes);

        // Serializing a query expression with postprocessing steps should fail.
        let post_processing_query_expr = QueryExpr::new(
            plan,
            vec![OwnedTablePostprocessing::Slice(SlicePostprocessing::new(
                None, None,
            ))],
        );
        let result = serialize_query_expr::<TestScalar>(&post_processing_query_expr);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::NotSupported)
        ));
    }

    #[test]
    fn we_cannot_generate_serialized_proof_plan_for_unsupported_plan() {
        let plan = DynProofPlan::Empty(EmptyExec::new());
        let result = serialize_query_expr::<TestScalar>(&QueryExpr::new(plan, vec![]));
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::NotSupported)
        ));
    }

    #[test]
    fn we_can_generate_serialized_proof_plan_for_simple_filter() {
        let table_ref = "namespace.table".parse().unwrap();
        let identifier_a = "a".into();
        let identifier_b = "b".into();
        let identifier_alias = "alias".into();

        let column_ref_a = ColumnRef::new(table_ref, identifier_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref, identifier_b, ColumnType::BigInt);

        let plan = DynProofPlan::Filter(FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b)),
                alias: identifier_alias,
            }],
            TableExpr { table_ref },
            DynProofExpr::Equals(EqualsExpr::new(
                Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a))),
                Box::new(DynProofExpr::Literal(LiteralExpr::new(SqlExpr::Value(
                    Value::Number("5".to_string(), false),
                )))),
            )),
        ));

        let query_expr = QueryExpr::new(plan, vec![]);
        let bytes = serialize_query_expr::<TestScalar>(&query_expr).unwrap();
        let expected_bytes = iter::empty::<u8>()
            .chain([FILTER_EXEC_NUM, 0, 1]) // filter expr, table number, result count
            .chain([COLUMN_EXPR_NUM, 0]) // column expr, column b (#0)
            .chain([EQUALS_EXPR_NUM]) // equals expr
            .chain([COLUMN_EXPR_NUM, 1]) // column expr, column a (#1)
            .chain([LITERAL_EXPR_NUM, BIGINT_TYPE_NUM]) // literal expr, literal type
            .chain([0; 31]) // leading 0s of literal value
            .chain([5]) // literal value
            .collect_vec();
        assert_eq!(bytes, expected_bytes);
    }
}
