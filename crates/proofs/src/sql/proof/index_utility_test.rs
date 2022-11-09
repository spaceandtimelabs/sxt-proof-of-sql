use super::are_indexes_valid;

#[test]
fn an_empty_index_slice_is_always_valid() {
    let ix = [];
    assert!(are_indexes_valid(&ix, 0));
    assert!(are_indexes_valid(&ix, 1));
}

#[test]
fn a_single_index_is_valid_if_within_range() {
    let ix = [0];
    assert!(!are_indexes_valid(&ix, 0));
    assert!(are_indexes_valid(&ix, 1));
}

#[test]
fn multiple_indexes_are_valid_if_sorted_and_within_range() {
    let ix = [0, 1];
    assert!(are_indexes_valid(&ix, 2));
    assert!(!are_indexes_valid(&ix, 1));

    let ix = [1, 0];
    assert!(!are_indexes_valid(&ix, 2));

    let ix = [0, 2, 3, 7];
    assert!(are_indexes_valid(&ix, 8));
    assert!(!are_indexes_valid(&ix, 7));

    let ix = [0, 3, 2, 7];
    assert!(!are_indexes_valid(&ix, 8));
}

#[test]
fn repeated_indexes_are_invalid() {
    let ix = [0, 1, 1];
    assert!(!are_indexes_valid(&ix, 2));
}
