use super::BNScalar;

fn convert_halo2_fq_to_limbs(value: halo2curves::bn256::Fq) -> [u64; 4] {
    unsafe { core::mem::transmute(value) }
}
fn convert_limbs_to_halo2_fq(value: [u64; 4]) -> halo2curves::bn256::Fq {
    unsafe { core::mem::transmute(value) }
}
fn convert_halo2_fr_to_limbs(value: halo2curves::bn256::Fr) -> [u64; 4] {
    unsafe { core::mem::transmute(value) }
}
fn convert_limbs_to_halo2_fr(value: [u64; 4]) -> halo2curves::bn256::Fr {
    unsafe { core::mem::transmute(value) }
}
/// Converts a Halo2 BN256 G1 Affine point to an Arkworks BN254 G1 Affine point.
pub fn convert_g1_affine_from_halo2_to_ark(
    point: halo2curves::bn256::G1Affine,
) -> ark_bn254::G1Affine {
    let infinity = point == halo2curves::bn256::G1Affine::default();
    let x = ark_ff::Fp::new_unchecked(ark_ff::BigInt(convert_halo2_fq_to_limbs(point.x)));
    let y = ark_ff::Fp::new_unchecked(ark_ff::BigInt(convert_halo2_fq_to_limbs(point.y)));
    ark_bn254::G1Affine { x, y, infinity }
}
/// Converts an Arkworks BN254 G1 Affine point to a Halo2 BN256 G1 Affine point.
pub fn convert_g1_affine_from_ark_to_halo2(
    point: ark_bn254::G1Affine,
) -> halo2curves::bn256::G1Affine {
    if point.infinity {
        halo2curves::bn256::G1Affine::default()
    } else {
        let x = convert_limbs_to_halo2_fq(bytemuck::cast(point.x.0 .0));
        let y = convert_limbs_to_halo2_fq(bytemuck::cast(point.y.0 .0));
        halo2curves::bn256::G1Affine { x, y }
    }
}
/// Converts a Halo2 Scalar to an Arkworks Scalar.
pub fn convert_scalar_from_halo2_to_ark(scalar: halo2curves::bn256::Fr) -> ark_bn254::Fr {
    ark_ff::Fp::new_unchecked(ark_ff::BigInt(convert_halo2_fr_to_limbs(scalar)))
}
/// Converts an Arkworks Scalar to a Halo2 Scalar.
pub fn convert_scalar_from_ark_to_halo2(scalar: ark_bn254::Fr) -> halo2curves::bn256::Fr {
    convert_limbs_to_halo2_fr(bytemuck::cast(scalar.0 .0))
}

impl From<halo2curves::bn256::G1Affine> for super::HyperKZGCommitment {
    fn from(point: halo2curves::bn256::G1Affine) -> Self {
        let commitment = ark_ec::AffineRepr::into_group(convert_g1_affine_from_halo2_to_ark(point));
        Self { commitment }
    }
}
impl From<super::HyperKZGCommitment> for halo2curves::bn256::G1Affine {
    fn from(commitment: super::HyperKZGCommitment) -> Self {
        convert_g1_affine_from_ark_to_halo2(ark_ec::CurveGroup::into_affine(commitment.commitment))
    }
}
impl From<halo2curves::bn256::Fr> for BNScalar {
    fn from(scalar: halo2curves::bn256::Fr) -> Self {
        Self(convert_scalar_from_halo2_to_ark(scalar))
    }
}
