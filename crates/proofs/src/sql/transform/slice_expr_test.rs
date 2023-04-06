use crate::record_batch;
use crate::sql::proof::TransformExpr;
use crate::sql::transform::test_utility::{composite_result, slice};

#[test]
fn we_can_slice_a_lazy_frame_using_only_a_positive_limit_value() {
    let limit = 3_usize;

    let data_a = vec![123, 342, -234, 777, 123, 34];
    let data_d = vec!["alfa", "beta", "abc", "f", "kl", "f"];
    let data_frame = record_batch!(
        "a" => data_a.to_vec(),
        "d" => data_d.to_vec()
    );

    let result_expr = composite_result(slice(limit as u64, 0));
    let data_frame = result_expr.transform_results(data_frame);

    assert_eq!(
        data_frame,
        record_batch!(
            "a" => data_a[0..limit].to_vec(),
            "d" => data_d[0..limit].to_vec()
        )
    );
}

#[test]
fn we_can_slice_a_lazy_frame_using_only_a_zero_limit_value() {
    let limit = 0;

    let data_a = vec![123, 342, -234, 777, 123, 34];
    let data_d = vec!["alfa", "beta", "abc", "f", "kl", "f"];
    let data_frame = record_batch!(
        "a" => data_a.to_vec(),
        "d" => data_d.to_vec()
    );

    let result_expr = composite_result(slice(limit as u64, 0));
    let data_frame = result_expr.transform_results(data_frame);

    assert_eq!(
        data_frame,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "d" => Vec::<String>::new()
        )
    );
}

#[test]
fn we_can_slice_a_lazy_frame_using_only_a_positive_offset_value() {
    let offset = 3;

    let data_a = vec![123, 342, -234, 777, 123, 34];
    let data_d = vec!["alfa", "beta", "abc", "f", "kl", "f"];
    let data_frame = record_batch!(
        "a" => data_a.to_vec(),
        "d" => data_d.to_vec()
    );

    let result_expr = composite_result(slice(u64::MAX, offset));
    let data_frame = result_expr.transform_results(data_frame);

    assert_eq!(
        data_frame,
        record_batch!(
            "a" => data_a[(offset as usize)..].to_vec(),
            "d" => data_d[(offset as usize)..].to_vec()
        )
    );
}

#[test]
fn we_can_slice_a_lazy_frame_using_only_a_negative_offset_value() {
    let offset = -2;

    let data_a = vec![123, 342, -234, 777, 123, 34];
    let data_d = vec!["alfa", "beta", "abc", "f", "kl", "f"];
    let data_frame = record_batch!(
        "a" => data_a.to_vec(),
        "d" => data_d.to_vec()
    );

    let result_expr = composite_result(slice(u64::MAX, offset));
    let data_frame = result_expr.transform_results(data_frame);

    assert_eq!(
        data_frame,
        record_batch!(
            "a" => data_a[(data_a.len() as i64 + offset) as usize..].to_vec(),
            "d" => data_d[(data_a.len() as i64 + offset) as usize..].to_vec()
        )
    );
}

#[test]
fn we_can_slice_a_lazy_frame_using_both_limit_and_offset_values() {
    let offset = -2;
    let limit = 1_usize;

    let data_a = vec![123, 342, -234, 777, 123, 34];
    let data_d = vec!["alfa", "beta", "abc", "f", "kl", "f"];
    let data_frame = record_batch!(
        "a" => data_a.to_vec(),
        "d" => data_d.to_vec()
    );

    let result_expr = composite_result(slice(limit as u64, offset));
    let data_frame = result_expr.transform_results(data_frame);
    let beg_expected_index = (data_a.len() as i64 + offset) as usize;

    assert_eq!(
        data_frame,
        record_batch!(
            "a" => data_a[beg_expected_index..(beg_expected_index + limit)].to_vec(),
            "d" => data_d[beg_expected_index..(beg_expected_index + limit)].to_vec()
        )
    );
}
