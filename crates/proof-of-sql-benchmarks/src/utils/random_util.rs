use bumpalo::Bump;
use indexmap::{indexmap, IndexMap};
use proof_of_sql::base::{
    database::{
        table_utility::{borrowed_decimal75, borrowed_timestamptz, borrowed_varchar, table},
        Column, ColumnType, Table, TableRef,
    },
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
};
use rand::Rng;
use sqlparser::ast::Ident;

pub type OptionalRandBound = Option<fn(usize) -> i64>;

/// # Panics
///
/// Will panic if:
/// - An unsupported `ColumnType` is encountered, triggering a panic in the `todo!()` macro.
#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
pub fn generate_random_columns<'a, S: Scalar>(
    alloc: &'a Bump,
    rng: &mut impl Rng,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    num_rows: usize,
) -> Vec<(Ident, Column<'a, S>)> {
    columns
        .iter()
        .map(|(id, ty, bound)| {
            (
                Ident::new(*id),
                match (ty, bound) {
                    (ColumnType::Boolean, _) => {
                        Column::Boolean(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::TinyInt, None) => {
                        Column::TinyInt(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::TinyInt, Some(b)) => {
                        Column::TinyInt(alloc.alloc_slice_fill_with(num_rows, |_| {
                            let b_value = b(num_rows);
                            let clamped_b_value =
                                b_value.clamp(i64::from(i8::MIN), i64::from(i8::MAX)) as i8;
                            rng.gen_range(-clamped_b_value..=clamped_b_value)
                        }))
                    }
                    (ColumnType::SmallInt, None) => {
                        Column::SmallInt(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::SmallInt, Some(b)) => {
                        Column::SmallInt(alloc.alloc_slice_fill_with(num_rows, |_| {
                            let b_value = b(num_rows);
                            let clamped_b_value =
                                b_value.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16;
                            rng.gen_range(-clamped_b_value..=clamped_b_value)
                        }))
                    }
                    (ColumnType::Int, None) => {
                        Column::Int(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::Int, Some(b)) => {
                        Column::Int(alloc.alloc_slice_fill_with(num_rows, |_| {
                            let b_value = b(num_rows);
                            let clamped_b_value =
                                b_value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                            rng.gen_range(-clamped_b_value..=clamped_b_value)
                        }))
                    }
                    (ColumnType::BigInt, None) => {
                        Column::BigInt(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::BigInt, Some(b)) => {
                        Column::BigInt(alloc.alloc_slice_fill_with(num_rows, |_| {
                            rng.gen_range(-b(num_rows)..=b(num_rows))
                        }))
                    }
                    (ColumnType::Int128, None) => {
                        Column::Int128(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::Int128, Some(b)) => {
                        Column::Int128(alloc.alloc_slice_fill_with(num_rows, |_| {
                            rng.gen_range((i128::from(-b(num_rows)))..=(i128::from(b(num_rows))))
                        }))
                    }
                    (ColumnType::VarChar, _) => {
                        let strs = alloc.alloc_slice_fill_with(num_rows, |_| {
                            let len = rng
                                .gen_range(0..=bound.map(|b| b(num_rows) as usize).unwrap_or(10));
                            alloc.alloc_str(
                                &rng.sample_iter(&rand::distributions::Alphanumeric)
                                    .take(len)
                                    .map(char::from)
                                    .collect::<String>(),
                            ) as &str
                        });
                        Column::VarChar((
                            strs,
                            alloc.alloc_slice_fill_iter(strs.iter().map(|&s| Into::into(s))),
                        ))
                    }
                    (ColumnType::Scalar, _) => {
                        let strs = alloc.alloc_slice_fill_with(num_rows, |_| {
                            let len = rng
                                .gen_range(0..=bound.map(|b| b(num_rows) as usize).unwrap_or(10));
                            alloc.alloc_str(
                                &rng.sample_iter(&rand::distributions::Alphanumeric)
                                    .take(len)
                                    .map(char::from)
                                    .collect::<String>(),
                            ) as &str
                        });
                        Column::Scalar(
                            alloc.alloc_slice_fill_iter(strs.iter().map(|&s| Into::into(s))),
                        )
                    }
                    (ColumnType::Decimal75(_, _), _) => {
                        let strs = alloc.alloc_slice_fill_with(num_rows, |_| {
                            let len = rng
                                .gen_range(0..=bound.map(|b| b(num_rows) as usize).unwrap_or(10));
                            alloc.alloc_str(
                                &rng.sample_iter(&rand::distributions::Alphanumeric)
                                    .take(len)
                                    .map(char::from)
                                    .collect::<String>(),
                            ) as &str
                        });
                        Column::Decimal75(
                            Precision::new(20).unwrap(), //Precision::new(rng.gen_range(1..75)).unwrap(),
                            0,                           //rng.gen::<i8>(),
                            alloc.alloc_slice_fill_iter(strs.iter().map(|&s| Into::into(s))),
                        )
                    }
                    (ColumnType::TimestampTZ(_, _), None) => Column::TimestampTZ(
                        PoSQLTimeUnit::Second,
                        PoSQLTimeZone::utc(),
                        alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()),
                    ),
                    (ColumnType::TimestampTZ(_, _), Some(b)) => Column::TimestampTZ(
                        PoSQLTimeUnit::Second,
                        PoSQLTimeZone::utc(),
                        alloc.alloc_slice_fill_with(num_rows, |_| {
                            rng.gen_range(-b(num_rows)..=b(num_rows))
                        }),
                    ),
                    _ => todo!(),
                },
            )
        })
        .collect()
}

pub fn generate_non_random_table<S: Scalar>(alloc: &Bump) -> IndexMap<TableRef, Table<S>> {
    let data = ["0x1", "0x2", "0x3", "0x2", "0x1"];
    indexmap! {
        TableRef::from_names(None, "transactions") => table(
            vec![
                borrowed_varchar("from_address", data, alloc),
                borrowed_varchar("to_address", ["0x2", "0x3", "0x1", "0x3", "0x2"], alloc),
                borrowed_decimal75("value", 20, 0, [100, 200, 300, 400, 500], alloc),
                borrowed_timestamptz("timestamp", PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), [1, 2, 3, 4, 4], alloc),
            ]
        )
    }
}

/// Generates a random table with the specified name and columns
pub fn generate_random_table<'a, S: Scalar>(
    table_name: &str,
    alloc: &'a Bump,
    rng: &'a mut impl Rng,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    num_rows: usize,
) -> IndexMap<TableRef, Table<'a, S>> {
    indexmap! {
        TableRef::from_names(None, table_name) => table(
            generate_random_columns(alloc, rng, columns, num_rows)
        )
    }
}
