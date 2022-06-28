#![allow(non_snake_case)]

/**
 * Adopted from dalek-cryptograph
 *
 * Copyright (c) 2016-2021 isis agora lovecruft. All rights reserved.
 * Copyright (c) 2016-2021 Henry de Valence. All rights reserved.
 *
 * See third_party/license/dalek.LICENSE
 */
use core::iter;

use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::VartimeMultiscalarMul;

use crate::base::proof::{ProofError, Transcript};
use crate::base::scalar::inner_product;

#[derive(Clone, Debug)]
pub struct InnerProductProof {
    pub L_vec: Vec<CompressedRistretto>,
    pub R_vec: Vec<CompressedRistretto>,
    pub a: Scalar,
}

impl InnerProductProof {
    /// Create an inner-product proof.
    ///
    /// The proof is created with respect to the bases \\(G\\),
    ///
    /// The `verifier` is passed in as a parameter so that the
    /// challenges depend on the *entire* transcript (including parent
    /// protocols).
    ///
    /// The lengths of the vectors must all be the same, and must all be
    /// either 0 or a power of 2.
    pub fn create(
        transcript: &mut Transcript,
        Q: &RistrettoPoint,
        G: &[RistrettoPoint],
        a: &[Scalar],
        b: &[Scalar],
    ) -> InnerProductProof {
        let mut n = G.len();

        // All of the input vectors must have the same length.
        assert_eq!(G.len(), n);
        assert_eq!(a.len(), n);
        assert_eq!(b.len(), n);
        // All of the input vectors must have a length that is a power of two.
        assert!(n.is_power_of_two());

        // Create cloned slices G, a, b backed by their respective
        // vectors.  This lets us reslice as we compress the lengths
        // of the vectors in the main loop below.
        let mut G = &mut G.to_vec()[..];
        let mut a = &mut a.to_vec()[..];
        let mut b = &mut b.to_vec()[..];

        transcript.innerproduct_domain_sep(n as u64);

        let lg_n = n.next_power_of_two().trailing_zeros() as usize;
        let mut L_vec = Vec::with_capacity(lg_n);
        let mut R_vec = Vec::with_capacity(lg_n);

        // If it's the first iteration, unroll the Hprime = H*y_inv scalar mults
        // into multiscalar muls, for performance.
        if n != 1 {
            n = n / 2;
            let (a_L, a_R) = a.split_at_mut(n);
            let (b_L, b_R) = b.split_at_mut(n);
            let (G_L, G_R) = G.split_at_mut(n);

            let c_L = inner_product(&a_L, &b_R);
            let c_R = inner_product(&a_R, &b_L);

            let L = RistrettoPoint::vartime_multiscalar_mul(
                a_L.iter().map(|&a_L_i| a_L_i).chain(iter::once(c_L)),
                G_R.iter().chain(iter::once(Q)),
            )
            .compress();

            let R = RistrettoPoint::vartime_multiscalar_mul(
                a_R.iter().map(|&a_R_i| a_R_i).chain(iter::once(c_R)),
                G_L.iter().chain(iter::once(Q)),
            )
            .compress();

            L_vec.push(L);
            R_vec.push(R);

            transcript.append_point(b"L", &L);
            transcript.append_point(b"R", &R);

            let u = transcript.challenge_scalar(b"u");
            let u_inv = u.invert();

            for i in 0..n {
                a_L[i] = a_L[i] * u + u_inv * a_R[i];
                b_L[i] = b_L[i] * u_inv + u * b_R[i];
                G_L[i] = RistrettoPoint::vartime_multiscalar_mul(&[u_inv, u], &[G_L[i], G_R[i]]);
            }

            a = a_L;
            b = b_L;
            G = G_L;
        }

        while n != 1 {
            n = n / 2;
            let (a_L, a_R) = a.split_at_mut(n);
            let (b_L, b_R) = b.split_at_mut(n);
            let (G_L, G_R) = G.split_at_mut(n);

            let c_L = inner_product(&a_L, &b_R);
            let c_R = inner_product(&a_R, &b_L);

            let L = RistrettoPoint::vartime_multiscalar_mul(
                a_L.iter().chain(iter::once(&c_L)),
                G_R.iter().chain(iter::once(Q)),
            )
            .compress();

            let R = RistrettoPoint::vartime_multiscalar_mul(
                a_R.iter().chain(iter::once(&c_R)),
                G_L.iter().chain(iter::once(Q)),
            )
            .compress();

            L_vec.push(L);
            R_vec.push(R);

            transcript.append_point(b"L", &L);
            transcript.append_point(b"R", &R);

            let u = transcript.challenge_scalar(b"u");
            let u_inv = u.invert();

            for i in 0..n {
                a_L[i] = a_L[i] * u + u_inv * a_R[i];
                b_L[i] = b_L[i] * u_inv + u * b_R[i];
                G_L[i] = RistrettoPoint::vartime_multiscalar_mul(&[u_inv, u], &[G_L[i], G_R[i]]);
            }

            a = a_L;
            b = b_L;
            G = G_L;
        }

        InnerProductProof {
            L_vec: L_vec,
            R_vec: R_vec,
            a: a[0],
        }
    }

