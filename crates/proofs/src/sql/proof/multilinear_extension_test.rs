use super::MultilinearExtension;
use crate::base::{database::Column, scalar::ArkScalar};

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_slice() {
    let slice: &[i64] = &[2, 3, 4, 5, 6];
    let evaluation_vec: Vec<ArkScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *slice.to_sumcheck_term(3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(slice.id(), (&evaluation_vec).id());
}

#[test]
fn we_can_use_multilinear_extension_methods_for_column() {
    let slice = Column::BigInt(&[2, 3, 4, 5, 6]);
    let evaluation_vec: Vec<ArkScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *slice.to_sumcheck_term(3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(slice.id(), (&evaluation_vec).id());
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_vec() {
    let slice: &Vec<i64> = &vec![2, 3, 4, 5, 6];
    let evaluation_vec: Vec<ArkScalar> =
        vec![101.into(), 102.into(), 103.into(), 104.into(), 105.into()];
    assert_eq!(
        slice.inner_product(&evaluation_vec),
        (2 * 101 + 3 * 102 + 4 * 103 + 5 * 104 + 6 * 105).into()
    );
    let mut res = evaluation_vec.clone();
    slice.mul_add(&mut res, &10.into());
    assert_eq!(
        res,
        vec![121.into(), 132.into(), 143.into(), 154.into(), 165.into()]
    );
    assert_eq!(
        *slice.to_sumcheck_term(3),
        vec![
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            0.into(),
            0.into(),
            0.into()
        ]
    );
    assert_ne!(slice.id(), (&evaluation_vec).id());
}
