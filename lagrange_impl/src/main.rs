use ark_ff::PrimeField;
use rand::seq::SliceRandom;
use std::cmp::max;
use ark_std::test_rng;  

#[derive(Debug, Clone)]
struct Polynomial<F: PrimeField> {
    coefficients: Vec<F>, //ascending degree
}

impl<F: PrimeField> Polynomial<F> {
    fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    fn evaluate(&self, x: F) -> F {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.pow([i as u64]))
            .sum()
    }
    fn degree(&self) -> usize {
        self.coefficients
        .iter()
        .rposition(|&coeff| coeff != F::zero())
        .unwrap_or(0)
    }

    fn add_polynomials(a: Vec<F>, b: Vec<F>) -> Vec<F> {
        let max_len = max(a.len(), b.len());
        let mut result = vec![F::zero(); max_len];
        for i in 0..a.len() {
            result[i] += a[i];
        }
        for i in 0..b.len() {
            result[i] += b[i];
        }
        result
    }

    fn multiply_polynomials(a: Vec<F>, b: Vec<F>) -> Vec<F> {
        let mut result = vec![F::one(); a.len() + b.len() - 1];
        for i in 0..a.len() {
            for j in 0..b.len() {
                result[i + j] += a[i] * b[j];
            }
        }
        result
    }

    fn scale_polynomial(p: Vec<F>, scalar: F) -> Vec<F> {
        p.into_iter().map(|coef| coef * scalar).collect()
    }

    fn interpolate(points: Vec<(F, F)>) -> Self {
        let mut result = vec![F::zero(); points.len()];
        for (i, &(x_i, y_i)) in points.iter().enumerate() {
            let mut l_i = vec![F::one()];
            for (j, &(x_j, _)) in points.iter().enumerate() {
                if i != j {
                    l_i = Self::multiply_polynomials(l_i, vec![-x_j, F::one()]);
                    let denom = x_i - x_j;
                    l_i = Self::scale_polynomial(l_i, denom.inverse().unwrap());
                }
            }
            let l_i = Self::scale_polynomial(l_i, y_i);
            result = Self::add_polynomials(result, l_i);
        }
        Self::new(result)
    }
}

fn generate_shares<F: PrimeField>(secret: F, n: usize, threshold: usize) -> Vec<(F, F)> {
    assert!(threshold <= n, "Threshold must be <= n");

    // Generate random coefficients for the polynomial
    let mut coefficients = vec![secret];
    let mut rng = test_rng();
    for _ in 1..threshold {
        coefficients.push(F::rand(&mut rng)); // Random coefficients in the field
    }

    let polynomial = Polynomial::new(coefficients);
    // print!("Polynomial: {:?}, Degree: {} ", polynomial.coefficients, polynomial.degree());

    // Evaluate the polynomial at different x values
    let mut shares = Vec::new();
    let mut x_values: Vec<i32> = (1..=100).collect(); // Assuming the field has at least 100 elements
    x_values.shuffle(&mut rng);
    for x in x_values.iter().take(n) {
        shares.push((F::from(*x), polynomial.evaluate(F::from(*x))));
    }

    shares
}

fn main() {
    use ark_bn254::Fq;
    // use ark_std::{rand::Rng, test_rng}; 

    let secret = Fq::from(25);
    let n = 10;
    let threshold = 4;

    let shares = generate_shares(secret, n, threshold);
    println!("Shares: {:?}", shares);

    // Randomly select threshold number of shares
    let mut rng = test_rng();
    let selected_shares: Vec<(Fq, Fq)> = shares
        .choose_multiple(&mut rng, threshold)
        .cloned()
        .collect();

    // Reconstruct the secret
    let polynomial = Polynomial::interpolate(selected_shares);
    println!("Degree of polynomial: {}", polynomial.degree());
    if (polynomial.degree() as usize) != threshold - 1 {
        println!("Degree incorrect");
        return;
    } else {
        print!("Polynomial: {:?} ", polynomial.coefficients);
    }

    if polynomial.evaluate(Fq::from(0)) == secret {
        println!("Secret successfully reconstructed!");
    } else {
        println!("Failed to reconstruct secret!");
    }

    // let p = Polynomial::new(vec![Fq::from(2), Fq::from(4), Fq::from(6), Fq::from(8)]); // 1 + 2x + 3x^2
    // println!("P: {:?}", p);
    // println!("P(2): {}", p.evaluate(Fq::from(2)));
    // println!("Degree of P: {}", p.degree());
}
