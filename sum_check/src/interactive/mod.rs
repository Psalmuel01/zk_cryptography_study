use crate::prover::split_and_sum;
use ark_ff::PrimeField;
use multivariate_poly::{partial_evaluate, MultilinearPolynomial};

#[derive(Debug, Clone)]
struct Prover<F: PrimeField> {
    claimed_sum: F,
    univariate_poly: [F; 2],
}

struct Verifier<F: PrimeField> {
    challenges: Vec<F>,
    initial_poly: MultilinearPolynomial<F>,
}

struct SumCheck<F: PrimeField> {
    poly: MultilinearPolynomial<F>,
    verifier: Verifier<F>,
}

#[allow(dead_code)]
impl<F: PrimeField> SumCheck<F> {
    fn init(poly: MultilinearPolynomial<F>) -> Self {
        Self {
            poly: poly.clone(),
            verifier: Verifier {
                challenges: vec![],
                initial_poly: poly,
            },
        }
    }

    fn prove(&mut self, claimed_sum: F) -> Prover<F> {
        let mut poly_coeff = self.poly.coefficients.clone();
        if self.verifier.challenges.len() > 0 {
            dbg!("now check challenge");
            poly_coeff = partial_evaluate(
                self.poly.coefficients.to_vec(),
                0,
                self.verifier.challenges[0],
            );
            self.poly.coefficients = poly_coeff.clone();
        }
        let round_poly = split_and_sum(&poly_coeff);

        Prover {
            claimed_sum,
            univariate_poly: round_poly,
        }
    }

    fn verify(&mut self, prover: Prover<F>, challenge: F) -> bool {
        let round_poly = prover.univariate_poly;
        dbg!(round_poly);
        dbg!(prover.claimed_sum);
        if prover.claimed_sum != round_poly.iter().sum() {
            return false;
        }
        self.verifier.challenges.insert(0, challenge);

        if self.verifier.challenges.len() == self.verifier.initial_poly.no_of_variables() {
            dbg!("last round!");
            let verifier_sum =
                round_poly[0] + self.verifier.challenges[0] * (round_poly[1] - round_poly[0]);
            self.verifier.challenges.reverse();
            let total_sum = self
                .verifier
                .initial_poly
                .evaluate(&self.verifier.challenges);
            dbg!(verifier_sum);
            dbg!(total_sum);
            if verifier_sum != total_sum {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    pub(crate) fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(|v| Fq::from(v)).collect()
    }

    #[test]
    fn test_sum_check() {
        let poly = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 2, 0, 10, 0, 17]));
        let mut sum_check = SumCheck::init(poly);
        let prover = sum_check.prove(Fq::from(29));
        let verifier = sum_check.verify(prover, Fq::from(5));
        dbg!(verifier);
        let prover_1 = sum_check.prove(Fq::from(127));
        let verifier_1 = sum_check.verify(prover_1, Fq::from(3));
        dbg!(verifier_1);
        let prover_2 = sum_check.prove(Fq::from(131));
        let verifier_2 = sum_check.verify(prover_2, Fq::from(2));
        dbg!(verifier_2);
        // check if all true
        assert!(verifier && verifier_1 && verifier_2);

        let poly = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut sum_check = SumCheck::init(poly);
        let prover = sum_check.prove(Fq::from(7));
        let verifier = sum_check.verify(prover, Fq::from(5));
        dbg!(verifier);
        let prover_1 = sum_check.prove(Fq::from(17));
        let verifier_1 = sum_check.verify(prover_1, Fq::from(2));
        dbg!(verifier_1);
        assert!(verifier && verifier_1);
    }
}
