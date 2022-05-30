use crate::base::math::log::*;

#[test]
fn test_log2() {
    assert_eq!(log2_down(1), 0);
    assert_eq!(log2_down(2), 1);
    assert_eq!(log2_down(3), 1);
    assert_eq!(log2_down(4), 2);

    assert_eq!(log2_up(1), 0);
    assert_eq!(log2_up(2), 1);
    assert_eq!(log2_up(3), 2);
    assert_eq!(log2_up(4), 2);
}
