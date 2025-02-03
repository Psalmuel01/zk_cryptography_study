use ark_ff::{BigInteger, PrimeField};
use sha3::Keccak256;
use multivariate_poly::MultilinearPolynomial;
use crate::transcript::Transcript;

#[derive(Debug, Clone)]
struct Prover<F: PrimeField> {
    claimed_sum: F,
}

struct Verifier<F: PrimeField> {
    challenges: Vec<F>,
}

struct SumCheck<F: PrimeField> {
    initial_poly: MultilinearPolynomial<F>,
    // claimed_sums: Vec<F>,
    // challenges: Vec<F>
}

impl <F: PrimeField> SumCheck<F> {
    fn init(poly: MultilinearPolynomial<F>) -> Self {
        Self {
            initial_poly: poly,
            // claimed_sums,
            // challenges,
        }
    }
}

