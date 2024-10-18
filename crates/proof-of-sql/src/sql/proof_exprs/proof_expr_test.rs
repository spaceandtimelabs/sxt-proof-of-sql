use super::{test_utility::*, DynProofExpr, ProofExpr};
use crate::base::{
    commitment::InnerProductProof,
    database::{
        owned_table_utility::*, Column, ColumnNullability, OwnedTableTestAccessor, TestAccessor,
    },
};
use bumpalo::Bump;
use curve25519_dalek::RistrettoPoint;

#[test]
fn we_can_compute_the_correct_result_of_a_complex_bool_expr_using_result_evaluate() {
    let data = owned_table([
        bigint("a", [1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 7, 8, 9, 999]),
        varchar(
            "b",
            [
                "g", "g", "t", "ghi", "g", "g", "jj", "f", "g", "g", "gar", "qwe", "g", "g", "poi",
                "zxc", "999",
            ],
        ),
        int128(
            "c",
            [
                3, 123, 3, 234, 3, 345, 3, 456, 3, 567, 3, 678, 3, 789, 3, 890, 999,
            ],
        ),
    ]);
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    // (a <= 5 || b == "g") && c != 3
    let bool_expr: DynProofExpr<RistrettoPoint> = and(
        or(
            lte(column(t, "a", &accessor), const_bigint(5)),
            equal(column(t, "b", &accessor), const_varchar("g")),
        ),
        not(equal(column(t, "c", &accessor), const_int128(3))),
    );
    let alloc = Bump::new();
    let res = bool_expr.result_evaluate(17, &alloc, &accessor);
    let expected_res = Column::Boolean(
        ColumnNullability::NotNullable,
        &[
            false, true, false, true, false, true, false, true, false, true, false, true, false,
            true, false, false, false,
        ],
    );
    assert_eq!(res, expected_res);
}
