use ark_ff::fields::MontConfig;
use ark_ff::PrimeField;
use ark_ff::{BigInt, BigInteger};
use bumpalo::Bump;
use byte_slice_cast::AsMutByteSlice;
use curve25519_dalek::scalar::Scalar;

#[cfg(test)]
pub type ArkScalarConfig = ark_curve25519::FrConfig;
#[cfg(not(test))]
type ArkScalarConfig = ark_curve25519::FrConfig;
pub type ArkScalar = ark_curve25519::Fr;

pub fn to_ark_scalar(x: &Scalar) -> ArkScalar {
    let mut values: [u64; 4] = [0; 4];
    values.as_mut_byte_slice().clone_from_slice(x.as_bytes());
    ArkScalarConfig::from_bigint(BigInt::new(values)).unwrap()
}

pub fn to_ark_scalars(xsp: &mut [ArkScalar], xs: &[Scalar]) {
    assert_eq!(xsp.len(), xs.len());
    let n = xsp.len();
    for i in 0..n {
        xsp[i] = to_ark_scalar(&xs[i]);
    }
}

pub fn from_ark_scalar(x: &ArkScalar) -> Scalar {
    let x = ArkScalarConfig::into_bigint(*x);
    let bytes = x.to_bytes_le();
    Scalar::from_canonical_bytes(bytes.try_into().unwrap()).unwrap()
}

pub fn convert_ark_scalar_slice_to_data_slice<'a>(
    alloc: &'a Bump,
    values: &[ArkScalar],
) -> &'a [[u64; 4]] {
    alloc.alloc_slice_fill_iter(values.iter().map(|s| s.into_bigint().0))
}
