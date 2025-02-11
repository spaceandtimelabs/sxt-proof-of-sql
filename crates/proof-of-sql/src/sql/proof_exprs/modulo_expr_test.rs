use crate::{
    base::{
        commitment::InnerProductProof,
        database::{owned_table_utility::*, OwnedTableTestAccessor},
    },
    sql::{
        proof::VerifiableQueryResult, proof_exprs::test_utility::*, proof_plans::test_utility::*,
    },
};

fn verify_tinyint_modulo(a: &[i8], b: &[i8], q: &[i8]) {
    let data = owned_table([
        tinyint("a", a.iter().copied()),
        tinyint("b", b.iter().copied()),
    ]);
    let t: crate::base::database::TableRef = "sxt.t".parse().unwrap();
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![aliased_plan(
            modulo(column(&t, "a", &accessor), column(&t, "b", &accessor)),
            "q",
        )],
        tab(&t),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([tinyint("q", q.iter().copied())]);
    assert_eq!(res, expected_res);
}

fn verify_int128_modulo(a: &[i128], b: &[i128], r: &[i128]) {
    let data = owned_table([
        int128("a", a.iter().copied()),
        int128("b", b.iter().copied()),
    ]);
    let t: crate::base::database::TableRef = "sxt.t".parse().unwrap();
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());
    let ast = projection(
        vec![aliased_plan(
            modulo(column(&t, "a", &accessor), column(&t, "b", &accessor)),
            "r",
        )],
        tab(&t),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([int128("r", r.iter().copied())]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_int_modulus_query() {
    let data = owned_table([
        tinyint("b", [2_i8, -115, 6, 126]),
        smallint("c", [7_i16, 36, -30000, 31104]),
        int("d", [4_i32, -115, i32::MIN + 12, 52]),
        bigint("e", [i64::MIN + 366, -68, i64::MAX, 126]),
        int128("f", [6_i128, i128::MIN + 3, 99, i128::MAX - 6]),
    ]);
    let t: crate::base::database::TableRef = "sxt.t".parse().unwrap();
    let accessor =
        OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t.clone(), data, 0, ());

    let ast = projection(
        vec![
            aliased_plan(modulo(column(&t, "b", &accessor), const_tinyint(2)), "b2"),
            aliased_plan(
                modulo(column(&t, "b", &accessor), column(&t, "c", &accessor)),
                "bc",
            ),
            aliased_plan(
                modulo(column(&t, "b", &accessor), column(&t, "d", &accessor)),
                "bd",
            ),
            aliased_plan(
                modulo(column(&t, "b", &accessor), column(&t, "e", &accessor)),
                "be",
            ),
            aliased_plan(
                modulo(column(&t, "b", &accessor), column(&t, "f", &accessor)),
                "bf",
            ),
            aliased_plan(modulo(column(&t, "c", &accessor), const_smallint(2)), "c2"),
            aliased_plan(
                modulo(column(&t, "c", &accessor), column(&t, "b", &accessor)),
                "cb",
            ),
            aliased_plan(
                modulo(column(&t, "c", &accessor), column(&t, "d", &accessor)),
                "cd",
            ),
            aliased_plan(
                modulo(column(&t, "c", &accessor), column(&t, "e", &accessor)),
                "ce",
            ),
            aliased_plan(
                modulo(column(&t, "c", &accessor), column(&t, "f", &accessor)),
                "cf",
            ),
        ],
        tab(&t),
    );
    let verifiable_res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&ast, &accessor, &());
    let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
    let expected_res = owned_table([
        tinyint("b2", [0_i8, -1, 0, 0]),
        smallint("bc", [2_i16, -7, 6, 126]),
        int("bd", [2_i32, 0, 6, 22]),
        bigint("be", [2_i64, -47, 6, 0]),
        int128("bf", [2_i128, -115, 6, 126]),
        smallint("c2", [1_i16, 0, 0, 0]),
        smallint("cb", [1_i16, 36, 0, 108]),
        int("cd", [3_i32, 36, -30000, 8]),
        bigint("ce", [7_i64, 36, -30000, 108]),
        int128("cf", [1_i128, 36, -3, 31104]),
    ]);
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_verify_nonnegative_only_modulus() {
    verify_tinyint_modulo(&[2, 7, 0, 54], &[1, 33, 6, 36], &[0, 7, 0, 18]);
}

#[test]
fn we_can_verify_nonpositive_only_modulus() {
    verify_tinyint_modulo(&[-2, -7, 0, -54], &[-1, -33, -6, -36], &[0, -7, 0, -18]);
}

#[test]
fn we_can_verify_nonpositive_numerator_and_positive_denominator_modulus() {
    verify_tinyint_modulo(&[-2, -7, 0, -54], &[1, 33, 6, 36], &[0, -7, 0, -18]);
}

#[test]
fn we_can_verify_nonnegative_numerator_and_negative_denominator_modulus() {
    verify_tinyint_modulo(&[2, 7, 0, 54], &[-1, -33, -6, -36], &[0, 7, 0, 18]);
}

#[test]
fn we_can_verify_zero_denominator_modulus() {
    verify_tinyint_modulo(
        &[1, -1, 0, i8::MAX, i8::MIN],
        &[0, 0, 0, 0, 0],
        &[1, -1, 0, i8::MAX, i8::MIN],
    );
}

#[test]
fn we_can_verify_minmax_numerator_and_plusminusonezero_denominator_modulus() {
    verify_tinyint_modulo(
        &[i8::MAX, i8::MIN, i8::MAX, i8::MIN, i8::MAX, i8::MIN],
        &[1, 1, -1, -1, 0, 0],
        &[0, 0, 0, 0, i8::MAX, i8::MIN],
    );
}

#[test]
fn we_can_verify_minmax_numerator_and_plusminusonezero_denominator_modulus_i128() {
    verify_int128_modulo(
        &[
            i128::MAX,
            i128::MIN,
            i128::MAX,
            i128::MIN,
            i128::MAX,
            i128::MIN,
        ],
        &[1, 1, -1, -1, 0, 0],
        &[0, 0, 0, 0, i128::MAX, i128::MIN],
    );
}

#[test]
fn we_can_verify_large_remainder_and_rhs() {
    let floor_of_sqrt: i128 = 13_043_817_825_332_782_212;
    verify_int128_modulo(
        &[i128::MAX, i128::MIN, i128::MAX, i128::MIN],
        // Floor of sqrt{i128::MAX}
        &[floor_of_sqrt, floor_of_sqrt, -floor_of_sqrt, -floor_of_sqrt],
        &[
            9_119_501_915_260_492_783,
            -9_119_501_915_260_492_784,
            9_119_501_915_260_492_783,
            -9_119_501_915_260_492_784,
        ],
    );
}
