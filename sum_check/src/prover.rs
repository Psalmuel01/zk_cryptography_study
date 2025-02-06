use ark_ff::{BigInteger, PrimeField};
use sha3::Keccak256;
use multivariate_poly::{partial_evaluate, MultilinearPolynomial};
use crate::transcript::Transcript;

#[derive(Debug, Clone)]
pub struct Proof<F: PrimeField> {
    pub claimed_sum: F,
    pub round_polys: Vec<[F; 2]>,
}

#[derive(Debug, Clone)]
pub struct Prover<F: PrimeField> {
    pub initial_poly: MultilinearPolynomial<F>,
    pub claimed_sum: F,
    pub transcripts: Transcript<Keccak256, F>,
}

impl <F: PrimeField> Prover<F> {
    pub fn new(poly_eval_points: &Vec<F>, claimed_sum: F) -> Self {
        let poly = MultilinearPolynomial::new(poly_eval_points.clone());
        Self {
            initial_poly: poly,
            claimed_sum: claimed_sum,
            transcripts: Transcript::init(Keccak256::default()),
        }
    }

    pub fn prove(&mut self) -> Proof<F> {
        let mut round_polys = Vec::new();

        // append poly eval coefficients
        self.transcripts.absorb(self.initial_poly.convert_to_bytes().as_slice());
        self.transcripts.absorb(self.claimed_sum.into_bigint().to_bytes_be().as_slice());

        let mut poly = self.initial_poly.clone();

        for _ in 0..self.initial_poly.dimension() {
            let round_poly_coeffs = split_and_sum(&poly.coefficients);
            let round_poly = MultilinearPolynomial::new(round_poly_coeffs.to_vec());
            self.transcripts.absorb(round_poly.convert_to_bytes().as_slice());
            round_polys.push(round_poly_coeffs);

            let challenge: F = self.transcripts.squeeze();
            // println!("challenge_prover: {}", challenge);
            poly = poly.partial_evaluate(0, challenge);

        };

        // println!("prover_round_poly: {:?}", round_polys);
        // dbg!(self.claimed_sum);
        // dbg!(&round_polys);

        Proof {
            claimed_sum: self.claimed_sum,
            round_polys: round_polys,
        }
    }
}

pub(crate) fn split_and_sum<F: PrimeField>(poly_coeff: &Vec<F>) -> [F; 2] {
    let mut result = [F::zero(); 2];
    let mid = poly_coeff.len() / 2;
    let (left, right) = poly_coeff.split_at(mid);

    let left_sum: F = left.iter().sum();
    let right_sum: F = right.iter().sum();

    result[0] = left_sum;
    result[1] = right_sum;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_split_and_sum() {
        let poly_coeff = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let result = split_and_sum(&poly_coeff);
        assert_eq!(result, [Fq::from(3), Fq::from(7)]);
    }
}