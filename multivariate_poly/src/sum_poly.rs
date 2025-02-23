use crate::{product_poly::ProductPoly, MultilinearPolynomial};
use ark_ff::PrimeField;

#[derive(Debug, Clone)]
pub struct SumPoly<F: PrimeField> {
    pub product_polys: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(product_polys: Vec<ProductPoly<F>>) -> Self {
        Self { product_polys }
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

    pub fn add_polynomial(&mut self, poly: ProductPoly<F>) {
        self.product_polys.push(poly);
    }

    pub fn add_polynomials(&mut self, polys: Vec<ProductPoly<F>>) {
        self.product_polys.extend(polys);
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        self.product_polys
            .iter()
            .flat_map(|poly| poly.convert_to_bytes())
            .collect()
    }

    pub fn partial_evaluate(&mut self, index: usize, eval_point: F) -> SumPoly<F> {
        let partials = self
            .product_polys
            .iter_mut()
            .map(|poly| {
                let coefficients = poly.partial_evaluate(index, eval_point).poly_coefficients;
                ProductPoly::new(coefficients)
            })
            .collect();

        SumPoly::new(partials)
    }

    pub fn sum_reduce(&mut self) -> ProductPoly<F> {
        let mut polyy = Vec::new();
        let mut new_poly = Vec::new();
        for poly in self.product_polys.iter_mut() {
            let sum_poly = poly.product_reduce();
            new_poly.push(sum_poly);
        }
        let product = ProductPoly::new(new_poly);
        let mut newer_poly = vec![F::zero(); 1 << self.degree()];
        for poly in product.poly_coefficients.iter() {
            for (i, coeff) in poly.coefficients.iter().enumerate() {
                newer_poly[i] += coeff;
            }
        }
        polyy.push(MultilinearPolynomial::new(newer_poly));
        ProductPoly::new(polyy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MultilinearPolynomial;
    use ark_bn254::Fq;

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
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        println!("{:?}", sum_poly);
    }

    #[test]
    fn test_sum_poly_evaluate() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let mut sum_poly = SumPoly::new(vec![poly1, poly2]);
        let eval_points = to_field(vec![5, 2]);
        assert_eq!(sum_poly.evaluate(eval_points), Fq::from(2312));
    }

    #[test]
    fn test_sum_poly_partial_evaluate() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let mut sum_poly = SumPoly::new(vec![poly1, poly2]);
        let eval_point = Fq::from(5);
        let partials = sum_poly.partial_evaluate(0, eval_point);
        println!("{:?}", &partials);
        // assert_eq!(
        //     partials.poly_coefficients[0].coefficients,
        //     to_field(vec![0, 17, 0, 17])
        // );
        // assert_eq!(
        //     partials.poly_coefficients[1].coefficients,
        //     to_field(vec![0, 17, 0, 17])
        // );
    }

    #[test]
    fn test_sum_poly_sum_reduce() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 3, 2, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 4]));
        let mul5 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 4]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 3, 2, 5]));
        let mul6 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 4]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 0, 0, 4]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let poly3: ProductPoly<Fq> = ProductPoly::new(vec![mul5, mul6]);
        let mut sum_poly = SumPoly::new(vec![poly1, poly2]);
        let reduced_sum_poly = sum_poly.sum_reduce();
        // assert_eq!(reduced_sum_poly.poly_coefficients[0].coefficients, to_field(vec![0, 4, 0, 10]));
        println!("{:?}", reduced_sum_poly);
    }

    #[test]
    fn test_sum_poly_convert_to_bytes() {
        let mul1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul3 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mul4 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly1: ProductPoly<Fq> = ProductPoly::new(vec![mul1, mul2]);
        let poly2: ProductPoly<Fq> = ProductPoly::new(vec![mul3, mul4]);
        let sum_poly = SumPoly::new(vec![poly1, poly2]);
        let bytes = sum_poly.convert_to_bytes();
        assert_eq!(bytes.len(), 512);
        // println!("{:?}", bytes);
    }
}
