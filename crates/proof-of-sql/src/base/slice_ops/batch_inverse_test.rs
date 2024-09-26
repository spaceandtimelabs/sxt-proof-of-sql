use crate::base::{scalar::Curve25519Scalar, slice_ops};
use num_traits::{Inv, Zero};

#[cfg_attr(test, allow(clippy::missing_panics_doc))]
#[test]
fn we_can_pseudo_invert_empty_arrays() {
    let input: Vec<Curve25519Scalar> = Vec::new();
    let mut res = Vec::new();
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_1_with_non_zero() {
    let input = [Curve25519Scalar::from(2_u32)];
    let mut res = vec![Curve25519Scalar::from(0_u32)];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
    assert!(res == vec![input[0].inv().unwrap()]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_1_with_zero() {
    let input = [Curve25519Scalar::from(0_u32)];
    let mut res = vec![Curve25519Scalar::from(0_u32)];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);
    assert!(res == vec![input[0]]);
}

#[test]
fn we_can_pseudo_invert_arrays_of_length_bigger_than_1_with_zeros_and_non_zeros() {
    let input = vec![
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(2_u32),
        (-33_i32).into(),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(45_u32),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(47_u32),
    ];
    let mut res = vec![Curve25519Scalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val != Curve25519Scalar::zero() {
            assert!(input_val.inv().unwrap() == res_val);
        } else {
            assert!(Curve25519Scalar::zero() == res_val);
        }
    }
}

#[test]
fn we_can_pseudo_invert_arrays_with_nonzero_count_bigger_than_min_chunking_size_with_zeros_and_non_zeros(
) {
    let input: Vec<_> = vec![
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(2_u32),
        (-33_i32).into(),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(45_u32),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(47_u32),
    ]
    .into_iter()
    .cycle()
    .take(slice_ops::MIN_RAYON_LEN * 10)
    .collect();

    let mut res = vec![Curve25519Scalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val != Curve25519Scalar::zero() {
            assert!(input_val.inv().unwrap() == res_val);
        } else {
            assert!(Curve25519Scalar::zero() == res_val);
        }
    }
}

#[test]
fn we_can_pseudo_invert_arrays_with_nonzero_count_smaller_than_min_chunking_size_with_zeros_and_non_zeros(
) {
    let input: Vec<_> = vec![
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(2_u32),
        (-33_i32).into(),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(45_u32),
        Curve25519Scalar::from(0_u32),
        Curve25519Scalar::from(47_u32),
    ]
    .into_iter()
    .cycle()
    .take(slice_ops::MIN_RAYON_LEN - 1)
    .collect();

    let mut res = vec![Curve25519Scalar::from(0_u32); input.len()];
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(&input[..]);
    slice_ops::batch_inversion(&mut res[..]);

    for (input_val, res_val) in input.iter().zip(res) {
        if *input_val != Curve25519Scalar::zero() {
            assert!(input_val.inv().unwrap() == res_val);
        } else {
            assert!(Curve25519Scalar::zero() == res_val);
        }
    }
}
