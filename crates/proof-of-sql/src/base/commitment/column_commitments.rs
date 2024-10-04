use super::{
    committable_column::CommittableColumn, ColumnCommitmentMetadata, ColumnCommitmentMetadataMap,
    ColumnCommitmentMetadataMapExt, ColumnCommitmentsMismatch, Commitment, VecCommitmentExt,
};
use crate::base::{
    database::{ColumnField, ColumnRef, CommitmentAccessor, TableRef},
    map::IndexSet,
};
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{iter, slice};
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Cannot create commitments with duplicate identifier.
#[derive(Debug, Snafu)]
#[snafu(display("cannot create commitments with duplicate identifier: {id}"))]
pub struct DuplicateIdentifiers {
    id: String,
}

/// Errors that can occur when attempting to append rows to ColumnCommitments.
#[derive(Debug, Snafu)]
pub enum AppendColumnCommitmentsError {
    /// Metadata between new and old columns are mismatched.
    #[snafu(transparent)]
    Mismatch {
        /// The underlying source error
        source: ColumnCommitmentsMismatch,
    },
    /// New columns have duplicate identifiers.
    #[snafu(transparent)]
    DuplicateIdentifiers {
        /// The underlying source error
        source: DuplicateIdentifiers,
    },
}

/// Commitments for a collection of columns with some metadata.
///
/// These columns do not need to belong to the same table, and can have differing lengths.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnCommitments<C> {
    commitments: Vec<C>,
    column_metadata: ColumnCommitmentMetadataMap,
}

impl<C: Commitment> ColumnCommitments<C> {
    /// Create a new [`ColumnCommitments`] for a table from a commitment accessor.
    pub fn from_accessor_with_max_bounds(
        table: TableRef,
        columns: &[ColumnField],
        accessor: &impl CommitmentAccessor<C>,
    ) -> Self {
        let column_metadata =
            ColumnCommitmentMetadataMap::from_column_fields_with_max_bounds(columns);
        let commitments = columns
            .iter()
            .map(|c| accessor.get_commitment(ColumnRef::new(table, c.name(), c.data_type())))
            .collect();
        ColumnCommitments {
            commitments,
            column_metadata,
        }
    }

    #[cfg(test)]
    pub(super) fn column_metadata_mut(&mut self) -> &mut ColumnCommitmentMetadataMap {
        &mut self.column_metadata
    }

    /// Returns a reference to the stored commitments.
    #[must_use]
    pub fn commitments(&self) -> &Vec<C> {
        &self.commitments
    }

    /// Returns a reference to the stored column metadata.
    #[must_use]
    pub fn column_metadata(&self) -> &ColumnCommitmentMetadataMap {
        &self.column_metadata
    }

    /// Returns the number of columns.
    #[must_use]
    pub fn len(&self) -> usize {
        self.column_metadata.len()
    }

