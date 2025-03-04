use crate::{prover::Proof, transcript::Transcript};
use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::MultilinearPolynomial;
use sha3::Keccak256;

#[derive(Debug, Clone)]
pub struct Verify<F: PrimeField> {
    pub transcript: Transcript<Keccak256, F>,
    pub original_poly: MultilinearPolynomial<F>,
}

impl<F: PrimeField> Verify<F> {
    pub fn new(coefficients: &Vec<F>) -> Self {
        Self {
            transcript: Transcript::init(Keccak256::default()),
            original_poly: MultilinearPolynomial::new(coefficients.clone()),
        }
    }

    pub fn verify(&mut self, proof: Proof<F>) -> bool {
        let mut challenges = vec![];

        // self.initial_poly = initial_poly.clone();
        self.transcript
            .absorb(self.original_poly.convert_to_bytes().as_slice());
        self.transcript
            .absorb(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());
        let mut claimed_sum = proof.claimed_sum;

        println!("round_polys: {:?}", proof.round_polys);

        for round_poly in proof.round_polys {
            if claimed_sum != round_poly.iter().sum() {
                return false;
            }
            let converted_poly = MultilinearPolynomial::new(round_poly.to_vec());
            self.transcript
                .absorb(converted_poly.convert_to_bytes().as_slice());
            println!("round poly sum: {:?} ", round_poly.iter().sum::<F>());
            println!("claimed sum: {}", claimed_sum);
            let challenge: F = self.transcript.squeeze();
            claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
            challenges.push(challenge);
        }

        println!("challenges: {:?}", challenges);
        println!("claimed sums: {}", claimed_sum);
        println!(
            "clameddd sum: {:?}",
            self.original_poly.evaluate(&challenges)
        );

        if claimed_sum != self.original_poly.evaluate(&challenges) {
            return false;
        }

        true
    }
}
