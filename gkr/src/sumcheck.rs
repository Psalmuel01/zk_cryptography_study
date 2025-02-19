use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::{sum_poly::SumPoly, product_poly::ProductPoly, MultilinearPolynomial};
use sum_check::transcript::Transcript;
use sha3::Keccak256;

#[derive(Debug, Clone)]
pub struct GKRProof<F: PrimeField> {
    pub claimed_sum: F,
    pub round_polys: Vec<[F; 2]>,
}

#[derive(Debug)]
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

    // pub fn prove(&mut self) -> GKRProof<F> {
    //     let mut round_polys = Vec::new();

    //     // append poly eval coefficients
    //     self.transcripts
    //         .absorb(self.initial_poly.convert_to_bytes().as_slice());
    //     self.transcripts
    //         .absorb(self.claimed_sum.into_bigint().to_bytes_be().as_slice());

    //     let mut poly = self.initial_poly;

    //     for _ in 0..self.initial_poly.degree() {
    //         let round_poly_coeffs = split_and_sum();
    //         let round_poly = MultilinearPolynomial::new(round_poly_coeffs.to_vec());
    //         self.transcripts
    //             .absorb(round_poly.convert_to_bytes().as_slice());
    //         round_polys.push(round_poly_coeffs);

    //         let challenge: F = self.transcripts.squeeze();
    //         // println!("challenge_prover: {}", challenge);
    //         poly = poly.partial_evaluate(0, challenge);
    //     }

    //     // println!("prover_round_poly: {:?}", round_polys);
    //     // dbg!(self.claimed_sum);
    //     // dbg!(&round_polys);

    //     GKRProof {
    //         claimed_sum: self.claimed_sum,
    //         round_polys: round_polys,
    //     }
    // }
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
