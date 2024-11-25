use crate::base::scalar::Curve25519Scalar;
use arrow::row;
use curve25519_dalek::{edwards::CompressedEdwardsY, EdwardsPoint, Scalar};
use rand::{rngs::StdRng, Rng, RngCore};

impl core::ops::Mul<EdwardsPoint> for Curve25519Scalar {
    type Output = EdwardsPoint;
    fn mul(self, rhs: EdwardsPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for EdwardsPoint {
    type Output = EdwardsPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}
impl core::ops::Mul<&EdwardsPoint> for Curve25519Scalar {
    type Output = EdwardsPoint;
    fn mul(self, rhs: &EdwardsPoint) -> Self::Output {
        curve25519_dalek::scalar::Scalar::from(self) * rhs
    }
}
impl core::ops::Mul<Curve25519Scalar> for &EdwardsPoint {
    type Output = EdwardsPoint;
    fn mul(self, rhs: Curve25519Scalar) -> Self::Output {
        self * curve25519_dalek::scalar::Scalar::from(rhs)
    }
}

pub fn generate_random_element(rng: &mut StdRng) -> EdwardsPoint {
    loop {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);

        if let Some(point) = CompressedEdwardsY(bytes).decompress() {
            return point;
        }
    }
}

#[test]
fn elements_should_be_associative_on_addition() {
    use core::iter;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(100);
    let total_triples = 1;
    iter::repeat_with(|| {
        (
            generate_random_element(&mut rng),
            generate_random_element(&mut rng),
            generate_random_element(&mut rng),
        )
    })
    .take(total_triples)
    .for_each(|(a, b, c)| {
        assert_eq!((a + b) + c, a + (b + c));
    });
}

#[test]
fn elements_should_be_commutative_on_addition() {
    use core::iter;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(100);
    let total_doubles = 10000;
    iter::repeat_with(|| {
        (
            generate_random_element(&mut rng),
            generate_random_element(&mut rng),
        )
    })
    .take(total_doubles)
    .for_each(|(a, b)| {
        assert_eq!(a + b, b + a);
    });
}

#[test]
fn elements_should_be_associative_on_scalar_multiplication() {
    use core::iter;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(100);
    let total_triples = 10000;
    iter::repeat_with(|| {
        (
            Curve25519Scalar::from(rng.gen::<u8>()),
            Curve25519Scalar::from(rng.gen::<u8>()),
            generate_random_element(&mut rng),
        )
    })
    .take(total_triples)
    .for_each(|(a, b, c)| {
        assert_eq!((a * b) * c, a * (b * c));
    });
}

#[test]
fn elements_should_be_distibutive() {
    use core::iter;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(100);
    let total_triples = 10000;
    iter::repeat_with(|| {
        (
            Curve25519Scalar::from(rng.gen::<u8>()),
            generate_random_element(&mut rng),
            generate_random_element(&mut rng),
        )
    })
    .take(total_triples)
    .for_each(|(a, b, c)| {
        assert_eq!(a * (b + c), a * b + a * c);
    });
}

#[test]
fn elements_should_pass_hyrax() {
    use core::iter;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(100);
    let dimension = 100;
    let high_vec = iter::repeat_with(|| Curve25519Scalar::from(rng.gen::<u8>()))
        .take(dimension)
        .collect::<Vec<_>>();
    let generators = iter::repeat_with(|| generate_random_element(&mut rng))
        .take(dimension)
        .collect::<Vec<_>>();
    let matrix = iter::repeat_with(|| {
        iter::repeat_with(|| Curve25519Scalar::from(rng.gen::<u8>()))
            .take(dimension)
            .collect::<Vec<_>>()
    })
    .take(dimension)
    .collect::<Vec<_>>();
    let row_commits = matrix.iter().map(|row| {
        row.iter()
            .zip(generators.clone())
            .fold(EdwardsPoint::default(), |acc, (s, e)| {
                acc + *s * e
            })
    }).collect::<Vec<_>>();
    let witness = (0..dimension).fold(vec![Curve25519Scalar::default(); dimension], |acc: Vec<Curve25519Scalar>, row_index| acc.iter().zip(matrix[row_index].clone()).map(|(es, ns)| *es + ns * high_vec[row_index]).collect());
    let row_commits_by_high = row_commits.iter().zip(high_vec).fold(EdwardsPoint::default(), |acc, (rc, w)| acc + rc * w);
    let generators_by_witness = generators.iter().zip(witness).fold(EdwardsPoint::default(), |acc, (rc, w)| acc + rc * w);
    assert_eq!(row_commits_by_high, generators_by_witness);
}
