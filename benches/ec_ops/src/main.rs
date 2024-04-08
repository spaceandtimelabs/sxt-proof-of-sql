use ark_bls12_381::{Bls12_381, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{pairing::Pairing, VariableBaseMSM};
use ark_std::UniformRand;
use blitzar::{compute::*, sequence::Sequence};
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use rand::Rng;
use std::time::Instant;

fn wake_up_gpu() {
    // Wake up the GPU so the timing results aren't impacted.
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..1).map(|_| rng.gen::<u8>()).collect();

    // Randomly obtain the generator points
    let gs: Vec<RistrettoPoint> = (0..data.len())
        .map(|_| RistrettoPoint::random(&mut rng))
        .collect();

    // Initialize commitments
    let mut commitments = vec![CompressedRistretto::default(); 1];

    // Compute commitment
    compute_curve25519_commitments_with_generators(&mut commitments, &[(*data).into()], &gs);
}

fn single_bls12_381_g1_commit<'a>(data: impl Into<Sequence<'a>>) {
    // Convert in order to get .len()
    let data = data.into();

    // Randomly obtain the generator points
    let mut rng = ark_std::test_rng();
    let generator_points: Vec<G1Affine> =
        (0..data.len()).map(|_| G1Affine::rand(&mut rng)).collect();

    // Initialize commitments
    let mut commitments = vec![[0_u8; 48]; 1];

    let start_time = Instant::now();

    // Compute commitment
    compute_bls12_381_g1_commitments_with_generators(&mut commitments, &[data], &generator_points);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "bls12-381 G1 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn single_curve25519_commit<'a>(data: impl Into<Sequence<'a>>) {
    // Convert in order to get .len()
    let data = data.into();

    // Randomly obtain the generator points
    let mut rng = rand::thread_rng();
    let gs: Vec<RistrettoPoint> = (0..data.len())
        .map(|_| RistrettoPoint::random(&mut rng))
        .collect();

    // Initialize commitments
    let mut commitments = vec![CompressedRistretto::default(); 1];

    let start_time = Instant::now();

    // Compute commitment
    compute_curve25519_commitments_with_generators(&mut commitments, &[data], &gs);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "curve25518 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn single_arkworks_bls12_381_g1_commit(data_size: usize) {
    // Randomly obtain the generator points
    let mut rng = ark_std::test_rng();
    let generator_points: Vec<G1Affine> =
        (0..data_size).map(|_| G1Affine::rand(&mut rng)).collect();

    // Randomly generate scalar data
    let scalar_data: Vec<Fr> = (0..data_size).map(|_| Fr::rand(&mut rng)).collect();

    let start_time = Instant::now();

    // compute msm in Arkworks
    let _ = G1Projective::msm(&generator_points, &scalar_data).unwrap();

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "Arkworks bls12-381 G1 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn single_arkworks_bls12_381_g2_commit(data_size: usize) {
    // Randomly obtain the generator points
    let mut rng = ark_std::test_rng();
    let generator_points: Vec<G2Affine> =
        (0..data_size).map(|_| G2Affine::rand(&mut rng)).collect();

    // Randomly generate scalar data
    let scalar_data: Vec<Fr> = (0..data_size).map(|_| Fr::rand(&mut rng)).collect();

    let start_time = Instant::now();

    // compute msm in Arkworks
    let _ = G2Projective::msm(&generator_points, &scalar_data).unwrap();

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "Arkworks bls12-381 G2 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn get_random_u256() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut random_array = [0u8; 32];

    for element in &mut random_array {
        *element = rng.gen::<u8>();
    }

    random_array
}

fn time_single_commitment_updates() {
    // Run the single commitment timing tests.
    let data_size = [1000, 10000, 100000, 1000000];
    for d in data_size {
        let mut data: Vec<[u8; 32]> = Vec::with_capacity(d);

        for _ in 0..d {
            data.push(get_random_u256());
        }

        println!(
            "Computing single commitment with {d} data elements of type {}",
            std::any::type_name::<[u8; 32]>()
        );
        single_bls12_381_g1_commit(data.as_slice());
        single_curve25519_commit(data.as_slice());
        single_arkworks_bls12_381_g1_commit(d);
        single_arkworks_bls12_381_g2_commit(d);
    }
}

