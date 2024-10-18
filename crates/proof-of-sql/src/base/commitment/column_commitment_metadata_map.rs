use super::{
    column_commitment_metadata::ColumnCommitmentMetadataMismatch, ColumnCommitmentMetadata,
    CommittableColumn,
};
use crate::base::{database::ColumnField, map::IndexMap};
use alloc::string::{String, ToString};
use proof_of_sql_parser::Identifier;
use snafu::Snafu;

/// Mapping of column identifiers to column metadata used to associate metadata with commitments.
pub type ColumnCommitmentMetadataMap = IndexMap<Identifier, ColumnCommitmentMetadata>;

/// During commitment operation, metadata indicates that operand tables cannot be the same.
#[derive(Debug, Snafu)]
pub enum ColumnCommitmentsMismatch {
    /// Anonymous metadata indicates a column mismatch.
    #[snafu(transparent)]
    ColumnCommitmentMetadata {
        /// The underlying source error
        source: ColumnCommitmentMetadataMismatch,
    },
    /// Commitments with different column counts cannot operate with each other.
    #[snafu(display("commitments with different column counts cannot operate with each other"))]
    NumColumns,
    /// Columns with mismatched identifiers cannot operate with each other.
    ///
    /// Strings are used here instead of Identifiers to decrease the size of this variant
    #[snafu(display(
        "column with identifier {id_a} cannot operate with column with identifier {id_b}"
    ))]
    Identifier {
        /// The first column identifier
        id_a: String,
        /// The second column identifier
        id_b: String,
    },
}

/// Extension trait intended for [`ColumnCommitmentMetadataMap`].
pub trait ColumnCommitmentMetadataMapExt {
    /// Construct this mapping from a slice of column fields, with the bounds of each column set to
    /// the widest possible bounds for the column type.
    fn from_column_fields_with_max_bounds(columns: &[ColumnField]) -> Self;

    /// Construct this mapping from an iterator of column identifiers and columns.
    fn from_columns<'a>(
        columns: impl IntoIterator<Item = (&'a Identifier, &'a CommittableColumn<'a>)>,
    ) -> Self
    where
        Self: Sized;