    // Computes three vectors of verification scalars \\([u\_{i}^{2}]\\), \\([u\_{i}^{-2}]\\) and \\([s\_{i}]\\) for combined multiscalar multiplication and the scalar b'
    // in a parent protocol. See [inner product protocol notes](index.html#verification-equation) for details.
    // The verifier must provide the input length \\(n\\) explicitly to avoid unbounded allocation within the inner product proof.
    pub(crate) fn verification_scalars(
        &self,
        transcript: &mut Transcript,
        b: &[Scalar],
    ) -> Result<(Vec<Scalar>, Vec<Scalar>, Vec<Scalar>, Scalar), ProofError> {
        let n = b.len();
        let lg_n = self.L_vec.len();
        if lg_n >= 32 {
            // 4 billion multiplications should be enough for anyone
            // and this check prevents overflow in 1<<lg_n below.
            return Err(ProofError::VerificationError);
        }
        if n != (1 << lg_n) {
            return Err(ProofError::VerificationError);
        }

        transcript.innerproduct_domain_sep(n as u64);

        // 1. Recompute x_k,...,x_1 based on the proof transcript

        let mut challenges = Vec::with_capacity(lg_n);
        for (L, R) in self.L_vec.iter().zip(self.R_vec.iter()) {
            transcript.append_point(b"L", L);
            transcript.append_point(b"R", R);
            challenges.push(transcript.challenge_scalar(b"u"));
        }

        // 2. Compute 1/(u_k...u_1) and 1/u_k, ..., 1/u_1

        let mut challenges_inv = challenges.clone();
        let allinv = Scalar::batch_invert(&mut challenges_inv);

        // 3. Compute folded b
        let b_prime = compute_b_prime(b, &challenges, &challenges_inv);

        // 4. Compute u_i^2 and (1/u_i)^2

        for i in 0..lg_n {
            // XXX missing square fn upstream
            challenges[i] = challenges[i] * challenges[i];
            challenges_inv[i] = challenges_inv[i] * challenges_inv[i];
        }
        let challenges_sq = challenges;
        let challenges_inv_sq = challenges_inv;

        // 5. Compute s values inductively.

        let mut s = Vec::with_capacity(n);
        s.push(allinv);
        for i in 1..n {
            let lg_i = (32 - 1 - (i as u32).leading_zeros()) as usize;
            let k = 1 << lg_i;
            // The challenges are stored in "creation order" as [u_k,...,u_1],
            // so u_{lg(i)+1} = is indexed by (lg_n-1) - lg_i
            let u_lg_i_sq = challenges_sq[(lg_n - 1) - lg_i];
            s.push(s[i - k] * u_lg_i_sq);
        }

        Ok((challenges_sq, challenges_inv_sq, s, b_prime))
    }

    /// This method is for testing that proof generation work,
    /// but for efficiency the actual protocols would use `verification_scalars`
    /// method to combine inner product verification with other checks
    /// in a single multiscalar multiplication.
    #[allow(dead_code)]
    pub fn verify(
        &self,
        transcript: &mut Transcript,
        P: &RistrettoPoint,
        Q: &RistrettoPoint,
        G: &[RistrettoPoint],
        b: &[Scalar],
    ) -> Result<(), ProofError> {
        let (u_sq, u_inv_sq, s, b_prime) = self.verification_scalars(transcript, b)?;

        let a_times_s = s.iter().map(|s_i| self.a * s_i).take(G.len());

        let neg_u_sq = u_sq.iter().map(|ui| -ui);
        let neg_u_inv_sq = u_inv_sq.iter().map(|ui| -ui);

        let Ls = self
            .L_vec
            .iter()
            .map(|p| p.decompress().ok_or(ProofError::VerificationError))
            .collect::<Result<Vec<_>, _>>()?;

        let Rs = self
            .R_vec
            .iter()
            .map(|p| p.decompress().ok_or(ProofError::VerificationError))
            .collect::<Result<Vec<_>, _>>()?;

        let expect_P = RistrettoPoint::vartime_multiscalar_mul(
            iter::once(self.a * b_prime)
                .chain(a_times_s)
                //.chain(h_times_b_div_s)
                .chain(neg_u_sq)
                .chain(neg_u_inv_sq),
            iter::once(Q)
                .chain(G.iter())
                //.chain(H.iter())
                .chain(Ls.iter())
                .chain(Rs.iter()),
        );

        if expect_P == *P {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }
}

fn compute_b_prime(b: &[Scalar], u_vec: &[Scalar], u_inv_vec: &[Scalar]) -> Scalar {
    let mut n = b.len();
    let mut b = &mut b.to_vec()[..];
    let mut u_index = 0;
    while n != 1 {
        n = n / 2;
        let (b_L, b_R) = b.split_at_mut(n);
        let u = u_vec[u_index];
        let u_inv = u_inv_vec[u_index];
        for i in 0..n {
            b_L[i] = b_L[i] * u_inv + u * b_R[i];
        }
        b = b_L;
        u_index += 1;
    }
    b[0]
}