fn bls12_381_g1_multi(data: &[Vec<[u8; 32]>]) {
    // Randomly obtain the generator points
    let mut rng = ark_std::test_rng();
    let generator_points: Vec<G1Affine> = (0..data[0].len())
        .map(|_| G1Affine::rand(&mut rng))
        .collect();

    // Initialize commitments
    let mut commitments: Vec<[u8; 48]> = vec![[0_u8; 48]; data.len()];
    let seq: Vec<Sequence> = (0..data.len()).map(|i| data[i][..].into()).collect();

    let start_time = Instant::now();

    // Compute commitment
    compute_bls12_381_g1_commitments_with_generators(&mut commitments, &seq, &generator_points);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "bls12-381 G1 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn curve25519_multi(data: &[Vec<[u8; 32]>]) {
    // Randomly obtain the generator points
    let mut rng = rand::thread_rng();
    let gs: Vec<RistrettoPoint> = (0..data[0].len())
        .map(|_| RistrettoPoint::random(&mut rng))
        .collect();

    // Initialize commitments
    let mut commitments = vec![CompressedRistretto::default(); data.len()];
    let seq: Vec<Sequence> = (0..data.len()).map(|i| data[i][..].into()).collect();

    let start_time = Instant::now();

    // Compute commitment
    compute_curve25519_commitments_with_generators(&mut commitments, &seq, &gs);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "curve25518 commitment time: {} milliseconds",
        elapsed_time.as_millis()
    );
}

fn create_random_2d_vector(rows: usize, cols: usize) -> Vec<Vec<[u8; 32]>> {
    let mut result = Vec::with_capacity(rows);

    for _ in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for _ in 0..cols {
            row.push(get_random_u256());
        }
        result.push(row);
    }

    result
}

fn time_multi_commitment() {
    const WIDTH: usize = 1000;
    const HEIGHT: [usize; 3] = [100, 500, 1000];

    for h in &HEIGHT {
        let data = create_random_2d_vector(*h, WIDTH);
        println!(
            "Computing multi commitments. {} commitments with {WIDTH} data elements of type {}.",
            h,
            std::any::type_name::<[u8; 32]>()
        );

        bls12_381_g1_multi(&data);
        curve25519_multi(&data);
    }
}

fn time_arkworks_inner_product_pairing() {
    let number_of_points: &[usize] = &[1, 10, 100, 1000, 10000, 100000];

    for n in number_of_points {
        let mut rng = ark_std::test_rng();

        let lhs: Vec<G1Projective> = (0..*n).map(|_| G1Projective::rand(&mut rng)).collect();

        let rhs: Vec<G2Projective> = (0..*n).map(|_| G2Projective::rand(&mut rng)).collect();

        let start_time = Instant::now();

        // Compute multi pairing.
        let _ = Bls12_381::multi_pairing(lhs, rhs);

        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time);

        println!(
            "Arkworks multi-pairing with {} elements time: {} milliseconds",
            n,
            elapsed_time.as_millis()
        );
    }
}

fn time_single_multi_exponentiation_in_arkworks() {
    // Define a random scalar
    let mut rng = ark_std::test_rng();
    let s = Fr::rand(&mut rng);

    // G1 element
    let g1 = G1Affine::rand(&mut rng);

    let start_time = Instant::now();

    let _ = G1Projective::msm(&[g1], &[s]);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "Arkworks single multi-exponentiation with a G1 element time: {} microseconds",
        elapsed_time.as_micros()
    );

    // G2 element
    let g2 = G2Affine::rand(&mut rng);

    let start_time = Instant::now();

    let _ = G2Projective::msm(&[g2], &[s]);

    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time);

    println!(
        "Arkworks single multi-exponentiation with a G2 element time: {} microseconds",
        elapsed_time.as_micros()
    );
}

fn main() {
    wake_up_gpu();

    time_single_commitment_updates();
    time_multi_commitment();
    time_arkworks_inner_product_pairing();
    time_single_multi_exponentiation_in_arkworks();
}
