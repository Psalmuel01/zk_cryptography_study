use ark_ff::PrimeField;

#[derive(Debug, Clone)]
pub struct MultilinearPolynomial<F: PrimeField> {
    pub terms: Vec<(Vec<usize>, F)>,
}

impl <F: PrimeField> MultilinearPolynomial<F> {
    pub fn new() -> Self {
        Self { terms: Vec::new() }
    }

    pub fn add_term(&mut self, variables: Vec<usize>, coefficient: F) {
        self.terms.push((variables, coefficient));
    }

    pub fn evaluate(&self, values: &[F]) -> F {
        let mut result = F::zero();
        for (variables, coefficient) in &self.terms {
            // Calculate the product of the values corresponding to the variables in this term
            let product: F = variables.iter().map(|&i| values[i]).product();
            result += *coefficient * product;
        }
        result
    }
}



// fn main() {
//     use ark_bn254::Fq;
//     // Create a multilinear polynomial
//     let mut polynomial: MultilinearPolynomial<Fq> = MultilinearPolynomial::new();

//     // Add terms for P(x, y, z) = 3x + 2y + 5xy + 4z + 7xyz
//     polynomial.add_term(vec![0], Fq::from(3));       // 3x
//     polynomial.add_term(vec![1], Fq::from(2));       // 2y
//     polynomial.add_term(vec![0, 1], Fq::from(5));    // 5xy
//     polynomial.add_term(vec![2], Fq::from(4));       // 4z
//     polynomial.add_term(vec![0, 1, 2], Fq::from(7)); // 7xyz

//     println!("Polynomial: {:?}", polynomial);

//     // Evaluate the polynomial for (1, 2, 3)
//     let values = vec![Fq::from(1), Fq::from(2), Fq::from(3)];
//     let result = polynomial.evaluate(&values);
//     println!("Result: {}", result);
// }