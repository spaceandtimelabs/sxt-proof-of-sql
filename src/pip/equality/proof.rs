#![allow(non_snake_case)]
#![allow(unused_variables)]

use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::traits::Identity;
use curve25519_dalek::scalar::Scalar;
use crate::base::proof::{Commitment, PIPProof, ProofError, Transcript};
use crate::pip::equality::hadamard::HadamardProof;
use pedersen::commitments::compute_commitments;
use std::slice;

use std::iter;
use std::ops::Sub;

pub struct EqualityProof{
    pub C_c: Commitment,
    pub C_e: Commitment,
    pub proof_ez0: HadamardProof,
    pub proof_czd: HadamardProof,
}

impl PIPProof for EqualityProof {
    fn create(transcript: &mut Transcript, input_columns: &[&[Scalar]], output_columns: &[&[Scalar]], input_commitments: &[Commitment]) -> Self {
        let a = input_columns[0];
        let b = input_columns[1];
        let e = output_columns[0];
        /*
        println!("create: a = {:?}", a);
        println!("create: b = {:?}", b);
         */
        create_equality_proof (transcript, a, b, e)
    }

    fn verify(&self, transcript: &mut Transcript, input_commitments: &[Commitment]) -> Result<(), ProofError> {
        let C_a = input_commitments[0];
        let C_b = input_commitments[1];
        let C_e = self.C_e;
        verify_proof(transcript, self, C_a, C_b)
    }

    fn get_output_commitments(&self) -> &[Commitment] {
      todo!()
          //self.C_e
    }
}


    fn create_equality_proof (transcript: &mut Transcript, a:&[Scalar], b: &[Scalar], e: &[Scalar]) -> EqualityProof {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), e.len());
        let mut z: Vec<Scalar> = Vec::new();
        //println!("x = {:?}", x);

        for i in 0..a.len(){
            z.push(Scalar::from(a[i] - b[i]));
        }

        let mut c: Vec<Scalar> = Vec::new();
        for i in 0..a.len(){
            if z[i].eq(&Scalar::from(0 as u32)) {
                c.push(Scalar::from(0 as u32));
            } else {
                c.push(Scalar::from(z[i].invert()));
            }
        }

        let mut d:Vec<Scalar> = Vec::new();
        let mut zero:Vec<Scalar> = Vec::new();

        for i in 0..b.len(){
            d.push(Scalar::from(1 as u32) - e[i]);
            zero.push(Scalar::from(0 as u32));
        }

        /*
        println!("a = {:?}", a);
        println!("b = {:?}", b);
        println!("c = {:?}", c);
        println!("d = {:?}", d);
        println!("e = {:?}", e);
        println!("f = {:?}", f);
        */

        /*
          let ab_vec: Vec<Scalar> = a_vec.iter().zip(b_vec.iter()).map(|(a, b)| a * b).collect();
            let mut c_ab = CompressedRistretto::identity();
            compute_commitments(slice::from_mut(&mut c_ab), &[&ab_vec[..]]);
            transcript.append_point(b"c_ab", &c_ab);
        */

        let mut C_c = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut C_c), &[&c[..]]);
        let mut C_e = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut C_e), &[&e[..]]);

        let C_c = Commitment{commitment:C_c, length:c.len()};
        let C_e = Commitment{commitment:C_e, length:e.len()};

        //let C_c =  pedersen::pedersen_commitment(&c);
        //let C_d =  pedersen::pedersen_commitment(&d);

        let proof_ez0 = HadamardProof::create_hadamard_proof(&e, &z, &zero);
        let proof_czd = HadamardProof::create_hadamard_proof(&c, &z, &d);

/*
        let result = compute_result(a, x);
        //println!("result = {:?}", result);

        let mut result_raw: Vec <Scalar> = Vec::new();
        for i in 0..result.len(){
            result_raw.push(result[i][1]);
        }

        let C_result = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut C_result), &[&result_raw[..]]);
*/
        //let C_result = pedersen::pedersen_commitment(&result_raw);

        let prover_results = EqualityProof{
            C_c,
            C_e,
            proof_ez0,
            proof_czd,
        };
        /*
        println!("{:?}", prover_results.C_c);
        println!("{:?}", prover_results.C_d);
        println!("{:?}", prover_results.proof_eb0.a);
        println!("{:?}", prover_results.proof_eb0.b);
        println!("{:?}", prover_results.proof_eb0.c);
        println!("{:?}", prover_results.proof_cbd.a);
        println!("{:?}", prover_results.proof_cbd.b);
        println!("{:?}", prover_results.proof_cbd.c);
        println!("{:?}", prover_results.proof_eaf.a);
        println!("{:?}", prover_results.proof_eaf.b);
        println!("{:?}", prover_results.proof_eaf.c);
        */
        //println!("{:?}", prover_results.result);
        //println!("{:?}", prover_results.length);

        prover_results
    }

fn verify_proof(transcript: &mut Transcript, proof: &EqualityProof, C_a: Commitment, C_b: Commitment) -> Result<(), ProofError>{
    let C_z_u = C_a.commitment.decompress().unwrap().sub(C_b.commitment.decompress().unwrap());
    let C_z = C_z_u.compress();
    let zero : Vec<Scalar> = iter::repeat(Scalar::from(0 as u32)).take(C_a.length).collect();
    let one : Vec<Scalar> = iter::repeat(Scalar::from(1 as u32)).take(C_a.length).collect();

    let mut C_1 = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut C_1), &[&one[..]]);
    //let C_1: RistrettoPoint = pedersen::pedersen_commitment(&one);
    let mut C_0 = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut C_0), &[&zero[..]]);
    //let C_0: RistrettoPoint = pedersen::pedersen_commitment(&zero);

    let C_d_u: RistrettoPoint = C_1.decompress().unwrap() - proof.C_e.commitment.decompress().unwrap();
    let C_d = C_d_u.compress();

    let C_z = Commitment{commitment:C_z, length:C_a.length};
    let C_d = Commitment{commitment:C_d, length:C_a.length};
    let C_0 = Commitment{commitment:C_0, length:C_a.length};
    let C_1 = Commitment{commitment:C_1, length:C_a.length};


    match proof.proof_ez0.verify(proof.C_e, C_z, C_0) {
        Ok(()) => proof.proof_czd.verify(proof.C_c, C_z, C_d),
        Err(e) => Err(e),
    }
/*
    println!("eb0: {}", proof.proof_eb0.verify(C_e, C_b, C_0));
    println!("cbd: {}", proof.proof_cbd.verify(proof.C_c, C_b, proof.C_d));
    println!("eaf: {}", proof.proof_eaf.verify(C_e, C_a, C_f));
    println!("Answer: {}", res);
*/


}

fn compute_result(a:&[Scalar], b: &[Scalar]) -> Vec <Vec<Scalar>>{
    assert_eq!(a.len(), b.len());
    let mut result= Vec::new();

    for i in 0..a.len(){
        if a[i] == b[i]{
            let mut temp: Vec <Scalar> = Vec::new();
            temp.push(Scalar::from(i as u32));
            temp.push(Scalar::one());
            result.push(temp);
        }
        else{
            let mut temp: Vec <Scalar> = Vec::new();
            temp.push(Scalar::from(i as u32));
            temp.push(Scalar::zero());
            result.push(temp);
        }


    }

result
}
