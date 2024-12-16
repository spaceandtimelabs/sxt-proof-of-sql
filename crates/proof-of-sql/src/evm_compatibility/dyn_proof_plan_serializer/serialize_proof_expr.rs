use super::{
    constants::{BIGINT_TYPE_NUM, COLUMN_EXPR_NUM, EQUALS_EXPR_NUM, LITERAL_EXPR_NUM},
    error::{ColumnNotFoundSnafu, NotSupportedSnafu},
    DynProofPlanSerializer, ProofPlanSerializationError,
};
use crate::{
    base::{database::LiteralValue, scalar::Scalar},
    evm_compatibility::primitive_serialize_ext::PrimitiveSerializeExt,
    sql::proof_exprs::{ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
};
use snafu::OptionExt;

impl<S: Scalar> DynProofPlanSerializer<S> {
    pub(super) fn serialize_dyn_proof_expr(
        self,
        expr: &DynProofExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        match expr {
            DynProofExpr::Column(column_expr) => self
                .serialize_u8(COLUMN_EXPR_NUM)
                .serialize_column_expr(column_expr),
            DynProofExpr::Literal(literal_expr) => self
                .serialize_u8(LITERAL_EXPR_NUM)
                .serialize_literal_expr(literal_expr),
            DynProofExpr::Equals(equals_expr) => self
                .serialize_u8(EQUALS_EXPR_NUM)
                .serialize_equals_expr(equals_expr),
            _ => NotSupportedSnafu.fail(),
        }
    }

    fn serialize_column_expr(
        self,
        column_expr: &ColumnExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        let column_number = self
            .column_refs
            .get(&column_expr.column_ref)
            .copied()
            .context(ColumnNotFoundSnafu)?;
        Ok(self.serialize_u8(column_number))
    }

    fn serialize_literal_expr(
        self,
        literal_expr: &LiteralExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        match literal_expr.value {
            LiteralValue::BigInt(value) => Ok(self
                .serialize_u8(BIGINT_TYPE_NUM)
                .serialize_scalar(value.into())),
            _ => NotSupportedSnafu.fail(),
        }
    }

    fn serialize_equals_expr(
        self,
        equals_expr: &EqualsExpr,
    ) -> Result<Self, ProofPlanSerializationError> {
        self.serialize_dyn_proof_expr(equals_expr.lhs.as_ref())?
            .serialize_dyn_proof_expr(equals_expr.rhs.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, TableRef},
            map::indexset,
            scalar::test_scalar::TestScalar,
        },
        sql::proof_exprs::AndExpr,
    };
    use core::iter;
    use itertools::Itertools;

    #[test]
    fn we_can_serialize_a_column_expr() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let column_0_ref: ColumnRef =
            ColumnRef::new(table_ref, "column_0".into(), ColumnType::BigInt);
        let column_1_ref: ColumnRef =
            ColumnRef::new(table_ref, "column_1".into(), ColumnType::BigInt);
        let column_2_ref: ColumnRef =
            ColumnRef::new(table_ref, "column_2".into(), ColumnType::BigInt);
        let serializer = DynProofPlanSerializer::<TestScalar>::try_new(
            indexset! {},
            indexset! { column_0_ref.clone(), column_1_ref.clone() },
        )
        .unwrap();

        // Serialization of column 0 should result in a single byte with value 0.
        let column_0_expr = ColumnExpr::new(column_0_ref);
        let bytes_0 = serializer
            .clone()
            .serialize_column_expr(&column_0_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes_0, vec![0]);

        // Serialization of column 1 should result in a single byte with value 1.
        let column_1_expr = ColumnExpr::new(column_1_ref);
        let bytes_1 = serializer
            .clone()
            .serialize_column_expr(&column_1_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(bytes_1, vec![1]);

        // Wrapping the column expression in a `DynProofExpr` should result in the same serialization,
        // but with the column expression number prepended.
        let wrapped_column_1_expr = DynProofExpr::Column(column_1_expr);
        let wrapped_bytes_1 = serializer
            .clone()
            .serialize_dyn_proof_expr(&wrapped_column_1_expr)
            .unwrap()
            .into_bytes();
        assert_eq!(wrapped_bytes_1, vec![COLUMN_EXPR_NUM, 1]);

        // Serialization of column 2 should result in an error because there are only two columns.
        let column_2_expr = ColumnExpr::new(column_2_ref);
        let result = serializer.clone().serialize_column_expr(&column_2_expr);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::ColumnNotFound)
        ));
    }

    #[test]
    fn we_can_serialize_a_literal_expr() {
        let serializer =
            DynProofPlanSerializer::<TestScalar>::try_new(indexset! {}, indexset! {}).unwrap();

        // Serialization of a big int literal should result in a byte with the big int type number,
        // followed by the big int value in big-endian form, padded with leading zeros to 32 bytes.
        let literal_bigint_expr = LiteralExpr::new(LiteralValue::BigInt(4200));
        let bigint_bytes = serializer
            .clone()
            .serialize_literal_expr(&literal_bigint_expr)
            .unwrap()
            .into_bytes();
        let expected_bigint_bytes = iter::empty::<u8>()
            .chain([BIGINT_TYPE_NUM])
            .chain([0; 30])
            .chain([16, 104])
            .collect_vec();
        assert_eq!(bigint_bytes, expected_bigint_bytes);

        // Wrapping the literal expression in a `DynProofExpr` should result in the same serialization,
        // but with the literal expression number prepended.
        let wrapped_literal_expr = DynProofExpr::Literal(literal_bigint_expr);
        let wrapped_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&wrapped_literal_expr)
            .unwrap()
            .into_bytes();
        let expected_wrapped_bytes = iter::empty::<u8>()
            .chain([LITERAL_EXPR_NUM])
            .chain(expected_bigint_bytes)
            .collect_vec();
        assert_eq!(wrapped_bytes, expected_wrapped_bytes);

        // Serialization of a small int literal should result in an error
        // because only big int literals are supported so far
        let literal_smallint_expr = LiteralExpr::new(LiteralValue::SmallInt(4200));
        let result = serializer
            .clone()
            .serialize_literal_expr(&literal_smallint_expr);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::NotSupported)
        ));
    }

    #[test]
    fn we_can_serialize_an_equals_expr() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let column_0_ref: ColumnRef =
            ColumnRef::new(table_ref, "column_0".into(), ColumnType::BigInt);
        let serializer = DynProofPlanSerializer::<TestScalar>::try_new(
            indexset! {},
            indexset! { column_0_ref.clone() },
        )
        .unwrap();

        let lhs = DynProofExpr::Column(ColumnExpr::new(column_0_ref));
        let rhs = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let lhs_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&lhs)
            .unwrap()
            .into_bytes();
        let rhs_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&rhs)
            .unwrap()
            .into_bytes();

        // Serialization of an equals expression should result in the serialization of the left-hand side,
        // followed by the serialization of the right-hand side.
        let equals_expr = EqualsExpr::new(Box::new(lhs.clone()), Box::new(rhs.clone()));
        let bytes = serializer
            .clone()
            .serialize_equals_expr(&equals_expr)
            .unwrap()
            .into_bytes();
        let expected_bytes = iter::empty::<u8>()
            .chain(lhs_bytes.clone())
            .chain(rhs_bytes.clone())
            .collect_vec();
        assert_eq!(bytes, expected_bytes);

        // Wrapping the equals expression in a `DynProofExpr` should result in the same serialization,
        // but with the equals expression number prepended.
        let wrapped_equals_expr = DynProofExpr::Equals(equals_expr);
        let wrapped_bytes = serializer
            .clone()
            .serialize_dyn_proof_expr(&wrapped_equals_expr)
            .unwrap()
            .into_bytes();
        let expected_wrapped_bytes = iter::empty::<u8>()
            .chain([EQUALS_EXPR_NUM])
            .chain(expected_bytes)
            .collect_vec();
        assert_eq!(wrapped_bytes, expected_wrapped_bytes);
    }

    #[test]
    fn we_cannot_serialize_an_unsupported_expr() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let column_0_ref: ColumnRef =
            ColumnRef::new(table_ref, "column_0".into(), ColumnType::BigInt);
        let serializer = DynProofPlanSerializer::<TestScalar>::try_new(
            indexset! {},
            indexset! { column_0_ref.clone() },
        )
        .unwrap();

        let lhs = DynProofExpr::Column(ColumnExpr::new(column_0_ref));
        let rhs = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::BigInt(4200)));
        let expr = DynProofExpr::And(AndExpr::new(Box::new(lhs.clone()), Box::new(rhs.clone())));
        let result = serializer.clone().serialize_dyn_proof_expr(&expr);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::NotSupported)
        ));
    }
}
