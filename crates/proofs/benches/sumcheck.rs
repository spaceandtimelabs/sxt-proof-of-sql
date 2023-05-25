use ark_std::rc::Rc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use merlin::Transcript;
use proofs::base::scalar::{One, ToArkScalar};
use proofs::{
    base::polynomial::{ArkScalar, CompositePolynomial, DenseMultilinearExtension},
    proof_primitive::sumcheck::SumcheckProof,
};
use rand::{thread_rng, Rng};
use std::time::Duration;

fn random_mle_with_num_vars<R: Rng>(
    v: usize,
    rng: &mut R,
) -> (ArkScalar, DenseMultilinearExtension) {
    let scalars: Vec<ArkScalar> = (0..2u32.pow(v as u32))
        .map(|_| rng.gen::<u32>())
        .map(ArkScalar::from)
        .collect();

    let sum = scalars.iter().sum();
    let scalars: Vec<ArkScalar> = scalars.iter().map(ToArkScalar::to_ark_scalar).collect();
    let mle = scalars;

    (sum, mle)
}

pub fn bench_sumcheck_prove_degree(c: &mut Criterion) {
    let mut rng = thread_rng();
    let num_vars = 10;

    let zero_evaluation_point = vec![ArkScalar::zero(); num_vars];

    let mut mles = Vec::new();

    let mut group = c.benchmark_group("sumcheck_prove_degree");
    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(10));
    for degree in 1..=16 {
        let (_, new_mle) = random_mle_with_num_vars(num_vars, &mut rng);
        mles.push(Rc::new(new_mle));

        let mut polynomial = CompositePolynomial::new(num_vars);
        polynomial.add_product(mles.clone(), One::one());

        let mut transcript = Transcript::new(b"sumcheck_degree");
        let mut zero_evaluation_point = zero_evaluation_point.clone();

        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(degree),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    SumcheckProof::create(
                        black_box(&mut transcript),
                        black_box(&mut zero_evaluation_point),
                        polynomial,
                    )
                })
            },
        );
    }
    group.finish();
}

pub fn bench_sumcheck_verify_degree(c: &mut Criterion) {
    let mut rng = thread_rng();
    let num_vars = 10;

    let zero_evaluation_point = vec![ArkScalar::zero(); num_vars];

    let mut expected = ArkScalar::one();
    let mut mles = Vec::new();

    let mut group = c.benchmark_group("sumcheck_verify_degree");
    for degree in 1..=16 {
        let (new_sum, new_mle) = random_mle_with_num_vars(num_vars, &mut rng);
        expected *= new_sum;
        mles.push(Rc::new(new_mle));

        let mut polynomial = CompositePolynomial::new(num_vars);
        polynomial.add_product(mles.clone(), One::one());

        // Create proof
        let mut transcript = Transcript::new(b"sumcheck_degree");
        let mut zero_evaluation_point = zero_evaluation_point.clone();
        let proof = SumcheckProof::create(&mut transcript, &mut zero_evaluation_point, &polynomial);

        // Verify proof w/ measurements
        let mut transcript = Transcript::new(b"sumcheck_degree");

        group.throughput(Throughput::Elements(degree as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(degree),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    proof.verify_without_evaluation(
                        black_box(&mut transcript),
                        polynomial.info(),
                        black_box(&expected),
                    )
                })
            },
        );
    }
    group.finish();
}

pub fn bench_sumcheck_prove_terms(c: &mut Criterion) {
    let mut rng = thread_rng();
    let num_vars = 10;

    let zero_evaluation_point = vec![ArkScalar::zero(); num_vars];
    let mut polynomial = CompositePolynomial::new(num_vars);

    let mut group = c.benchmark_group("sumcheck_prove_terms");
    for terms in 1..=16 {
        let (_, new_mle) = random_mle_with_num_vars(num_vars, &mut rng);
        polynomial.add_product([Rc::new(new_mle)], One::one());

        let mut transcript = Transcript::new(b"sumcheck_terms");
        let mut zero_evaluation_point = zero_evaluation_point.clone();

        group.throughput(Throughput::Elements(terms as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(terms),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    SumcheckProof::create(
                        black_box(&mut transcript),
                        black_box(&mut zero_evaluation_point),
                        polynomial,
                    )
                })
            },
        );
    }
    group.finish();
}

