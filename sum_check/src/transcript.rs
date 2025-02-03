// use ark_bn254::Fq;
// use univariate_poly::Polynomial;
// use multivariate_poly::MultilinearPolynomial;

use ark_ff::PrimeField;
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Transcript<K: HashTrait, F: PrimeField> {
    _field: PhantomData<F>,
    hash_function: K,
}

impl<K: HashTrait, F: PrimeField> Transcript<K, F> {
    pub fn init(hash_function: K) -> Self {
        Self {
            _field: PhantomData,
            hash_function,
        }
    }

    pub fn absorb(&mut self, data: &[u8]) {
        self.hash_function.append(data);
    }

    pub fn squeeze(&self) -> F {
        let hash_output = self.hash_function.generate_hash();
        F::from_le_bytes_mod_order(&hash_output)
    }
}

pub trait HashTrait {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&self) -> Vec<u8>;
}

impl HashTrait for Keccak256 {
    fn append(&mut self, data: &[u8]) {
        self.update(data);
    }

    fn generate_hash(&self) -> Vec<u8> {
        self.clone().finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::<Keccak256, Fq>::init(Keccak256::new());
        transcript.absorb(b"hello world");
        let challenge: Fq = transcript.squeeze();
        dbg!("{}", challenge);
    }
}

// /// Simple example of using Fiat-Shamir in a proof system
// fn fiat_shamir_example<F: PrimeField>() {
//     let mut transcript = Transcript::<Keccak256, F>::init(Keccak256::default());

//     // Prover commits to a value
//     let commitment: F = F::from(123u64);
//     let commitment_bytes = commitment.into_bigint().to_bytes_be();
//     let string_commitment = "hello world";
//     let poly_commitment = [
//         commitment_bytes.clone(),
//         string_commitment.as_bytes().to_vec(),
//     ];
//     let poly_commitment_bytes = poly_commitment.concat();

//     transcript.absorb(&commitment_bytes);
//     transcript.absorb(string_commitment.as_bytes());
//     transcript.absorb(&poly_commitment_bytes);
//     // Verifier derives a challenge
//     let challenge: F = transcript.squeeze();
//     println!("Generated Challenge: {:?}", challenge);
// }
