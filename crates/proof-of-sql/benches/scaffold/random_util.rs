use bumpalo::Bump;
use proof_of_sql::base::{
    database::{Column, ColumnType},
    scalar::Scalar,
};
use proof_of_sql_parser::Identifier;
use rand::Rng;

pub type OptionalRandBound = Option<fn(usize) -> i64>;
pub fn generate_random_columns<'a, S: Scalar>(
    alloc: &'a Bump,
    rng: &mut impl Rng,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    num_rows: usize,
) -> Vec<(Identifier, Column<'a, S>)> {
    columns
        .iter()
        .map(|(id, ty, bound)| {
            (
                id.parse().unwrap(),
                match (ty, bound) {
                    (ColumnType::BigInt, None) => {
                        Column::BigInt(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::BigInt, Some(b)) => {
                        Column::BigInt(alloc.alloc_slice_fill_with(num_rows, |_| {
                            rng.gen_range(-b(num_rows)..=b(num_rows))
                        }))
                    }
                    (ColumnType::Boolean, _) => {
                        Column::Boolean(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::Int128, None) => {
                        Column::Int128(alloc.alloc_slice_fill_with(num_rows, |_| rng.gen()))
                    }
                    (ColumnType::Int128, Some(b)) => {
                        Column::Int128(alloc.alloc_slice_fill_with(num_rows, |_| {
                            rng.gen_range((-b(num_rows) as i128)..=(b(num_rows) as i128))
                        }))
                    }
                    (ColumnType::VarChar, _) => {
                        let strs = alloc.alloc_slice_fill_with(num_rows, |_| {
                            let len = rng
                                .gen_range(0..=bound.map(|b| b(num_rows) as usize).unwrap_or(10));
                            alloc.alloc_str(
                                String::from_iter(
                                    rng.sample_iter(&rand::distributions::Alphanumeric)
                                        .take(len)
                                        .map(char::from),
                                )
                                .as_str(),
                            ) as &str
                        });
                        Column::VarChar((
                            strs,
                            alloc.alloc_slice_fill_iter(strs.iter().map(|&s| Into::into(s))),
                        ))
                    }
                    _ => todo!(),
                },
            )
        })
        .collect()
}
