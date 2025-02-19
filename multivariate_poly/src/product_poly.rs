use ark_ff::PrimeField;
use crate::{partial_evaluate, MultilinearPolynomial};

#[derive(Debug)]
pub struct ProductPoly<F: PrimeField> {
    pub poly_coefficients: Vec<MultilinearPolynomial<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    pub fn new(poly_coefficients: Vec<MultilinearPolynomial<F>>,) -> Self {
        Self {
            poly_coefficients,
        }
    }

    pub fn degree(&self) -> usize {
        self.poly_coefficients[0].dimension()
    }

    pub fn evaluate(&mut self, eval_points: &Vec<F>) -> F {
        let mut result = F::one();
        for poly in self.poly_coefficients.iter() {
            result *= poly.evaluate(&eval_points);
        }
        result
    }

   pub fn partial_evaluate(&mut self, index: usize, eval_point: F) -> MultilinearPolynomial<F> {
    let partials: Vec<_> = self.poly_coefficients
        .iter()
        .flat_map(|poly| {
            let partial_evals = partial_evaluate(poly.coefficients.clone(), index, eval_point);
            MultilinearPolynomial::new(partial_evals).coefficients
        })
        .collect();

        MultilinearPolynomial::new(partials)
    }

    pub fn sum_reduce(&mut self) -> MultilinearPolynomial<F> {
        let mut new_poly = Vec::new();
        let first_poly = &self.poly_coefficients[0].coefficients;
        let second_poly = &self.poly_coefficients[1].coefficients;
        for i in 0..1<<self.degree() {
            new_poly.push(first_poly[i] + second_poly[i]);
        }
        MultilinearPolynomial::new(new_poly)
    }

    pub fn product_reduce(&mut self) -> MultilinearPolynomial<F> {
        let mut new_poly = Vec::new();
        let first_poly = &self.poly_coefficients[0].coefficients;
        let second_poly = &self.poly_coefficients[1].coefficients;
        for i in 0..first_poly.len() {
            new_poly.push(first_poly[i] * second_poly[i]);
        }
        MultilinearPolynomial::new(new_poly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

   fn to_field(inputs: Vec<u64>) -> Vec<Fq> {
        inputs.iter().map(|x| Fq::from(*x)).collect()
    }

    #[test]
    fn test_product_poly_evaluate() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
        dbg!(&product_poly);
        let eval_points = to_field(vec![5, 2]);
        assert_eq!(product_poly.evaluate(&eval_points), Fq::from(1156));
    }

    #[test]
    fn test_product_poly_partial_evaluate() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
        let eval_point = Fq::from(5);
        let partials = product_poly.partial_evaluate(0, eval_point);
        assert_eq!(partials.coefficients, to_field(vec![0, 17, 0, 17]));
        // dbg!(partials);
    }


    #[test]
    fn test_product_poly_sum_reduce() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
        let reduced_sum_poly = product_poly.sum_reduce();
        assert_eq!(reduced_sum_poly.coefficients, to_field(vec![0, 4, 0, 10]));
        // dbg!(reduced_sum_poly);
    }

    #[test]
    fn test_product_poly_product_reduce() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
        let reduced_product_poly = product_poly.product_reduce();
        assert_eq!(reduced_product_poly.coefficients, to_field(vec![0, 4, 0, 25]));
        // dbg!(reduced_product_poly);
    }

    #[test]
    fn test_degree() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let product_poly = ProductPoly::new(vec![poly1, poly2]);
        assert_eq!(product_poly.degree(), 2);
    }
}