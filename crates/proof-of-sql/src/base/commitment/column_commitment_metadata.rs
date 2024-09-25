use super::{column_bounds::BoundsInner, committable_column::CommittableColumn, ColumnBounds};
use crate::base::database::ColumnType;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur when constructing invalid [`ColumnCommitmentMetadata`].
#[derive(Debug, Error)]
pub enum InvalidColumnCommitmentMetadata {
    /// Column of this type cannot have these bounds.
    #[error("column of type {0} cannot have bounds like {1:?}")]
    TypeBoundsMismatch(ColumnType, ColumnBounds),
}

/// During column operation, metadata indicates that the operand columns cannot be the same.
#[derive(Debug, Error)]
#[error("column with type {0} cannot operate with column with type {1}")]
pub struct ColumnCommitmentMetadataMismatch(ColumnType, ColumnType);

const EXPECT_BOUNDS_MATCH_MESSAGE: &str = "we've already checked the column types match, which is a stronger requirement (mapping of type variants to bounds variants is surjective)";

/// Anonymous metadata associated with a column commitment.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnCommitmentMetadata {
    column_type: ColumnType,
    bounds: ColumnBounds,
}

impl ColumnCommitmentMetadata {
    /// Construct a new [`ColumnCommitmentMetadata`].
    ///
    /// Will error if the supplied metadata are invalid.
    /// i.e., if The Bounds variant and column type do not match.
    pub fn try_new(
        column_type: ColumnType,
        bounds: ColumnBounds,
    ) -> Result<ColumnCommitmentMetadata, InvalidColumnCommitmentMetadata> {
        match (column_type, bounds) {
            (ColumnType::SmallInt, ColumnBounds::SmallInt(_))
            | (ColumnType::Int, ColumnBounds::Int(_))
            | (ColumnType::BigInt, ColumnBounds::BigInt(_))
            | (ColumnType::Int128, ColumnBounds::Int128(_))
            | (ColumnType::TimestampTZ(_, _), ColumnBounds::TimestampTZ(_))
            | (
                ColumnType::Boolean
                | ColumnType::VarChar
                | ColumnType::Scalar
                | ColumnType::Decimal75(..),
                ColumnBounds::NoOrder,
            ) => Ok(ColumnCommitmentMetadata {
                column_type,
                bounds,
            }),
            _ => Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(
                column_type,
                bounds,
            )),
        }
    }

    /// Construct a [`ColumnCommitmentMetadata`] with widest possible bounds for the column type.
    pub fn from_column_type_with_max_bounds(column_type: ColumnType) -> Self {
        let bounds = match column_type {
            ColumnType::SmallInt => ColumnBounds::SmallInt(super::Bounds::Bounded(
                BoundsInner::try_new(i16::MIN, i16::MAX)
                    .expect("i16::MIN and i16::MAX are valid bounds for SmallInt"),
            )),
            ColumnType::Int => ColumnBounds::Int(super::Bounds::Bounded(
                BoundsInner::try_new(i32::MIN, i32::MAX)
                    .expect("i32::MIN and i32::MAX are valid bounds for Int"),
            )),
            ColumnType::BigInt => ColumnBounds::BigInt(super::Bounds::Bounded(
                BoundsInner::try_new(i64::MIN, i64::MAX)
                    .expect("i64::MIN and i64::MAX are valid bounds for BigInt"),
            )),
            ColumnType::TimestampTZ(_, _) => ColumnBounds::TimestampTZ(super::Bounds::Bounded(
                BoundsInner::try_new(i64::MIN, i64::MAX)
                    .expect("i64::MIN and i64::MAX are valid bounds for TimeStamp"),
            )),
            ColumnType::Int128 => ColumnBounds::Int128(super::Bounds::Bounded(
                BoundsInner::try_new(i128::MIN, i128::MAX)
                    .expect("i128::MIN and i128::MAX are valid bounds for Int128"),
            )),
            _ => ColumnBounds::NoOrder,
        };
        Self::try_new(column_type, bounds).expect("default bounds for column type are valid")
    }

    #[cfg(test)]
    pub(super) fn bounds_mut(&mut self) -> &mut ColumnBounds {
        &mut self.bounds
    }

    /// Immutable reference to this column's type.
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }

    /// Immutable reference to this column's bounds.
    pub fn bounds(&self) -> &ColumnBounds {
        &self.bounds
    }

    /// Contruct a [`ColumnCommitmentMetadata`] by analyzing a column.
    pub fn from_column(column: &CommittableColumn) -> ColumnCommitmentMetadata {
        ColumnCommitmentMetadata {
            column_type: column.column_type(),
            bounds: ColumnBounds::from_column(column),
        }
    }

    /// Combine two [`ColumnCommitmentMetadata`] as if their source collections are being unioned.
    ///
    /// Can error if the two metadatas are mismatched.
    pub fn try_union(
        self,
        other: ColumnCommitmentMetadata,
    ) -> Result<ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch> {
        if self.column_type != other.column_type {
            return Err(ColumnCommitmentMetadataMismatch(
                self.column_type,
                other.column_type,
            ));
        }

        let bounds = self
            .bounds
            .try_union(other.bounds)
            .expect(EXPECT_BOUNDS_MATCH_MESSAGE);

        Ok(ColumnCommitmentMetadata {
            bounds,
            column_type: self.column_type,
        })
    }

    /// Combine two [`ColumnBounds`] as if their source collections are being differenced.
    ///
    /// This should be interpreted as the set difference of the two collections.
    /// The result would be the rows in self that are not also rows in other.
    pub fn try_difference(
        self,
        other: ColumnCommitmentMetadata,
    ) -> Result<ColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch> {
        if self.column_type != other.column_type {
            return Err(ColumnCommitmentMetadataMismatch(
                self.column_type,
                other.column_type,
            ));
        }

        let bounds = self
            .bounds
            .try_difference(other.bounds)
            .expect(EXPECT_BOUNDS_MATCH_MESSAGE);

        Ok(ColumnCommitmentMetadata {
            bounds,
            column_type: self.column_type,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::base::{
        commitment::column_bounds::Bounds, database::OwnedColumn, math::decimal::Precision,
        scalar::test_scalar::TestScalar,
    };
    use alloc::string::String;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_construct_metadata() {
        assert_eq!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::SmallInt,
                ColumnBounds::SmallInt(Bounds::Empty)
            )
            .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::SmallInt,
                bounds: ColumnBounds::SmallInt(Bounds::Empty)
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(ColumnType::Int, ColumnBounds::Int(Bounds::Empty))
                .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::Int,
                bounds: ColumnBounds::Int(Bounds::Empty)
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::BigInt,
                ColumnBounds::BigInt(Bounds::Empty)
            )
            .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::BigInt,
                bounds: ColumnBounds::BigInt(Bounds::Empty)
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(ColumnType::Boolean, ColumnBounds::NoOrder,).unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::Boolean,
                bounds: ColumnBounds::NoOrder,
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Decimal75(Precision::new(10).unwrap(), 0),
                ColumnBounds::NoOrder,
            )
            .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::Decimal75(Precision::new(10).unwrap(), 0),
                bounds: ColumnBounds::NoOrder,
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc),
                ColumnBounds::TimestampTZ(Bounds::Empty),
            )
            .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc),
                bounds: ColumnBounds::TimestampTZ(Bounds::Empty),
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Int128,
                ColumnBounds::Int128(Bounds::sharp(-5, 10).unwrap())
            )
            .unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::Int128,
                bounds: ColumnBounds::Int128(Bounds::sharp(-5, 10).unwrap())
            }
        );

        assert_eq!(
            ColumnCommitmentMetadata::try_new(ColumnType::VarChar, ColumnBounds::NoOrder).unwrap(),
            ColumnCommitmentMetadata {
                column_type: ColumnType::VarChar,
                bounds: ColumnBounds::NoOrder
            }
        );
    }

    #[test]
    fn we_cannot_construct_metadata_with_type_bounds_mismatch() {
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Boolean,
                ColumnBounds::BigInt(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Boolean,
                ColumnBounds::Int128(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));

        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Decimal75(Precision::new(10).unwrap(), 10),
                ColumnBounds::Int128(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Decimal75(Precision::new(10).unwrap(), 10),
                ColumnBounds::BigInt(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));

        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Scalar,
                ColumnBounds::BigInt(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Scalar,
                ColumnBounds::Int128(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));

        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::BigInt,
                ColumnBounds::Int128(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(ColumnType::BigInt, ColumnBounds::NoOrder),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));

        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::Int128,
                ColumnBounds::BigInt(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(ColumnType::Int128, ColumnBounds::NoOrder),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));

        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::VarChar,
                ColumnBounds::BigInt(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
        assert!(matches!(
            ColumnCommitmentMetadata::try_new(
                ColumnType::VarChar,
                ColumnBounds::Int128(Bounds::Empty)
            ),
            Err(InvalidColumnCommitmentMetadata::TypeBoundsMismatch(..))
        ));
    }

    #[test]
    fn we_can_construct_metadata_from_column() {
        let boolean_column =
            OwnedColumn::<TestScalar>::Boolean([true, false, true, false, true].to_vec());
        let committable_boolean_column = CommittableColumn::from(&boolean_column);
        let boolean_metadata = ColumnCommitmentMetadata::from_column(&committable_boolean_column);
        assert_eq!(boolean_metadata.column_type(), &ColumnType::Boolean);
        assert_eq!(boolean_metadata.bounds(), &ColumnBounds::NoOrder);

        let decimal_column = OwnedColumn::<TestScalar>::Decimal75(
            Precision::new(10).unwrap(),
            0,
            [1, 2, 3, 4, 5].map(TestScalar::from).to_vec(),
        );
        let committable_decimal_column = CommittableColumn::from(&decimal_column);
        let decimal_metadata = ColumnCommitmentMetadata::from_column(&committable_decimal_column);
        assert_eq!(
            decimal_metadata.column_type(),
            &ColumnType::Decimal75(Precision::new(10).unwrap(), 0)
        );
        assert_eq!(decimal_metadata.bounds(), &ColumnBounds::NoOrder);

        let timestamp_column: OwnedColumn<TestScalar> = OwnedColumn::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [1i64, 2, 3, 4, 5].to_vec(),
        );
        let committable_timestamp_column = CommittableColumn::from(&timestamp_column);
        let timestamp_metadata =
            ColumnCommitmentMetadata::from_column(&committable_timestamp_column);
        assert_eq!(
            timestamp_metadata.column_type(),
            &ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc)
        );
        if let ColumnBounds::TimestampTZ(Bounds::Sharp(bounds)) = timestamp_metadata.bounds() {
            assert_eq!(bounds.min(), &1);
            assert_eq!(bounds.max(), &5);
        } else {
            panic!("Bounds constructed from nonempty TimestampTZ column should be ColumnBounds::BigInt(Bounds::Sharp(_))");
        }

        let varchar_column = OwnedColumn::<TestScalar>::VarChar(
            ["Lorem", "ipsum", "dolor", "sit", "amet"]
                .map(String::from)
                .to_vec(),
        );
        let committable_varchar_column = CommittableColumn::from(&varchar_column);
        let varchar_metadata = ColumnCommitmentMetadata::from_column(&committable_varchar_column);
        assert_eq!(varchar_metadata.column_type(), &ColumnType::VarChar);
        assert_eq!(varchar_metadata.bounds(), &ColumnBounds::NoOrder);

        let bigint_column = OwnedColumn::<TestScalar>::BigInt([1, 2, 3, 1, 0].to_vec());
        let committable_bigint_column = CommittableColumn::from(&bigint_column);
        let bigint_metadata = ColumnCommitmentMetadata::from_column(&committable_bigint_column);
        assert_eq!(bigint_metadata.column_type(), &ColumnType::BigInt);
        if let ColumnBounds::BigInt(Bounds::Sharp(bounds)) = bigint_metadata.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("Bounds constructed from nonempty BigInt column should be ColumnBounds::BigInt(Bounds::Sharp(_))");
        }

        let int_column = OwnedColumn::<TestScalar>::Int([1, 2, 3, 1, 0].to_vec());
        let committable_int_column = CommittableColumn::from(&int_column);
        let int_metadata = ColumnCommitmentMetadata::from_column(&committable_int_column);
        assert_eq!(int_metadata.column_type(), &ColumnType::Int);
        if let ColumnBounds::Int(Bounds::Sharp(bounds)) = int_metadata.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("Bounds constructed from nonempty BigInt column should be ColumnBounds::Int(Bounds::Sharp(_))");
        }

        let smallint_column = OwnedColumn::<TestScalar>::SmallInt([1, 2, 3, 1, 0].to_vec());
        let committable_smallint_column = CommittableColumn::from(&smallint_column);
        let smallint_metadata = ColumnCommitmentMetadata::from_column(&committable_smallint_column);
        assert_eq!(smallint_metadata.column_type(), &ColumnType::SmallInt);
        if let ColumnBounds::SmallInt(Bounds::Sharp(bounds)) = smallint_metadata.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("Bounds constructed from nonempty BigInt column should be ColumnBounds::SmallInt(Bounds::Sharp(_))");
        }

        let int128_column = OwnedColumn::<TestScalar>::Int128([].to_vec());
        let committable_int128_column = CommittableColumn::from(&int128_column);
        let int128_metadata = ColumnCommitmentMetadata::from_column(&committable_int128_column);
        assert_eq!(int128_metadata.column_type(), &ColumnType::Int128);
        assert_eq!(
            int128_metadata.bounds(),
            &ColumnBounds::Int128(Bounds::Empty)
        );

        let scalar_column = OwnedColumn::Scalar([1, 2, 3, 4, 5].map(TestScalar::from).to_vec());
        let committable_scalar_column = CommittableColumn::from(&scalar_column);
        let scalar_metadata = ColumnCommitmentMetadata::from_column(&committable_scalar_column);
        assert_eq!(scalar_metadata.column_type(), &ColumnType::Scalar);
        assert_eq!(scalar_metadata.bounds(), &ColumnBounds::NoOrder);
    }

    #[test]
    fn we_can_union_matching_metadata() {
        // NoOrder cases
        let boolean_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Boolean,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            boolean_metadata.try_union(boolean_metadata).unwrap(),
            boolean_metadata
        );

        let decimal_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Decimal75(Precision::new(12).unwrap(), 0),
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            decimal_metadata.try_union(decimal_metadata).unwrap(),
            decimal_metadata
        );

        let varchar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::VarChar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            varchar_metadata.try_union(varchar_metadata).unwrap(),
            varchar_metadata
        );

        let scalar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Scalar,
            bounds: ColumnBounds::NoOrder,
        };
        assert_eq!(
            scalar_metadata.try_union(scalar_metadata).unwrap(),
            scalar_metadata
        );

        // Ordered case
        let ints = [1, 2, 3, 1, 0];
        let smallint_column_a = CommittableColumn::SmallInt(&ints[..2]);
        let smallint_metadata_a = ColumnCommitmentMetadata::from_column(&smallint_column_a);
        let smallint_column_b = CommittableColumn::SmallInt(&ints[2..]);
        let smallint_metadata_b = ColumnCommitmentMetadata::from_column(&smallint_column_b);
        let smallint_column_c = CommittableColumn::SmallInt(&ints);
        let smallint_metadata_c = ColumnCommitmentMetadata::from_column(&smallint_column_c);
        assert_eq!(
            smallint_metadata_a.try_union(smallint_metadata_b).unwrap(),
            smallint_metadata_c
        );

        let ints = [1, 2, 3, 1, 0];
        let int_column_a = CommittableColumn::Int(&ints[..2]);
        let int_metadata_a = ColumnCommitmentMetadata::from_column(&int_column_a);
        let int_column_b = CommittableColumn::Int(&ints[2..]);
        let int_metadata_b = ColumnCommitmentMetadata::from_column(&int_column_b);
        let int_column_c = CommittableColumn::Int(&ints);
        let int_metadata_c = ColumnCommitmentMetadata::from_column(&int_column_c);
        assert_eq!(
            int_metadata_a.try_union(int_metadata_b).unwrap(),
            int_metadata_c
        );

        let ints = [1, 2, 3, 1, 0];
        let bigint_column_a = CommittableColumn::BigInt(&ints[..2]);
        let bigint_metadata_a = ColumnCommitmentMetadata::from_column(&bigint_column_a);
        let bigint_column_b = CommittableColumn::BigInt(&ints[2..]);
        let bigint_metadata_b = ColumnCommitmentMetadata::from_column(&bigint_column_b);
        let bigint_column_c = CommittableColumn::BigInt(&ints);
        let bigint_metadata_c = ColumnCommitmentMetadata::from_column(&bigint_column_c);
        assert_eq!(
            bigint_metadata_a.try_union(bigint_metadata_b).unwrap(),
            bigint_metadata_c
        );

        // Ordered case for TimestampTZ
        // Example Unix epoch times
        let times = [
            1_625_072_400,
            1_625_076_000,
            1_625_079_600,
            1_625_072_400,
            1_625_065_000,
        ];
        let timezone = PoSQLTimeZone::Utc;
        let timeunit = PoSQLTimeUnit::Second;
        let timestamp_column_a = CommittableColumn::TimestampTZ(timeunit, timezone, &times[..2]);
        let timestamp_metadata_a = ColumnCommitmentMetadata::from_column(&timestamp_column_a);
        let timestamp_column_b = CommittableColumn::TimestampTZ(timeunit, timezone, &times[2..]);
        let timestamp_metadata_b = ColumnCommitmentMetadata::from_column(&timestamp_column_b);
        let timestamp_column_c = CommittableColumn::TimestampTZ(timeunit, timezone, &times);
        let timestamp_metadata_c = ColumnCommitmentMetadata::from_column(&timestamp_column_c);
        assert_eq!(
            timestamp_metadata_a
                .try_union(timestamp_metadata_b)
                .unwrap(),
            timestamp_metadata_c
        );
    }

    #[test]
    fn we_can_difference_timestamp_tz_matching_metadata() {
        // Ordered case
        let times = [
            1_625_072_400,
            1_625_076_000,
            1_625_079_600,
            1_625_072_400,
            1_625_065_000,
        ];
        let timezone = PoSQLTimeZone::Utc;
        let timeunit = PoSQLTimeUnit::Second;

        let timestamp_column_a = CommittableColumn::TimestampTZ(timeunit, timezone, &times[..2]);
        let timestamp_metadata_a = ColumnCommitmentMetadata::from_column(&timestamp_column_a);
        let timestamp_column_b = CommittableColumn::TimestampTZ(timeunit, timezone, &times);
        let timestamp_metadata_b = ColumnCommitmentMetadata::from_column(&timestamp_column_b);

        let b_difference_a = timestamp_metadata_b
            .try_difference(timestamp_metadata_a)
            .unwrap();
        assert_eq!(
            b_difference_a.column_type,
            ColumnType::TimestampTZ(timeunit, timezone)
        );
        if let ColumnBounds::TimestampTZ(Bounds::Bounded(bounds)) = b_difference_a.bounds {
            assert_eq!(bounds.min(), &1_625_065_000);
            assert_eq!(bounds.max(), &1_625_079_600);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let timestamp_column_empty = CommittableColumn::TimestampTZ(timeunit, timezone, &[]);
        let timestamp_metadata_empty =
            ColumnCommitmentMetadata::from_column(&timestamp_column_empty);

        assert_eq!(
            timestamp_metadata_b
                .try_difference(timestamp_metadata_empty)
                .unwrap(),
            timestamp_metadata_b
        );
        assert_eq!(
            timestamp_metadata_empty
                .try_difference(timestamp_metadata_b)
                .unwrap(),
            timestamp_metadata_empty
        );
    }

    #[test]
    fn we_can_difference_bigint_matching_metadata() {
        // Ordered case
        let ints = [1, 2, 3, 1, 0];
        let bigint_column_a = CommittableColumn::BigInt(&ints[..2]);
        let bigint_metadata_a = ColumnCommitmentMetadata::from_column(&bigint_column_a);
        let bigint_column_b = CommittableColumn::BigInt(&ints);
        let bigint_metadata_b = ColumnCommitmentMetadata::from_column(&bigint_column_b);

        let b_difference_a = bigint_metadata_b.try_difference(bigint_metadata_a).unwrap();
        assert_eq!(b_difference_a.column_type, ColumnType::BigInt);
        if let ColumnBounds::BigInt(Bounds::Bounded(bounds)) = b_difference_a.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let bigint_column_empty = CommittableColumn::BigInt(&[]);
        let bigint_metadata_empty = ColumnCommitmentMetadata::from_column(&bigint_column_empty);

        assert_eq!(
            bigint_metadata_b
                .try_difference(bigint_metadata_empty)
                .unwrap(),
            bigint_metadata_b
        );
        assert_eq!(
            bigint_metadata_empty
                .try_difference(bigint_metadata_b)
                .unwrap(),
            bigint_metadata_empty
        );
    }

    #[test]
    fn we_can_difference_smallint_matching_metadata() {
        // Ordered case
        let smallints = [1, 2, 3, 1, 0];
        let smallint_column_a = CommittableColumn::SmallInt(&smallints[..2]);
        let smallint_metadata_a = ColumnCommitmentMetadata::from_column(&smallint_column_a);
        let smallint_column_b = CommittableColumn::SmallInt(&smallints);
        let smallint_metadata_b = ColumnCommitmentMetadata::from_column(&smallint_column_b);

        let b_difference_a = smallint_metadata_b
            .try_difference(smallint_metadata_a)
            .unwrap();
        assert_eq!(b_difference_a.column_type, ColumnType::SmallInt);
        if let ColumnBounds::SmallInt(Bounds::Bounded(bounds)) = b_difference_a.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let smallint_column_empty = CommittableColumn::SmallInt(&[]);
        let smallint_metadata_empty = ColumnCommitmentMetadata::from_column(&smallint_column_empty);

        assert_eq!(
            smallint_metadata_b
                .try_difference(smallint_metadata_empty)
                .unwrap(),
            smallint_metadata_b
        );
        assert_eq!(
            smallint_metadata_empty
                .try_difference(smallint_metadata_b)
                .unwrap(),
            smallint_metadata_empty
        );
    }

    #[test]
    fn we_can_difference_int_matching_metadata() {
        // Ordered case
        let ints = [1, 2, 3, 1, 0];
        let int_column_a = CommittableColumn::Int(&ints[..2]);
        let int_metadata_a = ColumnCommitmentMetadata::from_column(&int_column_a);
        let int_column_b = CommittableColumn::Int(&ints);
        let int_metadata_b = ColumnCommitmentMetadata::from_column(&int_column_b);

        let b_difference_a = int_metadata_b.try_difference(int_metadata_a).unwrap();
        assert_eq!(b_difference_a.column_type, ColumnType::Int);
        if let ColumnBounds::Int(Bounds::Bounded(bounds)) = b_difference_a.bounds() {
            assert_eq!(bounds.min(), &0);
            assert_eq!(bounds.max(), &3);
        } else {
            panic!("difference of overlapping bounds should be Bounded");
        }

        let int_column_empty = CommittableColumn::Int(&[]);
        let int_metadata_empty = ColumnCommitmentMetadata::from_column(&int_column_empty);

        assert_eq!(
            int_metadata_b.try_difference(int_metadata_empty).unwrap(),
            int_metadata_b
        );
        assert_eq!(
            int_metadata_empty.try_difference(int_metadata_b).unwrap(),
            int_metadata_empty
        );
    }

    #[test]
    fn we_cannot_perform_arithmetic_on_mismatched_metadata() {
        let boolean_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Boolean,
            bounds: ColumnBounds::NoOrder,
        };
        let varchar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::VarChar,
            bounds: ColumnBounds::NoOrder,
        };
        let scalar_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Scalar,
            bounds: ColumnBounds::NoOrder,
        };
        let smallint_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::SmallInt,
            bounds: ColumnBounds::SmallInt(Bounds::Empty),
        };
        let int_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Int,
            bounds: ColumnBounds::Int(Bounds::Empty),
        };
        let bigint_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::BigInt,
            bounds: ColumnBounds::BigInt(Bounds::Empty),
        };
        let int128_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Int128,
            bounds: ColumnBounds::Int128(Bounds::Empty),
        };
        let decimal75_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Decimal75(Precision::new(4).unwrap(), 8),
            bounds: ColumnBounds::Int128(Bounds::Empty),
        };

        assert!(smallint_metadata.try_union(scalar_metadata).is_err());
        assert!(scalar_metadata.try_union(smallint_metadata).is_err());

        assert!(smallint_metadata.try_union(decimal75_metadata).is_err());
        assert!(decimal75_metadata.try_union(smallint_metadata).is_err());

        assert!(smallint_metadata.try_union(varchar_metadata).is_err());
        assert!(varchar_metadata.try_union(smallint_metadata).is_err());

        assert!(smallint_metadata.try_union(boolean_metadata).is_err());
        assert!(boolean_metadata.try_union(smallint_metadata).is_err());

        assert!(int_metadata.try_union(scalar_metadata).is_err());
        assert!(scalar_metadata.try_union(int_metadata).is_err());

        assert!(int_metadata.try_union(decimal75_metadata).is_err());
        assert!(decimal75_metadata.try_union(int_metadata).is_err());

        assert!(int_metadata.try_union(varchar_metadata).is_err());
        assert!(varchar_metadata.try_union(int_metadata).is_err());

        assert!(int_metadata.try_union(boolean_metadata).is_err());
        assert!(boolean_metadata.try_union(int_metadata).is_err());

        assert!(varchar_metadata.try_union(scalar_metadata).is_err());
        assert!(scalar_metadata.try_union(varchar_metadata).is_err());

        assert!(varchar_metadata.try_union(bigint_metadata).is_err());
        assert!(bigint_metadata.try_union(varchar_metadata).is_err());

        assert!(varchar_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(varchar_metadata).is_err());

        assert!(decimal75_metadata.try_union(scalar_metadata).is_err());
        assert!(scalar_metadata.try_union(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_union(bigint_metadata).is_err());
        assert!(bigint_metadata.try_union(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_union(varchar_metadata).is_err());
        assert!(varchar_metadata.try_union(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(decimal75_metadata).is_err());

        assert!(scalar_metadata.try_union(bigint_metadata).is_err());
        assert!(bigint_metadata.try_union(scalar_metadata).is_err());

        assert!(scalar_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(scalar_metadata).is_err());

        assert!(bigint_metadata.try_union(int128_metadata).is_err());
        assert!(int128_metadata.try_union(bigint_metadata).is_err());

        assert!(varchar_metadata.try_difference(scalar_metadata).is_err());
        assert!(scalar_metadata.try_difference(varchar_metadata).is_err());

        assert!(varchar_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(varchar_metadata).is_err());

        assert!(varchar_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(varchar_metadata).is_err());

        assert!(scalar_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(scalar_metadata).is_err());

        assert!(scalar_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(scalar_metadata).is_err());

        assert!(bigint_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(bigint_metadata).is_err());

        assert!(decimal75_metadata.try_difference(scalar_metadata).is_err());
        assert!(scalar_metadata.try_difference(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_difference(varchar_metadata).is_err());
        assert!(varchar_metadata.try_difference(decimal75_metadata).is_err());

        assert!(decimal75_metadata.try_difference(boolean_metadata).is_err());
        assert!(boolean_metadata.try_difference(decimal75_metadata).is_err());

        assert!(boolean_metadata.try_difference(bigint_metadata).is_err());
        assert!(bigint_metadata.try_difference(boolean_metadata).is_err());

        assert!(boolean_metadata.try_difference(int128_metadata).is_err());
        assert!(int128_metadata.try_difference(boolean_metadata).is_err());

        assert!(boolean_metadata.try_difference(varchar_metadata).is_err());
        assert!(varchar_metadata.try_difference(boolean_metadata).is_err());

        assert!(boolean_metadata.try_difference(scalar_metadata).is_err());
        assert!(scalar_metadata.try_difference(boolean_metadata).is_err());

        let different_decimal75_metadata = ColumnCommitmentMetadata {
            column_type: ColumnType::Decimal75(Precision::new(75).unwrap(), 0),
            bounds: ColumnBounds::Int128(Bounds::Empty),
        };

        assert!(decimal75_metadata
            .try_difference(different_decimal75_metadata)
            .is_err());
        assert!(different_decimal75_metadata
            .try_difference(decimal75_metadata)
            .is_err());

        assert!(decimal75_metadata
            .try_union(different_decimal75_metadata)
            .is_err());
        assert!(different_decimal75_metadata
            .try_union(decimal75_metadata)
            .is_err());

        let timestamp_tz_metadata_a = ColumnCommitmentMetadata {
            column_type: ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc),
            bounds: ColumnBounds::TimestampTZ(Bounds::Empty),
        };

        let timestamp_tz_metadata_b = ColumnCommitmentMetadata {
            column_type: ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::Utc),
            bounds: ColumnBounds::TimestampTZ(Bounds::Empty),
        };

        // Tests for union operations
        assert!(timestamp_tz_metadata_a.try_union(varchar_metadata).is_err());
        assert!(varchar_metadata.try_union(timestamp_tz_metadata_a).is_err());

        // Tests for difference operations
        assert!(timestamp_tz_metadata_a
            .try_difference(scalar_metadata)
            .is_err());
        assert!(scalar_metadata
            .try_difference(timestamp_tz_metadata_a)
            .is_err());

        // Tests for different time units within the same type
        assert!(timestamp_tz_metadata_a
            .try_union(timestamp_tz_metadata_b)
            .is_err());
        assert!(timestamp_tz_metadata_b
            .try_union(timestamp_tz_metadata_a)
            .is_err());

        // Difference with different time units
        assert!(timestamp_tz_metadata_a
            .try_difference(timestamp_tz_metadata_b)
            .is_err());
        assert!(timestamp_tz_metadata_b
            .try_difference(timestamp_tz_metadata_a)
            .is_err());
    }
}
