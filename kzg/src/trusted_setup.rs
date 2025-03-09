use ark_ec::{PrimeGroup, pairing::Pairing};
use ark_ff::PrimeField;

#[derive(Debug)]
pub struct TrustedSetup<P: Pairing> {
    pub size: usize,
    pub g1_powers: Vec<P::G1>,
    pub g2_powers: Vec<P::G2>,
}

pub fn initialize<F: PrimeField, P: Pairing>(taus: Vec<F>) -> TrustedSetup<P> {
    let size = taus.len();

    let g1_generator = P::G1::generator();
    let g2_generator = P::G2::generator();

    let lagrange_basis = compute_lagrange_basis(taus);

    let g1_powers = lagrange_basis
        .iter()
        .map(|tau| g1_generator.mul_bigint(tau.into_bigint()))
        .collect();
    let g2_powers = lagrange_basis
        .iter()
        .map(|tau| g2_generator.mul_bigint(tau.into_bigint()))
        .collect();

    TrustedSetup {
        size,
        g1_powers,
        g2_powers,
    }
}

pub fn compute_lagrange_basis<F: PrimeField>(taus: Vec<F>) -> Vec<F> {
    let size = taus.len();
    let mut result = Vec::with_capacity(size);

    for i in 0..1 << size {
        let mut product = F::one();
        for (j, tau) in taus.iter().enumerate() {
            let val = if (i & (1 << size - 1 -j)) != 0 {
                *tau
            } else {
                F::one() - tau
            };
            product *= val;
        }
        result.push(product);
    }

    result
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ark_bls12_381::{Bls12_381, Fr};

    pub fn setup() -> TrustedSetup<Bls12_381> {
        let tau_arr = vec![Fr::from(5), Fr::from(2), Fr::from(3)];
        initialize::<Fr, Bls12_381>(tau_arr)
    }

    #[test]
    fn test_compute_lagrange_basis() {
        let tau_arr = vec![Fr::from(5), Fr::from(2), Fr::from(3)];
        let result = compute_lagrange_basis(tau_arr);
        let expected = vec![
            Fr::from(-8),
            Fr::from(12),
            Fr::from(16),
            Fr::from(-24),
            Fr::from(10),
            Fr::from(-15),
            Fr::from(-20),
            Fr::from(30),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_initialize() {
        let tau_arr = vec![Fr::from(5), Fr::from(2), Fr::from(3)];
        let result = initialize::<Fr, Bls12_381>(tau_arr);
        dbg!(&result);
    }
}
