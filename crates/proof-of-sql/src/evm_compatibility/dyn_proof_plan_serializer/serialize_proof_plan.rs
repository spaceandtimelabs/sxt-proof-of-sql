use super::{
    constants::FILTER_EXEC_NUM,
    error::{NotSupportedSnafu, TableNotFoundSnafu, TooManyResultsSnafu},
    DynProofPlanSerializer, ProofPlanSerializationError,
};
use crate::{
    base::scalar::Scalar,
    evm_compatibility::primitive_serialize_ext::PrimitiveSerializeExt,
    sql::{
        proof_exprs::{AliasedDynProofExpr, TableExpr},
        proof_plans::{DynProofPlan, FilterExec},
    },
};
use snafu::OptionExt;

impl<S: Scalar> DynProofPlanSerializer<S> {
    pub fn serialize_dyn_proof_plan(
        self,
        plan: &DynProofPlan,
    ) -> Result<Self, ProofPlanSerializationError> {
        match plan {
            DynProofPlan::Filter(filter_exec) => self
                .serialize_u8(FILTER_EXEC_NUM)
                .serialize_filter_exec(filter_exec),
            _ => NotSupportedSnafu.fail(),
        }
    }

    fn serialize_filter_exec(
        self,
        filter_exec: &FilterExec,
    ) -> Result<Self, ProofPlanSerializationError> {
        let result_count = u8::try_from(filter_exec.aliased_results.len())
            .ok()
            .context(TooManyResultsSnafu)?;

        filter_exec
            .aliased_results
            .iter()
            .try_fold(
                self.serialize_table_expr(&filter_exec.table)?
                    .serialize_u8(result_count),
                Self::serialize_aliased_dyn_proof_expr,
            )?
            .serialize_dyn_proof_expr(&filter_exec.where_clause)
    }

    fn serialize_table_expr(
        self,
        table_expr: &TableExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        let table_number = self
            .table_refs
            .get(&table_expr.table_ref)
            .copied()
            .context(TableNotFoundSnafu)?;
        Ok(self.serialize_u8(table_number))
    }

    fn serialize_aliased_dyn_proof_expr(
        self,
        aliased_expr: &AliasedDynProofExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        self.serialize_dyn_proof_expr(&aliased_expr.expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{database::LiteralValue, map::indexset, scalar::test_scalar::TestScalar},
        sql::proof_exprs::{DynProofExpr, LiteralExpr},
    };
    use core::iter;
    use itertools::Itertools;

    #[test]
    fn we_can_serialize_an_aliased_dyn_proof_expr() {
        let serializer =
            DynProofPlanSerializer::<TestScalar>::try_new(indexset! {}, indexset! {}).unwrap();

        let expr = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let expr_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&expr)
            .unwrap()
            .into_bytes();

        let aliased_expr = AliasedDynProofExpr {
            expr: expr.clone(),
            alias: "alias".into(),
        };
        let bytes = serializer
            .clone()
            .serialize_aliased_dyn_proof_expr(&aliased_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes, expr_bytes);
    }

    #[test]
    fn we_can_serialize_a_table_expr() {
        let table_0_ref = "namespace.table_0".parse().unwrap();
        let table_1_ref = "namespace.table_1".parse().unwrap();
        let table_2_ref = "namespace.table_2".parse().unwrap();
        let serializer = DynProofPlanSerializer::<TestScalar>::try_new(
            indexset! { table_0_ref, table_1_ref },
            indexset! {},
        )
        .unwrap();

        // Serialization of table 0 should result in a single byte with value 0.
        let table_0_expr = TableExpr {
            table_ref: table_0_ref,
        };
        let bytes_0 = serializer
            .clone()
            .serialize_table_expr(&table_0_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes_0, vec![0]);

        // Serialization of table 1 should result in a single byte with value 1.
        let table_1_expr = TableExpr {
            table_ref: table_1_ref,
        };
        let bytes_1 = serializer
            .clone()
            .serialize_table_expr(&table_1_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes_1, vec![1]);

        // Serialization of table 2 should result in an error because it is not in the serializer's set.
        let table_2_expr = TableExpr {
            table_ref: table_2_ref,
        };
        let result = serializer.clone().serialize_table_expr(&table_2_expr);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::TableNotFound)
        ));
    }

    #[test]
    fn we_can_serialize_a_filter_exec() {
        let table_ref = "namespace.table".parse().unwrap();
        let serializer =
            DynProofPlanSerializer::<TestScalar>::try_new(indexset! { table_ref }, indexset! {})
                .unwrap();

        let expr_a = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let expr_b = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let expr_c = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let aliased_expr_0 = AliasedDynProofExpr {
            expr: expr_a.clone(),
            alias: "alias_0".into(),
        };
        let aliased_expr_1 = AliasedDynProofExpr {
            expr: expr_b.clone(),
            alias: "alias_1".into(),
        };
        let table_expr = TableExpr { table_ref };

        let expr_c_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&expr_c)
            .unwrap()
            .into_bytes();
        let aliased_expr_0_bytes = serializer
            .clone()
            .serialize_aliased_dyn_proof_expr(&aliased_expr_0)
            .unwrap()
            .into_bytes();
        let aliased_expr_1_bytes = serializer
            .clone()
            .serialize_aliased_dyn_proof_expr(&aliased_expr_1)
            .unwrap()
            .into_bytes();
        let table_expr_bytes = serializer
            .clone()
            .serialize_table_expr(&table_expr)
            .unwrap()
            .into_bytes();

        // Serialization of a filter exec should result in the table number, the number of results,
        // the serialized aliased expressions, and the serialized where clause.
        let filter_exec = FilterExec::new(vec![aliased_expr_0, aliased_expr_1], table_expr, expr_c);
        let expected_bytes = iter::empty::<u8>()
            .chain(table_expr_bytes)
            .chain([2])
            .chain(aliased_expr_0_bytes)
            .chain(aliased_expr_1_bytes)
            .chain(expr_c_bytes)
            .collect_vec();
        let bytes = serializer
            .clone()
            .serialize_filter_exec(&filter_exec)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes, expected_bytes);

        // Serialization of a filter DynProofPlan should result in the
        // filter exec number and the serialized filter exec.
        let wrapped_filter_exec = DynProofPlan::Filter(filter_exec);
        let expected_wrapped_bytes = iter::empty::<u8>()
            .chain([FILTER_EXEC_NUM])
            .chain(expected_bytes)
            .collect_vec();
        let wrapped_bytes = serializer
            .clone()
            .serialize_dyn_proof_plan(&wrapped_filter_exec)
            .unwrap()
            .into_bytes();
        assert_eq!(wrapped_bytes, expected_wrapped_bytes);
    }
}
