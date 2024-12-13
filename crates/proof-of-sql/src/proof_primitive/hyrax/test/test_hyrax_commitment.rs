use super::test_hyrax_configuration::TestHyraxConfiguration;
use crate::{
    base::{
        commitment::ColumnCommitments,
        database::{
            owned_table_utility::{bigint, owned_table, varchar},
            OwnedTable,
        },
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::hyrax::{
        base::hyrax_commitment::HyraxCommitment,
        test::test_hyrax_public_setup::TestHyraxPublicSetup,
    },
};
use core::iter;
use curve25519_dalek::RistrettoPoint;
use proof_of_sql_parser::Identifier;
use rand::SeedableRng;

pub type TestHyraxCommitment = HyraxCommitment<TestHyraxConfiguration>;

#[test]
fn we_can_append_rows_to_column_commitments() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators = iter::repeat_with(|| RistrettoPoint::random(&mut rng))
        .take(1000)
        .collect::<Vec<_>>();
    let public_setup = TestHyraxPublicSetup {
        generators: &generators,
    };
    let bigint_id: Identifier = "bigint_column".parse().unwrap();
    let bigint_data = [1i64, 5, -5, 0, 10];

    let varchar_id: Identifier = "varchar_column".parse().unwrap();
    let varchar_data = ["Lorem", "ipsum", "dolor", "sit", "amet"];

    let initial_columns: OwnedTable<TestScalar> = owned_table([
        bigint(bigint_id, bigint_data[..2].to_vec()),
        varchar(varchar_id, varchar_data[..2].to_vec()),
    ]);

    let mut column_commitments =
        ColumnCommitments::<TestHyraxCommitment>::try_from_columns_with_offset(
            initial_columns.inner_table(),
            0,
            &public_setup,
        )
        .unwrap();

    let append_columns: OwnedTable<TestScalar> = owned_table([
        bigint(bigint_id, bigint_data[2..].to_vec()),
        varchar(varchar_id, varchar_data[2..].to_vec()),
    ]);

    column_commitments
        .try_append_rows_with_offset(append_columns.inner_table(), 2, &public_setup)
        .unwrap();

    let total_columns: OwnedTable<TestScalar> = owned_table([
        bigint(bigint_id, bigint_data),
        varchar(varchar_id, varchar_data),
    ]);

    let expected_column_commitments = ColumnCommitments::try_from_columns_with_offset(
        total_columns.inner_table(),
        0,
        &public_setup,
    )
    .unwrap();

    assert_eq!(column_commitments, expected_column_commitments);
}
