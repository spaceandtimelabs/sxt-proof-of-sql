fn convert_limbs_to_halo2_fq(value: [u64; 4]) -> halo2curves::bn256::Fq {
    unsafe { core::mem::transmute(value) }
}
fn convert_halo2_fq_to_limbs(value: halo2curves::bn256::Fq) -> [u64; 4] {
    unsafe { core::mem::transmute(value) }
}
fn convert_limbs_to_halo2_fr(value: [u64; 4]) -> halo2curves::bn256::Fr {
    unsafe { core::mem::transmute(value) }
}
fn convert_halo2_fr_to_limbs(value: halo2curves::bn256::Fr) -> [u64; 4] {
    unsafe { core::mem::transmute(value) }
}

fn convert_fq_from_ark_to_halo2(field: ark_bn254::Fq) -> halo2curves::bn256::Fq {
    convert_limbs_to_halo2_fq(field.0 .0)
}
fn convert_fq_from_halo2_to_ark(field: halo2curves::bn256::Fq) -> ark_bn254::Fq {
    ark_ff::Fp::new_unchecked(ark_ff::BigInt(convert_halo2_fq_to_limbs(field)))
}
fn convert_fr_from_ark_to_halo2(field: ark_bn254::Fr) -> halo2curves::bn256::Fr {
    convert_limbs_to_halo2_fr(field.0 .0)
}
fn convert_fr_from_halo2_to_ark(field: halo2curves::bn256::Fr) -> ark_bn254::Fr {
    ark_ff::Fp::new_unchecked(ark_ff::BigInt(convert_halo2_fr_to_limbs(field)))
}

fn convert_g1_affine_from_halo2_to_ark(point: halo2curves::bn256::G1Affine) -> ark_bn254::G1Affine {
    use halo2curves::group::prime::PrimeCurveAffine;
    if point == halo2curves::bn256::G1Affine::identity() {
        ark_bn254::G1Affine::identity()
    } else {
        let x = convert_fq_from_halo2_to_ark(point.x);
        let y = convert_fq_from_halo2_to_ark(point.y);
        ark_bn254::G1Affine::new_unchecked(x, y)
    }
}
fn convert_g1_affine_from_ark_to_halo2(point: ark_bn254::G1Affine) -> halo2curves::bn256::G1Affine {
    use halo2curves::group::prime::PrimeCurveAffine;
    if point.infinity {
        halo2curves::bn256::G1Affine::identity()
    } else {
        let x = convert_fq_from_ark_to_halo2(point.x);
        let y = convert_fq_from_ark_to_halo2(point.y);
        halo2curves::bn256::G1Affine { x, y }
    }
}

impl From<&super::HyperKZGCommitment> for halo2curves::bn256::G1Affine {
    fn from(commitment: &super::HyperKZGCommitment) -> Self {
        use ark_ec::CurveGroup;
        convert_g1_affine_from_ark_to_halo2(commitment.commitment.into_affine())
    }
}
impl From<halo2curves::bn256::G1Affine> for super::HyperKZGCommitment {
    fn from(point: halo2curves::bn256::G1Affine) -> Self {
        use ark_ec::AffineRepr;
        let commitment = convert_g1_affine_from_halo2_to_ark(point).into_group();
        Self { commitment }
    }
}
impl From<&super::BNScalar> for halo2curves::bn256::Fr {
    fn from(value: &super::BNScalar) -> Self {
        convert_fr_from_ark_to_halo2(value.0)
    }
}
impl From<halo2curves::bn256::Fr> for super::BNScalar {
    fn from(scalar: halo2curves::bn256::Fr) -> Self {
        Self(convert_fr_from_halo2_to_ark(scalar))
    }
}

impl From<super::HyperKZGCommitment> for halo2curves::bn256::G1Affine {
    fn from(commitment: super::HyperKZGCommitment) -> Self {
        Self::from(&commitment)
    }
}
impl From<super::BNScalar> for halo2curves::bn256::Fr {
    fn from(value: super::BNScalar) -> Self {
        Self::from(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BNScalar, HyperKZGCommitment};
    use crate::base::scalar::MontScalar;
    use ark_ec::{AdditiveGroup, AffineRepr};
    use ark_ff::Field as _;
    use ark_std::UniformRand;
    use ff::Field as _;
    use halo2curves::group::prime::PrimeCurveAffine;

    #[test]
    fn we_can_convert_commitment_generator() {
        let commitment = HyperKZGCommitment::from(&ark_bn254::G1Affine::generator());
        let point = halo2curves::bn256::G1Affine::generator();
        assert_eq!(halo2curves::bn256::G1Affine::from(commitment), point);
        assert_eq!(commitment, HyperKZGCommitment::from(point));
    }

    #[test]
    fn we_can_convert_commitment_identity() {
        let commitment = HyperKZGCommitment::from(&ark_bn254::G1Affine::identity());
        let point = halo2curves::bn256::G1Affine::identity();
        assert_eq!(halo2curves::bn256::G1Affine::from(commitment), point);
        assert_eq!(commitment, HyperKZGCommitment::from(point));
    }

    #[test]
    fn we_can_convert_scalar_zero() {
        let scalar: BNScalar = MontScalar(ark_bn254::Fr::ZERO);
        let point = halo2curves::bn256::Fr::ZERO;
        assert_eq!(halo2curves::bn256::Fr::from(scalar), point);
        assert_eq!(scalar, BNScalar::from(point));
    }

    #[test]
    fn we_can_convert_scalar_one() {
        let scalar: BNScalar = MontScalar(ark_bn254::Fr::ONE);
        let point = halo2curves::bn256::Fr::ONE;
        assert_eq!(halo2curves::bn256::Fr::from(scalar), point);
        assert_eq!(scalar, BNScalar::from(point));
    }

    #[test]
    fn we_can_round_trip_random_commitments() {
        let mut rng = ark_std::test_rng();
        for _ in 0..100 {
            let ark_point = ark_bn254::G1Affine::rand(&mut rng);
            let commitment = HyperKZGCommitment::from(&ark_point);
            let halo2_point = halo2curves::bn256::G1Affine::from(commitment);
            let round_trip_commitment = HyperKZGCommitment::from(halo2_point);
            assert_eq!(commitment, round_trip_commitment);
        }
    }

    #[test]
    fn we_can_round_trip_random_ark_scalars() {
        let mut rng = ark_std::test_rng();
        for _ in 0..100 {
            let ark_scalar = MontScalar(ark_bn254::Fr::rand(&mut rng));
            let halo2_scalar = halo2curves::bn256::Fr::from(ark_scalar);
            let round_trip_scalar = BNScalar::from(halo2_scalar);
            assert_eq!(ark_scalar, round_trip_scalar);
        }
    }

    #[test]
    fn we_can_round_trip_random_points() {
        let mut rng = ark_std::test_rng();
        for _ in 0..100 {
            let halo2_point = halo2curves::bn256::G1Affine::random(&mut rng);
            let commitment = HyperKZGCommitment::from(halo2_point);
            let round_trip_point = halo2curves::bn256::G1Affine::from(commitment);
            assert_eq!(halo2_point, round_trip_point);
        }
    }

    #[test]
    fn we_can_round_trip_random_halo2_scalars() {
        let mut rng = ark_std::test_rng();
        for _ in 0..100 {
            let halo2_scalar = halo2curves::bn256::Fr::random(&mut rng);
            let ark_scalar = BNScalar::from(halo2_scalar);
            let round_trip_scalar = halo2curves::bn256::Fr::from(ark_scalar);
            assert_eq!(halo2_scalar, round_trip_scalar);
        }
    }
}
