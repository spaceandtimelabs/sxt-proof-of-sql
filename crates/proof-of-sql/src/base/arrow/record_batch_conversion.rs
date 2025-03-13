use super::{
    arrow_array_to_column_conversion::ArrayRefExt,
    record_batch_errors::{AppendRecordBatchTableCommitmentError, RecordBatchToColumnsError},
};
use crate::base::{
    commitment::{
        AppendColumnCommitmentsError, AppendTableCommitmentError, Commitment, TableCommitment,
        TableCommitmentFromColumnsError,
    },
    database::Column,
    scalar::Scalar,
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use sqlparser::ast::Ident;

/// This function will return an error if:
/// - The field name cannot be parsed into an [`Identifier`].
/// - The conversion of an Arrow array to a [`Column`] fails.
pub fn batch_to_columns<'a, S: Scalar + 'a>(
    batch: &'a RecordBatch,
    alloc: &'a Bump,
) -> Result<Vec<(Ident, Column<'a, S>)>, RecordBatchToColumnsError> {
    batch
        .schema()
        .fields()
        .into_iter()
        .zip(batch.columns())
        .map(|(field, array)| {
            let identifier: Ident = field.name().as_str().into();
            let column: Column<S> = array.to_column(alloc, &(0..array.len()), None)?;
            Ok((identifier, column))
        })
        .collect()
}

impl<C: Commitment> TableCommitment<C> {
    /// Append an arrow [`RecordBatch`] to the existing [`TableCommitment`].
    ///
    /// The row offset is assumed to be the end of the [`TableCommitment`]'s current range.
    ///
    /// Will error on a variety of mismatches, or if the provided columns have mixed length.
    #[expect(clippy::missing_panics_doc)]
    pub fn try_append_record_batch(
        &mut self,
        batch: &RecordBatch,
        setup: &C::PublicSetup<'_>,
    ) -> Result<(), AppendRecordBatchTableCommitmentError> {
        match self.try_append_rows(
            batch_to_columns::<C::Scalar>(batch, &Bump::new())?
                .iter()
                .map(|(a, b)| (a, b)),
            setup,
        ) {
            Ok(()) => Ok(()),
            Err(AppendTableCommitmentError::MixedLengthColumns { .. }) => {
                panic!("RecordBatches cannot have columns of mixed length")
            }
            Err(AppendTableCommitmentError::AppendColumnCommitments {
                source: AppendColumnCommitmentsError::DuplicateIdents { .. },
            }) => {
                panic!("RecordBatches cannot have duplicate identifiers")
            }
            Err(AppendTableCommitmentError::AppendColumnCommitments {
                source: AppendColumnCommitmentsError::Mismatch { source: e },
            }) => Err(e)?,
        }
    }
    /// Returns a [`TableCommitment`] to the provided arrow [`RecordBatch`].
    pub fn try_from_record_batch(
        batch: &RecordBatch,
        setup: &C::PublicSetup<'_>,
    ) -> Result<TableCommitment<C>, RecordBatchToColumnsError> {
        Self::try_from_record_batch_with_offset(batch, 0, setup)
    }

    /// Returns a [`TableCommitment`] to the provided arrow [`RecordBatch`] with the given row offset.
    #[expect(clippy::missing_panics_doc)]
    pub fn try_from_record_batch_with_offset(
        batch: &RecordBatch,
        offset: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Result<TableCommitment<C>, RecordBatchToColumnsError> {
        match Self::try_from_columns_with_offset(
            batch_to_columns::<C::Scalar>(batch, &Bump::new())?
                .iter()
                .map(|(a, b)| (a, b)),
            offset,
            setup,
        ) {
            Ok(commitment) => Ok(commitment),
            Err(TableCommitmentFromColumnsError::MixedLengthColumns { .. }) => {
                panic!("RecordBatches cannot have columns of mixed length")
            }
            Err(TableCommitmentFromColumnsError::DuplicateIdents { .. }) => {
                panic!("RecordBatches cannot have duplicate identifiers")
            }
        }
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::base::{
        commitment::naive_commitment::NaiveCommitment, scalar::test_scalar::TestScalar,
    };
    use arrow::{
        array::{Int64Array, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use std::sync::Arc;

    #[test]
    fn we_can_create_and_append_table_commitments_with_record_batches() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["1", "2", "3"])),
            ],
        )
        .unwrap();

        let b_scals = ["1".into(), "2".into(), "3".into()];

        let columns = [
            (&"a".into(), &Column::<TestScalar>::BigInt(&[1, 2, 3])),
            (
                &"b".into(),
                &Column::<TestScalar>::VarChar((&["1", "2", "3"], &b_scals)),
            ),
        ];

        let mut expected_commitment =
            TableCommitment::<NaiveCommitment>::try_from_columns_with_offset(columns, 0, &())
                .unwrap();

        let mut commitment =
            TableCommitment::<NaiveCommitment>::try_from_record_batch(&batch, &()).unwrap();

        assert_eq!(commitment, expected_commitment);

        let schema2 = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Utf8, false),
        ]));

        let batch2 = RecordBatch::try_new(
            schema2,
            vec![
                Arc::new(Int64Array::from(vec![4, 5, 6])),
                Arc::new(StringArray::from(vec!["4", "5", "6"])),
            ],
        )
        .unwrap();

        let b_scals2 = ["4".into(), "5".into(), "6".into()];

        let columns2 = [
            (&"a".into(), &Column::<TestScalar>::BigInt(&[4, 5, 6])),
            (
                &"b".into(),
                &Column::<TestScalar>::VarChar((&["4", "5", "6"], &b_scals2)),
            ),
        ];

        expected_commitment.try_append_rows(columns2, &()).unwrap();
        commitment.try_append_record_batch(&batch2, &()).unwrap();

        assert_eq!(commitment, expected_commitment);
    }
}
