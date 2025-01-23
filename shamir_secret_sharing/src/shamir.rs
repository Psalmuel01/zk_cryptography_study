use ark_ff::PrimeField;
use ark_std::test_rng;
use lagrange_impl::Polynomial;
use rand::seq::SliceRandom;

fn generate_shares<F: PrimeField>(secret: F, n: usize, threshold: usize) -> Vec<(F, F)> {
    assert!(threshold <= n, "Threshold must be <= n");

    // Generate random coefficients for the polynomial
    let mut coefficients = vec![secret];
    let mut rng = test_rng();
    for _ in 1..threshold {
        coefficients.push(F::rand(&mut rng)); // Random coefficients in the field
    }

    let polynomial = Polynomial::new(coefficients);

    // Evaluate the polynomial at different x values
    let mut shares = Vec::new();
    let mut x_values: Vec<i32> = (1..=100).collect(); // Assuming the field has at least 100 elements
    x_values.shuffle(&mut rng);
    for x in x_values.iter().take(n) {
        shares.push((F::from(*x), polynomial.evaluate(F::from(*x))));
    }

    shares
}

fn reconstruct_secret<F: PrimeField>(shares: &[(F, F)], threshold: usize) -> F {
    let points = &shares[..threshold];
    let polynomial = Polynomial::interpolate(points.to_vec());
    print!("polyyyyyy: {:?}", polynomial);
    polynomial.evaluate(F::zero())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_generate_shares() {
        let secret = Fq::from(25);
        let n = 10;
        let threshold = 4;

        let shares = generate_shares(secret, n, threshold);
        assert_eq!(shares.len(), n);
    }

    #[test]
    fn test_recover_secret() {
        let secret = Fq::from(25);
        let n = 10;
        let threshold = 4;

        let shares = generate_shares(secret, n, threshold);
        let selected_shares: Vec<(Fq, Fq)> = shares
            .choose_multiple(&mut test_rng(), threshold)
            .cloned()
            .collect();

        let polynomial = Polynomial::interpolate(selected_shares);
        let recovered_secret = polynomial.evaluate(Fq::from(0));
        assert_eq!(recovered_secret, secret);
    }
}
