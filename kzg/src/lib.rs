pub mod helpers;
pub mod trusted_setup;

use ark_ec::{PrimeGroup, pairing::Pairing};
use ark_ff::{PrimeField, Zero};
use multivariate_poly::MultilinearPolynomial;
use trusted_setup::TrustedSetup;

pub struct KZGProof {}

pub fn commit<F: PrimeField, P: Pairing>(
    poly: MultilinearPolynomial<F>,
    g1_taus: Vec<P::G1>,
) -> P::G1 {
    let mut commitment = P::G1::zero();

    for (i, g1_tau) in g1_taus.iter().enumerate() {
        commitment += g1_tau.mul_bigint(poly.coefficients[i].into_bigint());
    }

    commitment
}

pub fn open_poly<F: PrimeField>(poly: MultilinearPolynomial<F>, evals: Vec<F>) -> F {
    poly.evaluate(&evals)
}

pub fn prove<F: PrimeField, P: Pairing>(
    poly: MultilinearPolynomial<F>,
    trusted_setup: TrustedSetup<P>,
) -> KZGProof {
    KZGProof {}
}

#[cfg(test)]

pub mod tests {
    use super::*;
    use ark_bls12_381::{Bls12_381, Fr};

    #[test]
    fn test_commit() {
        let taus = vec![Fr::from(5), Fr::from(2), Fr::from(3)];
        let setup = TrustedSetup::<Bls12_381>::initialize(&taus);
        // dbg!(&setup);
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
        let commitment = commit::<Fr, Bls12_381>(poly, setup.g1_taus);
        dbg!(&commitment);
        // let evaluation = open_poly(poly, taus);
        // dbg!(evaluation);
    }
}
