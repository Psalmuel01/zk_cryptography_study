use crate::MultilinearPolynomial;
use ark_ff::PrimeField;

#[derive(Debug, Clone)]
pub struct ProductPoly<F: PrimeField> {
    pub poly_coefficients: Vec<MultilinearPolynomial<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    pub fn new(poly_coefficients: Vec<MultilinearPolynomial<F>>) -> Self {
        assert!(
            poly_coefficients
                .iter()
                .all(|poly| poly.no_of_variables() == poly_coefficients[0].no_of_variables()),
            "All polynomials must be of the same degree"
        );
        Self { poly_coefficients }
    }

    pub fn degree(&self) -> usize {
        self.poly_coefficients.len()
    }

    pub fn evaluate(&mut self, eval_points: &Vec<F>) -> F {
        let mut result = F::one();
        for poly in self.poly_coefficients.iter() {
            result *= poly.evaluate(&eval_points);
        }
        result
    }

    pub fn partial_evaluate(&mut self, index: usize, eval_point: F) -> Self {
        let partials: Vec<_> = self
            .poly_coefficients
            .iter()
            .map(|poly| {
                let coeff = poly.partial_evaluate(index, eval_point).coefficients;
                MultilinearPolynomial::new(coeff)
            })
            .collect();

        Self {
            poly_coefficients: partials,
        }
    }

    // pub fn sum_reduce(&mut self) -> MultilinearPolynomial<F> {
    //     let mut new_poly = Vec::new();
    //     let first_poly = &self.poly_coefficients[0].coefficients;
    //     let second_poly = &self.poly_coefficients[1].coefficients;
    //     for i in 0..1 << self.degree() {
    //         new_poly.push(first_poly[i] + second_poly[i]);
    //     }
    //     MultilinearPolynomial::new(new_poly)
    // }

    pub fn product_reduce(&mut self) -> MultilinearPolynomial<F> {
        assert!(
            self.poly_coefficients.len() > 1,
            "More than one polynomial is required"
        );

        let mut resultant_values = self.poly_coefficients[0].coefficients.clone();
        for poly in self.poly_coefficients.iter().skip(1) {
            for (i, value) in poly.coefficients.iter().enumerate() {
                resultant_values[i] *= value;
            }
        }
        MultilinearPolynomial::new(resultant_values)
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        self.poly_coefficients
            .iter()
            .flat_map(|poly| poly.convert_to_bytes())
            .collect()
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
        println!("{:?}", &product_poly);
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
        // assert_eq!(partials.coefficients, to_field(vec![0, 17, 0, 17]));
        println!("{:?}", partials);
    }

    // #[test]
    // fn test_sum_reduce() {
    //     let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
    //     let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
    //     let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
    //     let reduced_sum_poly = product_poly.sum_reduce();
    //     assert_eq!(reduced_sum_poly.coefficients, to_field(vec![0, 4, 0, 10]));
    //     // println!("{:?}", reduced_sum_poly);
    // }

    #[test]
    fn test_product_reduce() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let mut product_poly = ProductPoly::new(vec![poly1, poly2]);
        let reduced_product_poly = product_poly.product_reduce();
        assert_eq!(
            reduced_product_poly.coefficients,
            to_field(vec![0, 4, 0, 25])
        );
        println!("{:?}", reduced_product_poly);
    }

    #[test]
    fn test_degree() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let product_poly = ProductPoly::new(vec![poly1, poly2]);
        assert_eq!(product_poly.degree(), 2);
    }

    #[test]
    fn test_convert_to_bytes() {
        let poly1 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let poly2 = MultilinearPolynomial::new(to_field(vec![0, 2, 0, 5]));
        let product_poly = ProductPoly::new(vec![poly1, poly2]);
        let bytes = product_poly.convert_to_bytes();
        assert_eq!(bytes.len(), 256);
        // println!("{:?}", bytes);
    }
}
