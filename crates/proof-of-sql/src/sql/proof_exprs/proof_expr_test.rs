use super::{test_utility::*, DynProofExpr, ProofExpr};
use crate::base::{
    commitment::InnerProductProof,
    database::{table_utility::*, Column, NullableColumn, TableRef, TableTestAccessor, TestAccessor},
};
use bumpalo::Bump;

#[test]
fn we_can_compute_the_correct_result_of_a_complex_bool_expr_using_result_evaluate() {
    let alloc = Bump::new();
    let data = table([
        borrowed_bigint(
            "a",
            [1, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 7, 8, 9, 999],
            &alloc,
        ),
        borrowed_varchar(
            "b",
            [
                "g", "g", "t", "ghi", "g", "g", "jj", "f", "g", "g", "gar", "qwe", "g", "g", "poi",
                "zxc", "999",
            ],
            &alloc,
        ),
        borrowed_int128(
            "c",
            [
                3, 123, 3, 234, 3, 345, 3, 456, 3, 567, 3, 678, 3, 789, 3, 890, 999,
            ],
            &alloc,
        ),
    ]);
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let t = TableRef::new("sxt", "t");
    accessor.add_table(t.clone(), data.clone(), 0);
    // (a <= 5 || b == "g") && c != 3
    let bool_expr: DynProofExpr = and(
        or(
            lte(column(&t, "a", &accessor), const_bigint(5)),
            equal(column(&t, "b", &accessor), const_varchar("g")),
        ),
        not(equal(column(&t, "c", &accessor), const_int128(3))),
    );
    let res = bool_expr.result_evaluate(&alloc, &data);
    let expected_res = NullableColumn::new(Column::Boolean(&[
        false, true, false, true, false, true, false, true, false, true, false, true, false, true,
        false, false, false,
    ]));
    assert_eq!(res, expected_res);
}
