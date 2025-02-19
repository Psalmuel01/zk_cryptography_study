use ark_ff::PrimeField;

use crate::product_poly::ProductPoly;

#[derive(Debug)]
pub struct SumPoly<F: PrimeField> {
    pub product_polys: Vec<ProductPoly<F>>,
}

impl <F: PrimeField> SumPoly<F> {
    pub fn new(product_polys: Vec<ProductPoly<F>>) -> Self {
        Self {
            product_polys,
        }
    }

    pub fn degree(&self) -> usize {
        self.product_polys[0].degree()
    }

    pub fn evaluate(&mut self, eval_points: Vec<F>) -> F {
        let mut result = F::zero();
        for poly in self.product_polys.iter_mut() {
            result += poly.evaluate(&eval_points);
        }
        result
    }

    // pub fn partial_evaluate(&mut self, index: usize, eval_point: F) -> ProductPoly<F> {
    //     let partials= self.product_polys
    //         .iter()
    //         .flat_map(|poly| {
    //             let partial_evals = poly.partial_evaluate(index, eval_point);
    //             partial_evals.coefficients
    //         })
    //         .collect();

    //     ProductPoly::new(partials)
    // }
    
}

#[cfg(test)]
mod tests {
    use ark_bn254::Fq;
    use crate::MultilinearPolynomial;
    use super::*;

    fn to_field(inputs: Vec<u64>) -> Vec<Fq> {
        inputs.iter().map(|x| Fq::from(*x)).collect()
    }

    #[test]
    fn test_sum_poly() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![1, 2, 3]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![4, 5, 6]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![7, 8, 9]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![10, 11, 12]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2:  ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        dbg!(sum_poly);
        // assert_eq!(sum_poly.degree(), 0);
    }

    #[test]
    fn test_sum_poly_evaluate() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2:  ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let mut sum_poly = SumPoly::new(vec![poly1, poly2]);
        let eval_points = to_field(vec![5, 2]);
        assert_eq!(sum_poly.evaluate(eval_points), Fq::from(78));
    }
}