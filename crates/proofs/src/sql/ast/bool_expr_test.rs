use super::test_utility::*;
use crate::{
    base::database::{OwnedTableTestAccessor, TestAccessor},
    owned_table,
};
use bumpalo::Bump;

#[test]
fn we_can_compute_the_correct_result_of_a_complex_bool_expr_using_result_evaluate() {
    let data = owned_table!(
        "a" => [1_i64, 2, 3, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 7, 8, 9, 999],
        "b" => ["g", "g", "t", "ghi", "g", "g", "jj", "f", "g", "g", "gar", "qwe", "g", "g", "poi", "zxc", "999"],
        "c" => [3_i128, 123, 3, 234, 3, 345, 3, 456, 3, 567, 3, 678, 3, 789, 3, 890, 999],
    );
    let mut accessor = OwnedTableTestAccessor::new_empty();
    let t = "sxt.t".parse().unwrap();
    accessor.add_table(t, data, 0);
    // (a <= 5 || b == "g") && c != 3
    let bool_expr = and(
        or(lte(t, "a", 5, &accessor), equal(t, "b", "g", &accessor)),
        not(equal(t, "c", 3, &accessor)),
    );
    let alloc = Bump::new();
    let res = bool_expr.result_evaluate(17, &alloc, &accessor);
    let expected_res = &[
        false, true, false, true, false, true, false, true, false, true, false, true, false, true,
        false, false, false,
    ];
    assert_eq!(res, expected_res);
}
