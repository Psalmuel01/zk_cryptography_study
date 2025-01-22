use ark_ff::PrimeField;
use std::cmp::max;

#[derive(Debug, Clone)]
pub struct Polynomial<F: PrimeField> {
    pub coefficients: Vec<F>, //ascending degree
}

impl<F: PrimeField> Polynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    pub fn evaluate(&self, x: F) -> F {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.pow([i as u64]))
            .sum()
    }
    pub fn degree(&self) -> usize {
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
        let mut result = vec![F::zero(); a.len() + b.len() - 1];
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

    pub fn interpolate(points: Vec<(F, F)>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_evaluate_polynomial() {
        let coefficients = vec![Fq::from(1), Fq::from(2), Fq::from(3)];
        let polynomial = Polynomial::new(coefficients);

        assert_eq!(polynomial.evaluate(Fq::from(0)), Fq::from(1));
        assert_eq!(polynomial.evaluate(Fq::from(1)), Fq::from(6));
        assert_eq!(polynomial.evaluate(Fq::from(2)), Fq::from(17));
    }

    #[test]
    fn test_interpolate_known_points() {
        let points = vec![
            (Fq::from(0), Fq::from(0)),
            (Fq::from(1), Fq::from(2)),
            (Fq::from(2), Fq::from(4)),
            (Fq::from(3), Fq::from(6)),
        ];

        let polynomial = Polynomial::interpolate(points.clone());
        assert_eq!(polynomial.degree(), 1); // Linear polynomial y = 2x
        assert_eq!(polynomial.evaluate(Fq::from(0)), Fq::from(0));
        assert_eq!(polynomial.evaluate(Fq::from(1)), Fq::from(2));
        assert_eq!(polynomial.evaluate(Fq::from(2)), Fq::from(4));
        assert_eq!(polynomial.evaluate(Fq::from(3)), Fq::from(6));

        println!("Known Points: {:?}", points);
        println!("Polynomial Coefficients: {:?}", polynomial.coefficients);
    }
}