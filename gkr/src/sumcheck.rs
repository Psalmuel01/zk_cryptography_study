use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::{product_poly::ProductPoly, sum_poly::SumPoly, MultilinearPolynomial};
use sum_check::transcript::Transcript;
use sha3::Keccak256;

#[derive(Debug, Clone)]
pub struct SumCheckProof<F: PrimeField> {
    pub claimed_sum: F,
    pub round_polys: Vec<[F; 3]>,
}

#[derive(Debug, Clone)]
pub struct GKRProver<F: PrimeField> {
    pub initial_poly: SumPoly<F>,
    pub claimed_sum: F,
    pub transcripts: Transcript<Keccak256, F>,
}

impl<F: PrimeField> GKRProver<F> {
    pub fn new(poly_eval_points: Vec<ProductPoly<F>>, claimed_sum: F) -> Self {
        let poly = SumPoly::new(poly_eval_points);
        Self {
            initial_poly: poly,
            claimed_sum: claimed_sum,
            transcripts: Transcript::init(Keccak256::default()),
        }
    }

    pub fn prove(&mut self) -> SumCheckProof<F> {
        let mut round_polys = Vec::new();

        // append poly eval coefficients
        self.transcripts
            .absorb(self.initial_poly.convert_to_bytes().as_slice());
        self.transcripts
            .absorb(self.claimed_sum.into_bigint().to_bytes_be().as_slice());

        let mut poly = self.initial_poly.clone();

        for _ in 0..self.initial_poly.degree() {
            let round_poly_coeffs = split_and_sum(poly.clone());
            let round_poly = MultilinearPolynomial::new(round_poly_coeffs.to_vec());
            self.transcripts
                .absorb(round_poly.convert_to_bytes().as_slice());
            round_polys.push(round_poly_coeffs);

            let challenge: F = self.transcripts.squeeze();
            // println!("challenge_prover: {}", challenge);
            poly = poly.partial_evaluate(0, challenge);
        }

        // println!("prover_round_poly: {:?}", round_polys);
        // dbg!(self.claimed_sum);
        // dbg!(&round_polys);

        SumCheckProof {
            claimed_sum: self.claimed_sum,
            round_polys: round_polys,
        }
    }
}

fn split_and_sum<F: PrimeField>(mut poly: SumPoly<F>) -> [F; 3] {
    let length = poly.product_polys[0].degree() + 1;
    // print!("degree: {}", length);

    let mut res_vec = SumPoly::new(vec![]);

    for i in 0..length {
        let mut res = poly.partial_evaluate(0, F::from(i as u64));
        let reduced = res.sum_reduce();
        res_vec.add_polynomial(reduced);
    }

    let mut result = [F::zero(); 3];
    for (i, product_poly) in res_vec.product_polys.iter().enumerate() {
        let sum = product_poly.poly_coefficients[0].coefficients.iter().fold(F::zero(), |acc, x| acc + x);
        result[i] = sum;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;
    use sum_check::prover;

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
        println!("{:?}", result);
    }

    #[test]
    fn test_sumcheck() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        let mut prover = GKRProver::new(sum_poly.product_polys, Fq::from(2312));
        let proof = prover.prove();
        println!("{:?}", proof);
    }
}