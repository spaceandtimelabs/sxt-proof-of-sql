use curve25519_dalek::scalar::Scalar;
use byte_slice_cast::AsMutByteSlice;
use ark_ff::fields::{Fp256, MontBackend, MontConfig, MontFp};
use ark_ff::{BigInt, ToBytes, One};

#[derive(MontConfig)]
#[modulus = "7237005577332262213973186563042994240857116359379907606001950938285454250989"]
#[generator = "2"]
pub struct ArkScalarConfig;
pub type ArkScalar = Fp256<MontBackend<ArkScalarConfig, 4>>;

pub fn to_ark_scalar(x: &Scalar) -> ArkScalar {
    let mut values: [u64; 4] = [0; 4];
    values.as_mut_byte_slice().clone_from_slice(x.as_bytes());
    ArkScalarConfig::from_bigint(BigInt::new(values)).unwrap()
}

pub fn to_ark_scalars(xsp: & mut[ArkScalar], xs: &[Scalar]) {
    assert_eq!(xsp.len(), xs.len());
    let n = xsp.len();
    for i in 0..n {
        xsp[i] = to_ark_scalar(&xs[i]);
    }
}

pub fn from_ark_scalar(x: &ArkScalar) -> Scalar {
    let x = ArkScalarConfig::into_bigint(x.clone());
    let mut bytes = [0u8; 32];
    x.write(bytes.as_mut()).unwrap();
    Scalar::from_canonical_bytes(bytes).unwrap()
}
