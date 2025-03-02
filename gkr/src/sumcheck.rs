use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::sum_poly::SumPoly;
use sha3::Keccak256;
use sum_check::transcript::Transcript;
use univariate_poly::UnivariatePolynomial;

#[derive(Debug, Clone)]
pub struct SumCheckProof<F: PrimeField> {
    pub claimed_sum: F,
    pub round_polys: Vec<UnivariatePolynomial<F>>,
    pub random_challenges: Vec<F>,
}

#[derive(Debug, Clone)]
pub struct GKRProver<F: PrimeField> {
    pub initial_poly: SumPoly<F>,
    pub claimed_sum: F,
    pub transcripts: Transcript<Keccak256, F>,
}

impl<F: PrimeField> GKRProver<F> {
    pub fn new(poly: SumPoly<F>, claimed_sum: F) -> Self {
        Self {
            initial_poly: poly,
            claimed_sum: claimed_sum,
            transcripts: Transcript::init(Keccak256::default()),
        }
    }

    pub fn prove(&mut self) -> SumCheckProof<F> {
        // self.transcripts.absorb(self.initial_poly.convert_to_bytes().as_slice());
        self.transcripts
            .absorb(self.claimed_sum.into_bigint().to_bytes_be().as_slice());

        let mut round_polys = Vec::new();
        let mut current_poly = self.initial_poly.clone();
        let mut random_challenges = Vec::new();
        let no_of_variables = self.initial_poly.no_of_variables();

        for _ in 0..no_of_variables {
            let round_split = split_and_sum(current_poly.clone());

            let x_values: Vec<F> = (0..=self.initial_poly.degree())
                .map(|i| F::from(i as u64))
                .collect();
            // let y_values: Vec<F> = round_split;

            let points: Vec<(F, F)> = x_values
                .iter()
                .zip(round_split.iter())
                .map(|(x, y)| (*x, *y))
                .collect();

            println!("points: {:?}", points);

            let univariate_poly = UnivariatePolynomial::interpolate(points.clone());
            println!("univariate_poly: {:?}", univariate_poly);

            self.transcripts
                .absorb(univariate_poly.convert_to_bytes().as_slice());
            round_polys.push(univariate_poly);

            let challenge: F = self.transcripts.squeeze();
            current_poly = current_poly.partial_evaluate(0, challenge);
            random_challenges.push(challenge);
            println!("challenge_prover: {}", challenge);
        }

        println!("prover_round_poly: {:?}", round_polys);
        println!("challengess: {:?}", random_challenges);
        dbg!(self.claimed_sum);
        dbg!(&round_polys);

        SumCheckProof {
            claimed_sum: self.claimed_sum,
            round_polys: round_polys,
            random_challenges,
        }
    }

    pub fn verify(&mut self, proof: SumCheckProof<F>) -> bool {
        // self.transcripts.absorb(self.initial_poly.convert_to_bytes().as_slice());
        self.transcripts
            .absorb(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

        let mut current_claimed_sum = proof.claimed_sum;
        let mut challenges = Vec::with_capacity(proof.round_polys.len());

        for round_poly in proof.round_polys {
            if round_poly.evaluate(F::from(0)) + round_poly.evaluate(F::from(1))
                != current_claimed_sum
            {
                println!("failed to verify 1");
                return false;
            }
            dbg!("check here");
            self.transcripts
                .absorb(round_poly.convert_to_bytes().as_slice());

            let challenge: F = self.transcripts.squeeze();
            current_claimed_sum = round_poly.evaluate(challenge);
            println!("claimed_sum 2: {}", current_claimed_sum);
            challenges.push(challenge);
        }

        // if claimed_sum != self.initial_poly.evaluate(challenges) {
        //     println!("failed to verify 2");
        //     return false;
        // }
        true
    }
}

fn split_and_sum<F: PrimeField>(mut poly: SumPoly<F>) -> Vec<F> {
    let length = poly.product_polys[0].degree() + 1;

    let mut evaluations = Vec::with_capacity(length);

    for i in 0..length {
        let mut partial_eval = poly.partial_evaluate(0, F::from(i as u64));
        let evaluation = partial_eval.sum_reduce().coefficients.iter().sum();
        evaluations.push(evaluation);
    }

    evaluations
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;
    use multivariate_poly::{product_poly::ProductPoly, MultilinearPolynomial};

    fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(Fq::from).collect()
    }

    #[test]
    fn test_split_and_sum() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 2]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 3]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 1]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 7]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        let result = split_and_sum(sum_poly);
        assert_eq!(result, [Fq::from(0), Fq::from(13), Fq::from(52)]);
        println!("{:?}", result);
    }

    #[test]
    fn test_sumcheck() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 2]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 3]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 1]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 7]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        let mut prover = GKRProver::new(sum_poly, Fq::from(13));
        let proof = prover.prove();
        println!("{:?}", proof);
        let verify = prover.verify(proof);
        assert_eq!(verify, true);
    }
}
