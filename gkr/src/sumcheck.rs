use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::sum_poly::SumPoly;
use sha3::Keccak256;
use sum_check::transcript::Transcript;
use univariate_poly::UnivariatePolynomial;

#[derive(Debug, Clone)]
pub struct PartialProof<F: PrimeField> {
    pub claimed_sum: F,
    pub round_polys: Vec<UnivariatePolynomial<F>>,
    pub random_challenges: Vec<F>,
}

#[derive(Debug, Clone)]
pub struct PartialVerif<F: PrimeField> {
    pub is_proof_valid: bool,
    pub random_challenges: Vec<F>,
    pub last_claimed_sum: F,
}

pub fn partial_prove<F: PrimeField>(
    sum_poly: SumPoly<F>,
    claimed_sum: F,
    transcript: &mut Transcript<Keccak256, F>,
) -> PartialProof<F> {
    // transcript.absorb(initial_poly.convert_to_bytes().as_slice());
    transcript.absorb(claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut round_polys = Vec::new();
    let mut current_poly = sum_poly.clone();
    let mut random_challenges = Vec::new();
    let no_of_variables = sum_poly.no_of_variables();

    for _ in 0..no_of_variables {
        let round_split = split_and_sum(current_poly.clone());

        let x_values: Vec<F> = (0..=sum_poly.degree()).map(|i| F::from(i as u64)).collect();
        // let y_values: Vec<F> = round_split;

        let points: Vec<(F, F)> = x_values
            .iter()
            .zip(round_split.iter())
            .map(|(x, y)| (*x, *y))
            .collect();

        dbg!(&points);

        let univariate_poly = UnivariatePolynomial::interpolate(points);
        dbg!(&univariate_poly);

        transcript.absorb(univariate_poly.convert_to_bytes().as_slice());
        round_polys.push(univariate_poly);

        let challenge: F = transcript.squeeze();
        current_poly = current_poly.partial_evaluate(0, challenge);
        random_challenges.push(challenge);
        println!("challenge_prover: {}", challenge);
    }

    println!("challengess: {:?}", random_challenges);
    dbg!(claimed_sum);
    dbg!(&round_polys);

    PartialProof {
        claimed_sum: claimed_sum,
        round_polys: round_polys,
        random_challenges,
    }
}

pub fn partial_verify<F: PrimeField>(
    proof: &PartialProof<F>,
    transcript: &mut Transcript<Keccak256, F>,
) -> PartialVerif<F> {
    // transcript.absorb(initial_poly.convert_to_bytes().as_slice());
    transcript.absorb(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut current_claimed_sum = proof.claimed_sum;
    let mut challenges = Vec::with_capacity(proof.round_polys.len());

    for round_poly in &proof.round_polys {
        if round_poly.evaluate(F::from(0)) + round_poly.evaluate(F::from(1)) != current_claimed_sum
        {
            return PartialVerif {
                is_proof_valid: false,
                random_challenges: challenges,
                last_claimed_sum: current_claimed_sum,
            };
        }

        transcript.absorb(round_poly.convert_to_bytes().as_slice());
        let challenge: F = transcript.squeeze();

        current_claimed_sum = round_poly.evaluate(challenge);
        challenges.push(challenge);

        dbg!(current_claimed_sum);
        dbg!(round_poly.evaluate(challenge));
    }

    PartialVerif {
        is_proof_valid: true,
        random_challenges: challenges,
        last_claimed_sum: current_claimed_sum,
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
        let mut prover_transcript = Transcript::<Keccak256, Fq>::init(Keccak256::default());
        let mut verifier_transcript = Transcript::<Keccak256, Fq>::init(Keccak256::default());

        let proof = partial_prove(sum_poly, Fq::from(13), &mut prover_transcript);
        dbg!(&proof);
        let verify = partial_verify(&proof, &mut verifier_transcript);
        assert_eq!(verify.is_proof_valid, true);
    }
}
