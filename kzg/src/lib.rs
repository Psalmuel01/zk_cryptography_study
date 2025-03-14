pub mod trusted_setup;

use ark_ec::{
    PrimeGroup,
    pairing::{Pairing, PairingOutput},
};
use ark_ff::{AdditiveGroup, PrimeField, Zero};
use multivariate_poly::MultilinearPolynomial;
use trusted_setup::TrustedSetup;

pub struct KZG<F: PrimeField, P: Pairing> {
    pub poly: MultilinearPolynomial<F>,
    pub setup: TrustedSetup<P>,
}

#[derive(Debug)]
pub struct KZGProof<F: PrimeField, P: Pairing> {
    pub commitment: P::G1,
    pub poly_opened: F,
    pub quotient_evals: Vec<P::G1>,
}

impl<F: PrimeField, P: Pairing> KZG<F, P> {
    pub fn init(poly: MultilinearPolynomial<F>, setup: TrustedSetup<P>) -> Self {
        Self { poly, setup }
    }

    pub fn prove(&self, open_vals: &Vec<F>) -> KZGProof<F, P> {
        // commit poly
        let commitment = commit::<F, P>(&self.setup.g1_taus, &self.poly.coefficients);

        // open poly
        let v = self.poly.evaluate(&open_vals);

        // compute poly minus v
        let poly_minus_v = self
            .poly
            .coefficients
            .iter()
            .map(|coeff| *coeff - v)
            .collect();
        let sub_poly = MultilinearPolynomial::new(poly_minus_v);

        let mut quotient_evals = Vec::with_capacity(open_vals.len());

        for i in 0..open_vals.len() {
            let quotient_poly = compute_quotient(&sub_poly);
            let blown_quotient_poly = blow_up(quotient_poly, i + 1);

            let quotient_eval = self
                .setup
                .g1_taus
                .iter()
                .zip(blown_quotient_poly.coefficients.iter())
                .map(|(g1_taus, coeffs)| g1_taus.mul_bigint(coeffs.into_bigint()))
                .sum();
            quotient_evals.push(quotient_eval);
        }

        KZGProof {
            commitment,
            poly_opened: v,
            quotient_evals,
        }
    }

    pub fn verify(&self, proof: KZGProof<F, P>, open_vals: &Vec<F>) -> bool {
        let g1_generator = P::G1::generator();
        let g2_generator = P::G2::generator();

        // pairing(g1_(f(τ) - v), g2_1) == pairing(Σ(g1_Q(τ), g2_(τ - a)))

        let lhs = P::pairing(
            proof.commitment - g1_generator.mul_bigint(proof.poly_opened.into_bigint()),
            g2_generator,
        );

        let mut rhs = PairingOutput::ZERO;
        for (i, tau) in self.setup.g2_taus.iter().enumerate() {
            rhs += P::pairing(
                proof.quotient_evals[i],
                *tau - g2_generator.mul_bigint(open_vals[i].into_bigint()),
            );
        }

        dbg!(&lhs);
        dbg!(&rhs);

        lhs == rhs
    }
}

pub fn commit<F: PrimeField, P: Pairing>(g1_taus: &Vec<<P>::G1>, poly_coeffs: &Vec<F>) -> P::G1 {
    let mut commitment = P::G1::zero();

    for (i, g1_tau) in g1_taus.iter().enumerate() {
        commitment += g1_tau.mul_bigint(poly_coeffs[i].into_bigint());
    }

    commitment
}

pub fn compute_quotient<F: PrimeField>(
    poly: &MultilinearPolynomial<F>,
) -> MultilinearPolynomial<F> {
    let mid = poly.coefficients.len() / 2;
    let (eval_zero, eval_one) = poly.coefficients.split_at(mid);
    // Q(x) = arr of y_2 - arr of y_1
    let quotient = eval_one
        .iter()
        .zip(eval_zero.iter())
        .map(|(eval_one, eval_zero)| *eval_one - *eval_zero)
        .collect();
    // dbg!(&quotient);
    MultilinearPolynomial::new(quotient)
}

pub fn blow_up<F: PrimeField>(
    poly: MultilinearPolynomial<F>,
    blow_up_times: usize,
) -> MultilinearPolynomial<F> {
    assert!(
        poly.coefficients.len().is_power_of_two(),
        "Polynomial must have a power of two"
    );
    let mut blown_coeffs = poly.coefficients;

    for _ in 0..blow_up_times {
        blown_coeffs = extend(blown_coeffs)
    }

    MultilinearPolynomial::new(blown_coeffs)
}

pub fn extend<F: PrimeField>(coeffs: Vec<F>) -> Vec<F> {
    let n_bits = coeffs.len();
    let total_combinations = 1 << n_bits;
    let mut blown_coeffs = Vec::with_capacity(total_combinations);
    for _ in 0..2 {
        blown_coeffs.extend(coeffs.iter());
    }
    blown_coeffs
}

#[cfg(test)]

pub mod tests {
    use super::*;
    use ark_bls12_381::{Bls12_381, Fr};

    #[test]
    fn test_kzg() {
        let taus = vec![Fr::from(5), Fr::from(2), Fr::from(3)];
        let setup = TrustedSetup::<Bls12_381>::initialize(&taus);
        let values = vec![
            Fr::from(0),
            Fr::from(4),
            Fr::from(0),
            Fr::from(4),
            Fr::from(0),
            Fr::from(4),
            Fr::from(3),
            Fr::from(7),
        ];
        let poly = MultilinearPolynomial::new(values);
        let kzg = KZG::init(poly, setup);

        let open_vals = vec![Fr::from(6), Fr::from(4), Fr::from(0)];
        let proof = kzg.prove(&open_vals);
        dbg!(&proof);

        let verify = kzg.verify(proof, &open_vals);
        dbg!(verify);

        assert_eq!(verify, true);
    }
}
