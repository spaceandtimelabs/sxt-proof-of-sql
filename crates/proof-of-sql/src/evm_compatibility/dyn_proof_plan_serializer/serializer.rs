use super::{
    error::{TooManyColumnsSnafu, TooManyTablesSnafu},
    ProofPlanSerializationError,
};
use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::{IndexMap, IndexSet},
        scalar::Scalar,
    },
    evm_compatibility::primitive_serialize_ext::PrimitiveSerializeExt,
};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A serializer for a `DynProofPlan`.
#[derive(Debug, Clone)]
pub struct DynProofPlanSerializer<S: Scalar> {
    pub(super) table_refs: IndexMap<TableRef, u8>,
    pub(super) column_refs: IndexMap<ColumnRef, u8>,
    data: Vec<u8>,
    _phantom: PhantomData<S>,
}

impl<S: Scalar> PrimitiveSerializeExt<S> for DynProofPlanSerializer<S> {
    fn serialize_slice(mut self, value: &[u8]) -> Self {
        self.data.extend_from_slice(value);
        self
    }
}

impl<S: Scalar> DynProofPlanSerializer<S> {
    /// Converts the serialized plan into a byte vector.
    ///
    /// # Returns
    ///
    /// * `Vec<u8>` - The serialized byte vector.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Creates a new serializer with the given table and column references.
    ///
    /// # Arguments
    ///
    /// * `table_refs` - A set of table references.
    /// * `column_refs` - A set of column references.
    ///
    /// # Returns
    ///
    /// * `Result<Self, ProofPlanSerializationError>` - The new serializer or an error if there are too many tables or columns.
    pub fn try_new(
        table_refs: IndexSet<TableRef>,
        column_refs: IndexSet<ColumnRef>,
    ) -> Result<Self, ProofPlanSerializationError> {
        if u8::try_from(table_refs.len()).is_err() {
            TooManyTablesSnafu.fail()?;
        }
        if u8::try_from(column_refs.len()).is_err() {
            TooManyColumnsSnafu.fail()?;
        }
        Ok(Self {
            table_refs: table_refs.into_iter().zip(0..).collect(),
            column_refs: column_refs.into_iter().zip(0..).collect(),
            data: Vec::new(),
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{
            database::{ColumnRef, ColumnType, TableRef},
            map::{indexmap, indexset},
            scalar::test_scalar::TestScalar,
        },
        evm_compatibility::dyn_proof_plan_serializer::{
            DynProofPlanSerializer, ProofPlanSerializationError,
        },
    };

    #[test]
    fn we_can_create_dyn_proof_plan_serializer() {
        let table_ref_1: TableRef = "namespace.table1".parse().unwrap();
        let table_ref_2: TableRef = "namespace.table2".parse().unwrap();
        let column_ref_1: ColumnRef =
            ColumnRef::new(table_ref_1, "column1".into(), ColumnType::BigInt);
        let column_ref_2: ColumnRef =
            ColumnRef::new(table_ref_2, "column2".into(), ColumnType::BigInt);

        let table_refs = indexset! { table_ref_1, table_ref_2 };
        let column_refs = indexset! { column_ref_1.clone(), column_ref_2.clone() };
        let serializer =
            DynProofPlanSerializer::<TestScalar>::try_new(table_refs, column_refs).unwrap();
        assert_eq!(
            serializer.table_refs,
            indexmap! { table_ref_1 => 0, table_ref_2 => 1 }
        );
        assert_eq!(
            serializer.column_refs,
            indexmap! { column_ref_1 => 0, column_ref_2 => 1 }
        );
    }

    #[test]
    fn we_cannot_create_dyn_proof_plan_serializer_with_too_many_tables() {
        let table_refs = (0..=u8::MAX as usize)
            .map(|i| format!("namespace.table{i}").parse().unwrap())
            .collect();

        let result = DynProofPlanSerializer::<TestScalar>::try_new(table_refs, indexset! {});
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::TooManyTables)
        ));
    }

    #[test]
    fn we_cannot_create_dyn_proof_plan_serializer_with_too_many_columns() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let column_refs = (0..=u8::MAX as usize)
            .map(|i| {
                ColumnRef::new(
                    table_ref,
                    format!("column{i}").as_str().into(),
                    ColumnType::BigInt,
                )
            })
            .collect();

        let result = DynProofPlanSerializer::<TestScalar>::try_new(indexset! {}, column_refs);
        assert!(matches!(
            result,
            Err(ProofPlanSerializationError::TooManyColumns)
        ));
    }
}