    /// Returns true if there are no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.column_metadata.is_empty()
    }

    /// Returns the commitment with the given identifier.
    #[must_use]
    pub fn get_commitment(&self, identifier: &Identifier) -> Option<C> {
        self.column_metadata
            .get_index_of(identifier)
            .map(|index| self.commitments[index].clone())
    }

    /// Returns the metadata for the commitment with the given identifier.
    #[must_use]
    pub fn get_metadata(&self, identifier: &Identifier) -> Option<&ColumnCommitmentMetadata> {
        self.column_metadata.get(identifier)
    }

    /// Iterate over the metadata and commitments by reference.
    pub fn iter(&self) -> Iter<C> {
        self.into_iter()
    }

    /// Returns [`ColumnCommitments`] to the provided columns using the given generator offset
    pub fn try_from_columns_with_offset<'a, COL>(
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        offset: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Result<ColumnCommitments<C>, DuplicateIdentifiers>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        // Check for duplicate identifiers
        let mut unique_identifiers = IndexSet::default();
        let unique_columns = columns
            .into_iter()
            .map(|(identifier, column)| {
                if unique_identifiers.insert(identifier) {
                    Ok((identifier, column))
                } else {
                    Err(DuplicateIdentifiers {
                        id: identifier.to_string(),
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (identifiers, committable_columns): (Vec<&Identifier>, Vec<CommittableColumn>) =
            unique_columns
                .into_iter()
                .map(|(identifier, column)| {
                    let committable_column: CommittableColumn = column.into();
                    (identifier, committable_column)
                })
                .unzip();

        let column_metadata = ColumnCommitmentMetadataMap::from_columns(
            identifiers.into_iter().zip(committable_columns.iter()),
        );

        let commitments = Vec::<C>::from_columns_with_offset(committable_columns, offset, setup);

        Ok(ColumnCommitments {
            commitments,
            column_metadata,
        })
    }

    /// Append rows of data from the provided columns to the existing commitments.
    ///
    /// The given generator offset will be used for committing to the new rows.
    /// You most likely want this to be equal to the 0-indexed row number of the first new row.
    ///
    /// Will error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn try_append_rows_with_offset<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        offset: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Result<(), AppendColumnCommitmentsError>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        // Check for duplicate identifiers.
        let mut unique_identifiers = IndexSet::default();
        let unique_columns = columns
            .into_iter()
            .map(|(identifier, column)| {
                if unique_identifiers.insert(identifier) {
                    Ok((identifier, column))
                } else {
                    Err(DuplicateIdentifiers {
                        id: identifier.to_string(),
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        let (identifiers, committable_columns): (Vec<&Identifier>, Vec<CommittableColumn>) =
            unique_columns
                .into_iter()
                .map(|(identifier, column)| {
                    let committable_column: CommittableColumn = column.into();
                    (identifier, committable_column)
                })
                .unzip();

        let column_metadata = ColumnCommitmentMetadataMap::from_columns(
            identifiers.into_iter().zip(committable_columns.iter()),
        );

        self.column_metadata = self.column_metadata.clone().try_union(column_metadata)?;

        self.commitments
            .try_append_rows_with_offset(committable_columns, offset, setup)
            .expect("we've already checked that self and other have equal column counts");

        Ok(())
    }

    /// Add new columns to this [`ColumnCommitments`] using the given generator offset.
    pub fn try_extend_columns_with_offset<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = (&'a Identifier, COL)>,
        offset: usize,
        setup: &C::PublicSetup<'_>,
    ) -> Result<(), DuplicateIdentifiers>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        // Check for duplicates *between* the existing and new columns.
        //
        // The existing columns should not have any duplicates within themselves due to
        // ColumnCommitments construction preventing it.
        //
        // If the new columns contain duplicates between each other, we'll catch this in the next
        // step.
        let unique_columns = columns
            .into_iter()
            .map(|(identifier, column)| {
                if self.column_metadata.contains_key(identifier) {
                    Err(DuplicateIdentifiers {
                        id: identifier.to_string(),
                    })
                } else {
                    Ok((identifier, column))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        // this constructor will check for duplicates among the new columns
        let new_column_commitments =
            ColumnCommitments::<C>::try_from_columns_with_offset(unique_columns, offset, setup)?;

        self.commitments.extend(new_column_commitments.commitments);
        self.column_metadata
            .extend(new_column_commitments.column_metadata);

        Ok(())
    }

    /// Add two [`ColumnCommitments`] together.
    ///
    /// Will error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn try_add(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized,
    {
        let column_metadata = self.column_metadata.try_union(other.column_metadata)?;
        let commitments = self
            .commitments
            .try_add(other.commitments)
            .expect("we've already checked that self and other have equal column counts");

        Ok(ColumnCommitments {
            column_metadata,
            commitments,
        })
    }

    /// Subtract two [`ColumnCommitments`].
    ///
    /// Will error on a variety of mismatches.
    /// See [`ColumnCommitmentsMismatch`] for an enumeration of these errors.
    pub fn try_sub(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized,
    {
        let column_metadata = self.column_metadata.try_difference(other.column_metadata)?;
        let commitments = self
            .commitments
            .try_sub(other.commitments)
            .expect("we've already checked that self and other have equal column counts");

        Ok(ColumnCommitments {
            column_metadata,
            commitments,
        })
    }
}

/// Owning iterator for [`ColumnCommitments`].
pub type IntoIter<C> = iter::Map<
    iter::Zip<<ColumnCommitmentMetadataMap as IntoIterator>::IntoIter, vec::IntoIter<C>>,
    fn(((Identifier, ColumnCommitmentMetadata), C)) -> (Identifier, ColumnCommitmentMetadata, C),
>;

impl<C> IntoIterator for ColumnCommitments<C> {
    type Item = (Identifier, ColumnCommitmentMetadata, C);
    type IntoIter = IntoIter<C>;
    fn into_iter(self) -> Self::IntoIter {
        self.column_metadata
            .into_iter()
            .zip(self.commitments)
            .map(|((identifier, metadata), commitment)| (identifier, metadata, commitment))
    }
}

/// Borrowing iterator for [`ColumnCommitments`].
pub type Iter<'a, C> = iter::Map<
    iter::Zip<<&'a ColumnCommitmentMetadataMap as IntoIterator>::IntoIter, slice::Iter<'a, C>>,
    fn(
        ((&'a Identifier, &'a ColumnCommitmentMetadata), &'a C),
    ) -> (&'a Identifier, &'a ColumnCommitmentMetadata, &'a C),
>;

impl<'a, C> IntoIterator for &'a ColumnCommitments<C> {
    type Item = (&'a Identifier, &'a ColumnCommitmentMetadata, &'a C);
    type IntoIter = Iter<'a, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.column_metadata
            .iter()
            .zip(self.commitments.iter())
            .map(|((identifier, metadata), commitment)| (identifier, metadata, commitment))
    }
}

impl<C> FromIterator<(Identifier, ColumnCommitmentMetadata, C)> for ColumnCommitments<C> {
    fn from_iter<T: IntoIterator<Item = (Identifier, ColumnCommitmentMetadata, C)>>(
        iter: T,
    ) -> Self {
        let (column_metadata, commitments) = iter
            .into_iter()
            .map(|(identifier, metadata, commitment)| ((identifier, metadata), commitment))
            .unzip();

        ColumnCommitments {
            commitments,
            column_metadata,
        }
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::base::{
        commitment::{column_bounds::Bounds, ColumnBounds},
        database::{owned_table_utility::*, ColumnType, OwnedColumn, OwnedTable},
        scalar::Curve25519Scalar,
    };
    use curve25519_dalek::RistrettoPoint;

    #[test]
    fn we_can_construct_column_commitments_from_columns_and_identifiers() {
        // empty case
        let column_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset::<
                &OwnedColumn<Curve25519Scalar>,
            >([], 0, &())
            .unwrap();
        assert_eq!(column_commitments.len(), 0);
        assert!(column_commitments.is_empty());
        assert!(column_commitments.commitments().is_empty());
        assert!(column_commitments.column_metadata().is_empty());

        // nonempty case
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let owned_table = owned_table::<Curve25519Scalar>([
            bigint(bigint_id, [1, 5, -5, 0]),
            // "int128_column" => [100i128, 200, 300, 400], TODO: enable this column once blitzar
            // supports it
            varchar(varchar_id, ["Lorem", "ipsum", "dolor", "sit"]),
            scalar(scalar_id, [1000, 2000, -1000, 0]),
        ]);

        let column_commitments = ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
            owned_table.inner_table(),
            0,
            &(),
        )
        .unwrap();

        assert_eq!(column_commitments.len(), 3);

        let expected_commitments = Vec::<RistrettoPoint>::from_columns_with_offset(
            owned_table.inner_table().values(),
            0,
            &(),
        );
        assert_eq!(column_commitments.commitments(), &expected_commitments);

        assert_eq!(
            column_commitments
                .column_metadata()
                .keys()
                .collect::<Vec<_>>(),
            vec![&bigint_id, &varchar_id, &scalar_id],
        );

        assert_eq!(
            column_commitments
                .get_metadata(&bigint_id)
                .unwrap()
                .column_type(),
            &ColumnType::BigInt
        );
        assert_eq!(
            column_commitments.get_commitment(&bigint_id).unwrap(),
            expected_commitments[0]
        );

        assert_eq!(
            column_commitments
                .get_metadata(&varchar_id)
                .unwrap()
                .column_type(),
            &ColumnType::VarChar
        );
        assert_eq!(
            column_commitments.get_commitment(&varchar_id).unwrap(),
            expected_commitments[1]
        );

        assert_eq!(
            column_commitments
                .get_metadata(&scalar_id)
                .unwrap()
                .column_type(),
            &ColumnType::Scalar
        );
        assert_eq!(
            column_commitments.get_commitment(&scalar_id).unwrap(),
            expected_commitments[2]
        );
    }

    #[test]
    fn we_can_construct_column_commitments_from_iter() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let owned_table = owned_table::<Curve25519Scalar>([
            bigint(bigint_id, [1, 5, -5, 0]),
            // "int128_column" => [100i128, 200, 300, 400], TODO: enable this column once blitzar
            // supports it
            varchar(varchar_id, ["Lorem", "ipsum", "dolor", "sit"]),
            scalar(scalar_id, [1000, 2000, -1000, 0]),
        ]);

        let column_commitments_from_columns =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                owned_table.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let column_commitments_from_iter =
            ColumnCommitments::from_iter(column_commitments_from_columns.clone());

        assert_eq!(
            column_commitments_from_iter,
            column_commitments_from_columns
        );
    }
    #[test]
    fn we_cannot_construct_commitments_with_duplicate_identifiers() {
        let duplicate_identifier_a = "duplicate_identifier_a".parse().unwrap();
        let duplicate_identifier_b = "duplicate_identifier_b".parse().unwrap();
        let unique_identifier = "unique_identifier".parse().unwrap();

        let empty_column = OwnedColumn::<Curve25519Scalar>::BigInt(vec![]);

        let from_columns_result = ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
            [
                (&duplicate_identifier_b, &empty_column),
                (&duplicate_identifier_b, &empty_column),
                (&unique_identifier, &empty_column),
            ],
            0,
            &(),
        );
        assert!(matches!(
            from_columns_result,
            Err(DuplicateIdentifiers { .. })
        ));

        let mut existing_column_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                [
                    (&duplicate_identifier_a, &empty_column),
                    (&unique_identifier, &empty_column),
                ],
                0,
                &(),
            )
            .unwrap();

        let extend_with_existing_column_result = existing_column_commitments
            .try_extend_columns_with_offset([(&duplicate_identifier_a, &empty_column)], 0, &());
        assert!(matches!(
            extend_with_existing_column_result,
            Err(DuplicateIdentifiers { .. })
        ));

        let extend_with_duplicate_columns_result = existing_column_commitments
            .try_extend_columns_with_offset(
                [
                    (&duplicate_identifier_b, &empty_column),
                    (&duplicate_identifier_b, &empty_column),
                ],
                0,
                &(),
            );
        assert!(matches!(
            extend_with_duplicate_columns_result,
            Err(DuplicateIdentifiers { .. })
        ));

        let append_result = existing_column_commitments.try_append_rows_with_offset(
            [
                (&duplicate_identifier_a, &empty_column),
                (&unique_identifier, &empty_column),
                (&duplicate_identifier_a, &empty_column),
            ],
            0,
            &(),
        );
        assert!(matches!(
            append_result,
            Err(AppendColumnCommitmentsError::DuplicateIdentifiers { .. })
        ));
    }

    #[test]
    fn we_can_iterate_over_column_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let owned_table = owned_table::<Curve25519Scalar>([
            bigint(bigint_id, [1, 5, -5, 0]),
            varchar(varchar_id, ["Lorem", "ipsum", "dolor", "sit"]),
            scalar(scalar_id, [1000, 2000, -1000, 0]),
        ]);
        let column_commitments = ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
            owned_table.inner_table(),
            0,
            &(),
        )
        .unwrap();

        let expected_commitments = Vec::<RistrettoPoint>::from_columns_with_offset(
            owned_table.inner_table().values(),
            0,
            &(),
        );

        let mut iterator = column_commitments.iter();

        let (identifier, metadata, commitment) = iterator.next().unwrap();
        assert_eq!(commitment, &expected_commitments[0]);
        assert_eq!(identifier, &bigint_id);
        assert_eq!(metadata.column_type(), &ColumnType::BigInt);

        let (identifier, metadata, commitment) = iterator.next().unwrap();
        assert_eq!(commitment, &expected_commitments[1]);
        assert_eq!(identifier, &varchar_id);
        assert_eq!(metadata.column_type(), &ColumnType::VarChar);

        let (identifier, metadata, commitment) = iterator.next().unwrap();
        assert_eq!(commitment, &expected_commitments[2]);
        assert_eq!(identifier, &scalar_id);
        assert_eq!(metadata.column_type(), &ColumnType::Scalar);
    }

    #[test]
    fn we_can_append_rows_to_column_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0];

        let initial_columns: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[..2].to_vec()),
            varchar(varchar_id, varchar_data[..2].to_vec()),
            scalar(scalar_id, scalar_data[..2].to_vec()),
        ]);

        let mut column_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                initial_columns.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let append_columns: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[2..].to_vec()),
            varchar(varchar_id, varchar_data[2..].to_vec()),
            scalar(scalar_id, scalar_data[2..].to_vec()),
        ]);

        column_commitments
            .try_append_rows_with_offset(append_columns.inner_table(), 2, &())
            .unwrap();

        let total_columns: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data),
            varchar(varchar_id, varchar_data),
            scalar(scalar_id, scalar_data),
        ]);

        let expected_column_commitments =
            ColumnCommitments::try_from_columns_with_offset(total_columns.inner_table(), 0, &())
                .unwrap();

        assert_eq!(column_commitments, expected_column_commitments);
    }

    #[test]
    fn we_cannot_append_rows_to_mismatched_column_commitments() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("column_a", [1, 2, 3, 4]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);
        let mut base_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                base_table.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table([
            varchar("column_a", ["5", "6", "7", "8"]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);
        assert!(matches!(
            base_commitments.try_append_rows_with_offset(table_diff_type.inner_table(), 4, &()),
            Err(AppendColumnCommitmentsError::Mismatch {
                source: ColumnCommitmentsMismatch::ColumnCommitmentMetadata { .. }
            })
        ));

        let table_diff_id: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("column_a", [5, 6, 7, 8]),
            varchar("b", ["amet", "ipsum", "dolor", "sit"]),
        ]);
        println!(
            "{:?}",
            base_commitments.try_append_rows_with_offset(table_diff_id.inner_table(), 4, &())
        );
        assert!(matches!(
            base_commitments.try_append_rows_with_offset(table_diff_id.inner_table(), 4, &()),
            Err(AppendColumnCommitmentsError::Mismatch {
                source: ColumnCommitmentsMismatch::Identifier { .. }
            })
        ));

        let table_diff_len: OwnedTable<Curve25519Scalar> =
            owned_table([bigint("column_a", [5, 6, 7, 8])]);
        assert!(matches!(
            base_commitments.try_append_rows_with_offset(table_diff_len.inner_table(), 4, &()),
            Err(AppendColumnCommitmentsError::Mismatch {
                source: ColumnCommitmentsMismatch::NumColumns
            })
        ));
    }

    #[test]
    fn we_can_extend_columns_to_column_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0];

        let initial_columns: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data),
            varchar(varchar_id, varchar_data),
        ]);
        let mut column_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                initial_columns.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let new_columns = owned_table::<Curve25519Scalar>([scalar(scalar_id, scalar_data)]);
        column_commitments
            .try_extend_columns_with_offset(new_columns.inner_table(), 0, &())
            .unwrap();

        let expected_columns = owned_table::<Curve25519Scalar>([
            bigint(bigint_id, bigint_data),
            varchar(varchar_id, varchar_data),
            scalar(scalar_id, scalar_data),
        ]);
        let expected_commitments =
            ColumnCommitments::try_from_columns_with_offset(expected_columns.inner_table(), 0, &())
                .unwrap();

        assert_eq!(column_commitments, expected_commitments);
    }

    #[test]
    fn we_can_add_column_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0];

        let columns_a: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[..2].to_vec()),
            varchar(varchar_id, varchar_data[..2].to_vec()),
            scalar(scalar_id, scalar_data[..2].to_vec()),
        ]);

        let column_commitments_a =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                columns_a.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let columns_b: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[2..].to_vec()),
            varchar(varchar_id, varchar_data[2..].to_vec()),
            scalar(scalar_id, scalar_data[2..].to_vec()),
        ]);
        let column_commitments_b =
            ColumnCommitments::try_from_columns_with_offset(columns_b.inner_table(), 2, &())
                .unwrap();

        let columns_sum: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data),
            varchar(varchar_id, varchar_data),
            scalar(scalar_id, scalar_data),
        ]);
        let column_commitments_sum =
            ColumnCommitments::try_from_columns_with_offset(columns_sum.inner_table(), 0, &())
                .unwrap();

        assert_eq!(
            column_commitments_a.try_add(column_commitments_b).unwrap(),
            column_commitments_sum
        );
    }

    #[test]
    fn we_cannot_add_mismatched_column_commitments() {
        let base_table: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("column_a", [1, 2, 3, 4]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);
        let base_commitments = ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
            base_table.inner_table(),
            0,
            &(),
        )
        .unwrap();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table([
            varchar("column_a", ["5", "6", "7", "8"]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);
        let commitments_diff_type =
            ColumnCommitments::try_from_columns_with_offset(table_diff_type.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            base_commitments.clone().try_add(commitments_diff_type),
            Err(ColumnCommitmentsMismatch::ColumnCommitmentMetadata { .. })
        ));

        let table_diff_id: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("column_a", [5, 6, 7, 8]),
            varchar("b", ["amet", "ipsum", "dolor", "sit"]),
        ]);
        let commitments_diff_id =
            ColumnCommitments::try_from_columns_with_offset(table_diff_id.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            base_commitments.clone().try_add(commitments_diff_id),
            Err(ColumnCommitmentsMismatch::Identifier { .. })
        ));

        let table_diff_len: OwnedTable<Curve25519Scalar> =
            owned_table([bigint("column_a", [5, 6, 7, 8])]);
        let commitments_diff_len =
            ColumnCommitments::try_from_columns_with_offset(table_diff_len.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            base_commitments.clone().try_add(commitments_diff_len),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));
    }

    #[test]
    fn we_can_sub_column_commitments() {
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let bigint_data = [1i64, 5, -5, 0, 10];

        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let scalar_data = [1000, 2000, 3000, -1000, 0];

        let columns_subtrahend: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[..2].to_vec()),
            varchar(varchar_id, varchar_data[..2].to_vec()),
            scalar(scalar_id, scalar_data[..2].to_vec()),
        ]);

        let column_commitments_subtrahend =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                columns_subtrahend.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let columns_minuend: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data),
            varchar(varchar_id, varchar_data),
            scalar(scalar_id, scalar_data),
        ]);
        let column_commitments_minuend =
            ColumnCommitments::try_from_columns_with_offset(columns_minuend.inner_table(), 0, &())
                .unwrap();

        let actual_difference = column_commitments_minuend
            .try_sub(column_commitments_subtrahend)
            .unwrap();

        let expected_difference_columns: OwnedTable<Curve25519Scalar> = owned_table([
            bigint(bigint_id, bigint_data[2..].to_vec()),
            varchar(varchar_id, varchar_data[2..].to_vec()),
            scalar(scalar_id, scalar_data[2..].to_vec()),
        ]);
        let expected_difference = ColumnCommitments::try_from_columns_with_offset(
            expected_difference_columns.inner_table(),
            2,
            &(),
        )
        .unwrap();

        assert_eq!(
            actual_difference.commitments(),
            expected_difference.commitments()
        );

        assert_eq!(
            actual_difference
                .column_metadata()
                .keys()
                .collect::<Vec<_>>(),
            vec![&bigint_id, &varchar_id, &scalar_id],
        );

        let bigint_metadata = actual_difference.get_metadata(&bigint_id).unwrap();
        assert_eq!(bigint_metadata.column_type(), &ColumnType::BigInt);
        if let ColumnBounds::BigInt(Bounds::Bounded(bounds)) = bigint_metadata.bounds() {
            assert_eq!(bounds.min(), &-5);
            assert_eq!(bounds.max(), &10);
        }

        let varchar_metadata = actual_difference.get_metadata(&varchar_id).unwrap();
        assert_eq!(varchar_metadata.column_type(), &ColumnType::VarChar);
        assert_eq!(varchar_metadata.bounds(), &ColumnBounds::NoOrder);

        let scalar_metadata = actual_difference.get_metadata(&scalar_id).unwrap();
        assert_eq!(scalar_metadata.column_type(), &ColumnType::Scalar);
        assert_eq!(scalar_metadata.bounds(), &ColumnBounds::NoOrder);
    }

    #[test]
    fn we_cannot_sub_mismatched_column_commitments() {
        let minuend_table: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("column_a", [1, 2, 3, 4]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);
        let minuend_commitments =
            ColumnCommitments::<RistrettoPoint>::try_from_columns_with_offset(
                minuend_table.inner_table(),
                0,
                &(),
            )
            .unwrap();

        let table_diff_type: OwnedTable<Curve25519Scalar> = owned_table([
            varchar("column_a", ["1", "2"]),
            varchar("column_b", ["Lorem", "ipsum"]),
        ]);
        let commitments_diff_type =
            ColumnCommitments::try_from_columns_with_offset(table_diff_type.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            minuend_commitments.clone().try_sub(commitments_diff_type),
            Err(ColumnCommitmentsMismatch::ColumnCommitmentMetadata { .. })
        ));

        let table_diff_id: OwnedTable<Curve25519Scalar> =
            owned_table([bigint("column_a", [1, 2]), varchar("b", ["Lorem", "ipsum"])]);
        let commitments_diff_id =
            ColumnCommitments::try_from_columns_with_offset(table_diff_id.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            minuend_commitments.clone().try_sub(commitments_diff_id),
            Err(ColumnCommitmentsMismatch::Identifier { .. })
        ));

        let table_diff_len: OwnedTable<Curve25519Scalar> =
            owned_table([bigint("column_a", [1, 2])]);
        let commitments_diff_len =
            ColumnCommitments::try_from_columns_with_offset(table_diff_len.inner_table(), 4, &())
                .unwrap();
        assert!(matches!(
            minuend_commitments.clone().try_sub(commitments_diff_len),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));
    }
}
