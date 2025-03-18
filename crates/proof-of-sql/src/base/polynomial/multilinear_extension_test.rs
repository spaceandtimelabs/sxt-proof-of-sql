use super::MultilinearExtension;
use crate::base::{
    database::Column,
    math::fixed_size_binary_width::FixedSizeBinaryWidth,
    scalar::{test_scalar::TestScalar, Scalar, ScalarExt},
};
use bumpalo::Bump;

#[test]
fn we_can_use_multilinear_extension_methods_for_fixed_size_binary_column() {
    let data = [1, 2, 3, 4, 5, 6, 7, 8];
    let col = Column::FixedSizeBinary(FixedSizeBinaryWidth::try_from(4).unwrap(), &data);

    let evaluation_vec = vec![101.into(), 102.into()];

    let row0_scalar = TestScalar::from_fixed_size_byte_slice(&data[0..4]);
    let row1_scalar = TestScalar::from_fixed_size_byte_slice(&data[4..8]);

    let expected_inner_product =
        row0_scalar * TestScalar::from(101) + row1_scalar * TestScalar::from(102);
    assert_eq!(col.inner_product(&evaluation_vec), expected_inner_product);

    let mut res = evaluation_vec.clone();
    let multiplier = TestScalar::from(10);

    col.mul_add(&mut res, &multiplier);
    assert_eq!(
        res,
        vec![
            TestScalar::from(101) + multiplier * row0_scalar,
            TestScalar::from(102) + multiplier * row1_scalar,
        ]
    );

    let sumcheck = col.to_sumcheck_term(2);
    assert_eq!(sumcheck.len(), 4);
    assert_eq!(sumcheck[0], row0_scalar);
    assert_eq!(sumcheck[1], row1_scalar);
    assert_eq!(sumcheck[2], TestScalar::ZERO);
    assert_eq!(sumcheck[3], TestScalar::ZERO);

    let (ptr, len) = col.id();
    assert_eq!(len, 2);
    assert_eq!(ptr, data.as_ptr().cast());
}

#[test]
fn allocated_slices_must_have_different_ids_even_when_one_is_empty() {
    let alloc = Bump::new();
    let foo = alloc.alloc_slice_fill_default(5) as &[TestScalar];
    let bar = alloc.alloc_slice_fill_default(0) as &[TestScalar];
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&foo),
        MultilinearExtension::<TestScalar>::id(&bar)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_slice() {
    let slice: &[i64] = &[2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
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
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
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
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_column() {
    let slice = Column::BigInt(&[2, 3, 4, 5, 6]);
    let evaluation_vec: Vec<TestScalar> =
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
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
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
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}

#[test]
fn we_can_use_multilinear_extension_methods_for_i64_vec() {
    let slice: &Vec<i64> = &vec![2, 3, 4, 5, 6];
    let evaluation_vec: Vec<TestScalar> =
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
        *MultilinearExtension::<TestScalar>::to_sumcheck_term(&slice, 3),
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
    assert_ne!(
        MultilinearExtension::<TestScalar>::id(&slice),
        MultilinearExtension::<TestScalar>::id(&&evaluation_vec)
    );
}
