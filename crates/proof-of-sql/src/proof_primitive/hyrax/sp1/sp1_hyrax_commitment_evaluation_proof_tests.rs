use crate::{
    base::{
        commitment::{QueryCommitments, QueryCommitmentsExt},
        database::{
            owned_table_utility::{bigint, boolean, owned_table},
            ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, TestAccessor,
        },
        scalar::Curve25519Scalar,
    },
    proof_primitive::hyrax::{
        base::{hyrax_public_setup::HyraxPublicSetup, hyrax_scalar::HyraxScalarWrapper},
        sp1::sp1_hyrax_commitment_evaluation_proof::Sp1HyraxCommitmentEvaluationProof,
    },
    sql::{parse::QueryExpr, proof::QueryProof},
};
use alloc::vec::Vec;
use core::iter;
use curve25519_dalek::{EdwardsPoint, Scalar};
use proof_of_sql_parser::Identifier;
use rand::{random, thread_rng, Rng, RngCore, SeedableRng};

#[test]
fn we_can_verify_hyrax_proof() {
    let nu = 7;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators: Vec<EdwardsPoint> =
        iter::repeat_with(|| EdwardsPoint::mul_base(&Scalar::from(rng.next_u64())))
            .take(1 << nu)
            .collect();
    let setup = HyraxPublicSetup {
        generators: &generators,
    };
    let mut accessor =
        OwnedTableTestAccessor::<Sp1HyraxCommitmentEvaluationProof>::new_empty_with_setup(setup);
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([boolean("a", [true, false, false, true, false, true])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE a;".parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<Sp1HyraxCommitmentEvaluationProof>::new(query.proof_expr(), &accessor, &setup);
    let owned_table_result = proof
        .verify(query.proof_expr(), &accessor, &serialized_result, &setup)
        .unwrap()
        .table;
    let expected_result = owned_table([boolean("a", [true, true, true])]);
    assert_eq!(owned_table_result, expected_result);
}

#[test]
fn reproduce_normalization_error() {
    let nu = 7;
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators: Vec<EdwardsPoint> =
        iter::repeat_with(|| EdwardsPoint::mul_base(&Scalar::from(rng.next_u64())))
            .take(1 << nu)
            .collect();
    let setup = HyraxPublicSetup {
        generators: &generators,
    };
    let mut accessor =
        OwnedTableTestAccessor::<Sp1HyraxCommitmentEvaluationProof>::new_empty_with_setup(setup);
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        generate_table(2_usize.pow(nu * 2 - 1)),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT name, COUNT(*) FROM table WHERE is_valid GROUP BY name"
            .parse()
            .unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<Sp1HyraxCommitmentEvaluationProof>::new(query.proof_expr(), &accessor, &setup);
    let query_commitments = QueryCommitments::from_accessor_with_max_bounds(
        vec![
            ColumnRef::new(
                "sxt.table".parse().unwrap(),
                Identifier::try_new("is_valid").unwrap(),
                ColumnType::Boolean,
            ),
            ColumnRef::new(
                "sxt.table".parse().unwrap(),
                Identifier::try_new("name").unwrap(),
                ColumnType::BigInt,
            ),
            ColumnRef::new(
                "sxt.table".parse().unwrap(),
                Identifier::try_new("value").unwrap(),
                ColumnType::BigInt,
            ),
        ],
        &accessor,
    );
    let _result = proof.verify(
        query.proof_expr(),
        &query_commitments,
        &serialized_result,
        &setup,
    );
}

fn generate_table(length: usize) -> OwnedTable<HyraxScalarWrapper<Curve25519Scalar>> {
    let mut rng = thread_rng();
    let number_of_names = 10;
    let name_choices: Vec<i64> = iter::repeat_with(random::<i64>)
        .take(number_of_names)
        .collect();
    owned_table([
        boolean("is_valid", iter::repeat_with(random::<bool>).take(length)),
        bigint("value", iter::repeat_with(random::<i64>).take(length)),
        bigint(
            "name",
            iter::repeat_with(|| name_choices[rng.gen_range(0..number_of_names)]).take(length),
        ),
    ])
}
