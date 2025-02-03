use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::{partial_evaluate, MultilinearPolynomial};
use crate::prover::split_and_sum;

#[derive(Debug, Clone)]
struct Prover<F: PrimeField> {
    claimed_sum: F,
    univariate_poly: [F; 2],
}

struct Verifier<F: PrimeField> {
    challenges: Vec<F>,
}

struct SumCheck<F: PrimeField> {
    initial_poly: MultilinearPolynomial<F>,
    verifier: Verifier<F>,
    // claimed_sums: Vec<F>,
    // challenges: Vec<F>
}

impl <F: PrimeField> SumCheck<F> {
    fn init(poly: MultilinearPolynomial<F>) -> Self {
        Self {
            initial_poly: poly,
            verifier: Verifier { challenges: vec![] },
            // claimed_sums,
            // challenges,
        }
    }

    fn prove (&mut self, claimed_sum: F) -> Prover<F> {
        let mut poly_coeff = self.initial_poly.coefficients.clone();
        if self.verifier.challenges.len() > 0 {
            dbg!("now check challenge");
            poly_coeff = partial_evaluate(self.initial_poly.coefficients.to_vec(), 0, self.verifier.challenges[0]);
            self.initial_poly.coefficients = poly_coeff.clone();
        }
        let round_poly = split_and_sum(&poly_coeff);
        
        Prover {
            claimed_sum,
            univariate_poly: round_poly
        }
    }

    fn verifier (&mut self, prover: Prover<F>, challenge: F) -> bool {
        let round_poly = prover.univariate_poly;
        dbg!(round_poly);
        dbg!(prover.claimed_sum);
        if prover.claimed_sum != round_poly.iter().sum() {
            return false;
        }
        self.verifier.challenges.insert(0, challenge);

        if round_poly[0] == F::zero() {
            dbg!("last round!");
            let verifier_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
            // let total_sum = self.initial_poly.evaluate(&self.verifier.challenges);
            dbg!(verifier_sum);
            dbg!(prover.claimed_sum);
            dbg!(self.verifier.challenges.clone());
            if verifier_sum != prover.claimed_sum {
            return false
            }
        }

        true
    }
}


#[cfg(test)]
mod tests {
    use crate::test::to_field;

    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_sum_check() {
        let poly = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 2, 0, 10, 0, 17]));
        let mut sum_check = SumCheck::init(poly);
        let prover = sum_check.prove(Fq::from(29));
        let verifier = sum_check.verifier(prover, Fq::from(5));
        dbg!(verifier);
        let prover_1 = sum_check.prove(Fq::from(127));
        let verifier_1 = sum_check.verifier(prover_1, Fq::from(3));
        dbg!(verifier_1);
        let prover_2 = sum_check.prove(Fq::from(131));
        let verifier_2 = sum_check.verifier(prover_2, Fq::from(2));
        dbg!(verifier_2);
        // assert!(verifier);
    }
}
