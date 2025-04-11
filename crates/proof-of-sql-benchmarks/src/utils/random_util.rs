use bumpalo::Bump;
use proof_of_sql::base::{
    database::{Column, ColumnType},
    scalar::Scalar,
};
use rand::Rng;
use sqlparser::ast::Ident;

pub type OptionalRandBound = Option<fn(usize) -> i64>;
/// # Panics
///
/// Will panic if:
/// - An unsupported `ColumnType` is encountered, triggering a panic in the `todo!()` macro.
#[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
                    _ => todo!(),
                },
            )
        })
        .collect()
}