pub fn bench_sumcheck_verify_terms(c: &mut Criterion) {
    let mut rng = thread_rng();
    let num_vars = 10;

    let zero_evaluation_point = vec![ArkScalar::zero(); num_vars];

    let mut expected_sum = ArkScalar::zero();
    let mut polynomial = CompositePolynomial::new(num_vars);

    let mut group = c.benchmark_group("sumcheck_verify_terms");
    for terms in 1..=16 {
        let (new_sum, new_mle) = random_mle_with_num_vars(num_vars, &mut rng);
        polynomial.add_product([Rc::new(new_mle)], One::one());
        expected_sum += new_sum;

        // Create proof
        let mut transcript = Transcript::new(b"sumcheck_terms");
        let mut zero_evaluation_point = zero_evaluation_point.clone();
        let proof = SumcheckProof::create(&mut transcript, &mut zero_evaluation_point, &polynomial);

        // Verify proof w/ measurements
        let mut transcript = Transcript::new(b"sumcheck_terms");

        group.throughput(Throughput::Elements(terms as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(terms),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    proof.verify_without_evaluation(
                        black_box(&mut transcript),
                        polynomial.info(),
                        black_box(&expected_sum),
                    )
                })
            },
        );
    }
    group.finish();
}

pub fn bench_sumcheck_prove_rows(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("sumcheck_prove_rows");
    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(10));
    for num_vars in 1..=16 {
        let (_, mle) = random_mle_with_num_vars(num_vars, &mut rng);
        let rows = 2u32.pow(num_vars as u32);

        let mut polynomial = CompositePolynomial::new(num_vars);

        polynomial.add_product([Rc::new(mle.clone())], One::one());

        let mut transcript = Transcript::new(b"sumcheck_rows");
        let mut zero_evaluation_point = vec![ArkScalar::zero(); num_vars];

        group.throughput(Throughput::Elements(rows as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(rows),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    SumcheckProof::create(
                        black_box(&mut transcript),
                        black_box(&mut zero_evaluation_point),
                        polynomial,
                    )
                })
            },
        );
    }
    group.finish();
}

pub fn bench_sumcheck_verify_rows(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut group = c.benchmark_group("sumcheck_verify_rows");
    for num_vars in 1..=16 {
        let (column_sum, mle) = random_mle_with_num_vars(num_vars, &mut rng);
        let rows = 2u32.pow(num_vars as u32);

        let mut polynomial = CompositePolynomial::new(num_vars);
        polynomial.add_product([Rc::new(mle.clone())], One::one());

        // Create proof
        let mut transcript = Transcript::new(b"sumcheck_rows");
        let mut zero_evaluation_point = vec![ArkScalar::zero(); num_vars];
        let proof = SumcheckProof::create(&mut transcript, &mut zero_evaluation_point, &polynomial);

        // Verify proof w/ measurements
        let mut transcript = Transcript::new(b"sumcheck_rows");

        group.throughput(Throughput::Elements(rows as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(rows),
            &polynomial,
            |b, polynomial| {
                b.iter(|| {
                    proof.verify_without_evaluation(
                        black_box(&mut transcript),
                        polynomial.info(),
                        black_box(&column_sum),
                    )
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_sumcheck_prove_degree,
    bench_sumcheck_verify_degree,
    bench_sumcheck_prove_terms,
    bench_sumcheck_verify_terms,
    bench_sumcheck_prove_rows,
    bench_sumcheck_verify_rows
);
criterion_main!(benches);