    /// Combine two metadata maps as if the source table commitments are being unioned.
    fn try_union(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized;

    /// Combine two metadata maps as if the source table commitments are being differenced.
    fn try_difference(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized;
}

impl ColumnCommitmentMetadataMapExt for ColumnCommitmentMetadataMap {
    fn from_column_fields_with_max_bounds(columns: &[ColumnField]) -> Self {
        columns
            .iter()
            .map(|f| {
                (
                    f.name(),
                    ColumnCommitmentMetadata::from_column_type_with_max_bounds(f.data_type()),
                )
            })
            .collect()
    }

    fn from_columns<'a>(
        columns: impl IntoIterator<Item = (&'a Identifier, &'a CommittableColumn<'a>)>,
    ) -> Self
    where
        Self: Sized,
    {
        columns
            .into_iter()
            .map(|(identifier, column)| {
                (*identifier, ColumnCommitmentMetadata::from_column(column))
            })
            .collect()
    }

    fn try_union(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized,
    {
        if self.len() != other.len() {
            return Err(ColumnCommitmentsMismatch::NumColumns);
        }

        self.into_iter()
            .zip(other)
            .map(|((identifier_a, metadata_a), (identifier_b, metadata_b))| {
                if identifier_a != identifier_b {
                    Err(ColumnCommitmentsMismatch::Identifier {
                        id_a: identifier_a.to_string(),
                        id_b: identifier_b.to_string(),
                    })?;
                }

                Ok((identifier_a, metadata_a.try_union(metadata_b)?))
            })
            .collect()
    }

    fn try_difference(self, other: Self) -> Result<Self, ColumnCommitmentsMismatch>
    where
        Self: Sized,
    {
        if self.len() != other.len() {
            return Err(ColumnCommitmentsMismatch::NumColumns);
        }

        self.into_iter()
            .zip(other)
            .map(|((identifier_a, metadata_a), (identifier_b, metadata_b))| {
                if identifier_a != identifier_b {
                    Err(ColumnCommitmentsMismatch::Identifier {
                        id_a: identifier_a.to_string(),
                        id_b: identifier_b.to_string(),
                    })?;
                }

                Ok((identifier_a, metadata_a.try_difference(metadata_b)?))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::{column_bounds::Bounds, ColumnBounds},
        database::{owned_table_utility::*, ColumnType, ColumnTypeAssociatedData, OwnedTable},
        scalar::Curve25519Scalar,
    };
    use alloc::vec::Vec;
    use itertools::Itertools;

    fn metadata_map_from_owned_table(
        table: OwnedTable<Curve25519Scalar>,
    ) -> ColumnCommitmentMetadataMap {
        let (identifiers, columns): (Vec<&Identifier>, Vec<CommittableColumn>) = table
            .inner_table()
            .into_iter()
            .map(|(identifier, owned_column)| (identifier, CommittableColumn::from(owned_column)))
            .unzip();

        ColumnCommitmentMetadataMap::from_columns(identifiers.into_iter().zip(columns.iter()))
    }

    #[test]
    fn we_can_construct_metadata_map_from_columns() {
        let col_meta = ColumnTypeAssociatedData::NOT_NULLABLE;
        // No-columns case
        let empty_metadata_map = ColumnCommitmentMetadataMap::from_columns([]);
        assert_eq!(empty_metadata_map.len(), 0);

        // With columns
        let table: OwnedTable<Curve25519Scalar> = owned_table([
            bigint("bigint_column", [1, 5, -5, 0]),
            int128("int128_column", [100, 200, 300, 400]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit"]),
            scalar("scalar_column", [1000, 2000, -1000, 0]),
        ]);

        let metadata_map = metadata_map_from_owned_table(table);

        assert_eq!(metadata_map.len(), 4);

        let (index_0, metadata_0) = metadata_map.get_index(0).unwrap();
        assert_eq!(index_0, "bigint_column");
        assert_eq!(metadata_0.column_type(), &ColumnType::BigInt(col_meta));
        if let ColumnBounds::BigInt(Bounds::Sharp(bounds)) = metadata_0.bounds() {
            assert_eq!(bounds.min(), &-5);
            assert_eq!(bounds.max(), &5);
        } else {
            panic!("metadata constructed from BigInt column should have BigInt/Sharp bounds");
        }

        let (index_1, metadata_1) = metadata_map.get_index(1).unwrap();
        assert_eq!(index_1, "int128_column");
        assert_eq!(metadata_1.column_type(), &ColumnType::Int128(col_meta));
        if let ColumnBounds::Int128(Bounds::Sharp(bounds)) = metadata_1.bounds() {
            assert_eq!(bounds.min(), &100);
            assert_eq!(bounds.max(), &400);
        } else {
            panic!("metadata constructed from Int128 column should have Int128/Sharp bounds");
        }

        let (index_2, metadata_2) = metadata_map.get_index(2).unwrap();
        assert_eq!(index_2, "varchar_column");
        assert_eq!(metadata_2.column_type(), &ColumnType::VarChar(col_meta));
        assert_eq!(metadata_2.bounds(), &ColumnBounds::NoOrder);

        let (index_3, metadata_3) = metadata_map.get_index(3).unwrap();
        assert_eq!(index_3, "scalar_column");
        assert_eq!(metadata_3.column_type(), &ColumnType::Scalar(col_meta));
        assert_eq!(metadata_3.bounds(), &ColumnBounds::NoOrder);
    }

    #[test]
    fn we_can_union_matching_metadata_maps() {
        let table_a = owned_table([
            bigint("bigint_column", [1, 5]),
            int128("int128_column", [100, 200]),
            varchar("varchar_column", ["Lorem", "ipsum"]),
            scalar("scalar_column", [1000, 2000]),
        ]);
        let metadata_a = metadata_map_from_owned_table(table_a);

        let table_b = owned_table([
            bigint("bigint_column", [-5, 0, 10]),
            int128("int128_column", [300, 400, 500]),
            varchar("varchar_column", ["dolor", "sit", "amet"]),
            scalar("scalar_column", [-1000, 0, -2000]),
        ]);
        let metadata_b = metadata_map_from_owned_table(table_b);

        let table_c = owned_table([
            bigint("bigint_column", [1, 5, -5, 0, 10]),
            int128("int128_column", [100, 200, 300, 400, 500]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit", "amet"]),
            scalar("scalar_column", [1000, 2000, -1000, 0, -2000]),
        ]);
        let metadata_c = metadata_map_from_owned_table(table_c);

        assert_eq!(metadata_a.try_union(metadata_b).unwrap(), metadata_c);
    }
    #[test]
    fn we_can_difference_matching_metadata_maps() {
        let col_meta = ColumnTypeAssociatedData::NOT_NULLABLE;
        let table_a = owned_table([
            bigint("bigint_column", [1, 5]),
            int128("int128_column", [100, 200]),
            varchar("varchar_column", ["Lorem", "ipsum"]),
            scalar("scalar_column", [1000, 2000]),
        ]);
        let metadata_a = metadata_map_from_owned_table(table_a);

        let table_b = owned_table([
            bigint("bigint_column", [1, 5, -5, 0, 10]),
            int128("int128_column", [100, 200, 300, 400, 500]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit", "amet"]),
            scalar("scalar_column", [1000, 2000, -1000, 0, -2000]),
        ]);
        let metadata_b = metadata_map_from_owned_table(table_b);

        let b_difference_a = metadata_b.try_difference(metadata_a.clone()).unwrap();

        assert_eq!(b_difference_a.len(), 4);

        // Check metatadata for ordered columns is mostly the same (now bounded)
        let (index_0, metadata_0) = b_difference_a.get_index(0).unwrap();
        assert_eq!(index_0, "bigint_column");
        assert_eq!(metadata_0.column_type(), &ColumnType::BigInt(col_meta));
        if let ColumnBounds::BigInt(Bounds::Bounded(bounds)) = metadata_0.bounds() {
            assert_eq!(bounds.min(), &-5);
            assert_eq!(bounds.max(), &10);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let (index_1, metadata_1) = b_difference_a.get_index(1).unwrap();
        assert_eq!(index_1, "int128_column");
        assert_eq!(metadata_1.column_type(), &ColumnType::Int128(col_meta));
        if let ColumnBounds::Int128(Bounds::Bounded(bounds)) = metadata_1.bounds() {
            assert_eq!(bounds.min(), &100);
            assert_eq!(bounds.max(), &500);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        // Check metadata for unordered columns remains the same
        assert_eq!(
            b_difference_a.get_index(2).unwrap(),
            metadata_a.get_index(2).unwrap()
        );

        assert_eq!(
            b_difference_a.get_index(3).unwrap(),
            metadata_a.get_index(3).unwrap()
        );
    }

    #[test]
    fn we_cannot_perform_arithmetic_on_metadata_maps_with_different_column_counts() {
        let table_a = owned_table([
            bigint("bigint_column", [1, 5, -5, 0, 10]),
            int128("int128_column", [100, 200, 300, 400, 500]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit", "amet"]),
            scalar("scalar_column", [1000, 2000, -1000, 0, -2000]),
        ]);
        let metadata_a = metadata_map_from_owned_table(table_a);

        let table_b = owned_table([
            bigint("bigint_column", [1, 5, -5, 0, 10]),
            varchar("varchar_column", ["Lorem", "ipsum", "dolor", "sit", "amet"]),
        ]);
        let metadata_b = metadata_map_from_owned_table(table_b);

        assert!(matches!(
            metadata_a.clone().try_union(metadata_b.clone()),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));
        assert!(matches!(
            metadata_b.try_union(metadata_a.clone()),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));

        let emtpy_metadata = ColumnCommitmentMetadataMap::default();

        assert!(matches!(
            metadata_a.clone().try_union(emtpy_metadata.clone()),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));
        assert!(matches!(
            emtpy_metadata.try_union(metadata_a),
            Err(ColumnCommitmentsMismatch::NumColumns)
        ));
    }

    #[allow(clippy::similar_names)]
    #[test]
    fn we_cannot_perform_arithmetic_on_mismatched_metadata_maps_with_same_column_counts() {
        let id_a = "column_a";
        let id_b = "column_b";
        let id_c = "column_c";
        let id_d = "column_d";
        let ints = [1i64, 2, 3, 4];
        let strings = ["Lorem", "ipsum", "dolor", "sit"];

        let ab_ii_metadata =
            metadata_map_from_owned_table(owned_table([bigint(id_a, ints), bigint(id_b, ints)]));

        let ab_iv_metadata = metadata_map_from_owned_table(owned_table([
            bigint(id_a, ints),
            varchar(id_b, strings),
        ]));

        let ab_vi_metadata = metadata_map_from_owned_table(owned_table([
            varchar(id_a, strings),
            bigint(id_b, ints),
        ]));

        let ad_ii_metadata =
            metadata_map_from_owned_table(owned_table([bigint(id_a, ints), bigint(id_d, ints)]));

        let cb_ii_metadata =
            metadata_map_from_owned_table(owned_table([bigint(id_c, ints), bigint(id_b, ints)]));

        let cd_vv_metadata = metadata_map_from_owned_table(owned_table([
            varchar(id_c, strings),
            varchar(id_d, strings),
        ]));

        // each pairwise combination of these maps is a different kind of mismatch
        // these combinations cover every possible way 2 tables with 2 columns could mismatch
        let mismatched_metadata_maps = [
            ab_ii_metadata,
            ab_iv_metadata,
            ab_vi_metadata,
            ad_ii_metadata,
            cb_ii_metadata,
            cd_vv_metadata,
        ];

        for (metadata_map_a, metadata_map_b) in
            mismatched_metadata_maps.into_iter().tuple_combinations()
        {
            assert!(metadata_map_a
                .clone()
                .try_union(metadata_map_b.clone())
                .is_err());
            assert!(metadata_map_b
                .clone()
                .try_union(metadata_map_a.clone())
                .is_err());
            assert!(metadata_map_a
                .clone()
                .try_difference(metadata_map_b.clone())
                .is_err());
            assert!(metadata_map_b.try_difference(metadata_map_a).is_err());
        }
    }
}
