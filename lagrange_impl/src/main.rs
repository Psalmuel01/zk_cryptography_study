#[derive(Debug, Clone)]
struct Polynomial {
    coefficients: Vec<f64>, //ascending degree
}

impl Polynomial {
    fn new(coefficients: Vec<f64>) -> Self {
        Self { coefficients }
    }
    
    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .enumerate()
            .map(|(i, &coef)| coef * x.powf(i as f64))
            .sum()
    }

    fn degree(&self) -> usize {
        self.coefficients.len() - 1
    }

    fn add_polynomials(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
        let max_len = a.len().max(b.len());
        let mut result = vec![0.0_f64; max_len];
        for i in 0..a.len() {
            result[i] += a[i];
        }
        for i in 0..b.len() {
            result[i] += b[i];
        }
        result
    }

    fn multiply_polynomials(a: Vec<f64>, b: Vec<f64>) -> Vec<f64> {
        let mut result = vec![0.0_f64; a.len() + b.len() - 1];
        for i in 0..a.len() {
            for j in 0..b.len() {
                result[i + j] += a[i] * b[j];
            }
        }
        result
    }

    fn scale_polynomial(p: Vec<f64>, scalar: f64) -> Vec<f64> {
        p.into_iter().map(|coef| coef * scalar).collect()
    }

    fn interpolate(points: Vec<(f64, f64)>) -> Self {
        let n = points.len();
        let mut coefficients = vec![0.0; n];

        // calculate denominators
        let mut denominators = vec![1.0; n];
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    denominators[i] *= points[i].0 - points[j].0;
                }
            }
        }

        // calculate the ith term of the polynomial
        for i in 0..n {
            let mut term = vec![1.0];
            for j in 0..n {
                if i != j {
                    // multiply by (x - x_j)   
                    term = Self::multiply_polynomials(term, vec![-points[j].0, 1.0]);
                }
            }

            // scale by y_i / denominator
            term = Self::scale_polynomial(term, points[i].1 / denominators[i]);

            // add to result
            coefficients = Self::add_polynomials(coefficients, term);
        }

        Self::new(coefficients)

    }
}


fn main() {
    let p = Polynomial::new(vec![1.0, 2.0, 3.0]); // 1 + 2x + 3x^2
   
    println!("P: {:?}", p);
    println!("P(2): {}", p.evaluate(2.0));
    println!("Degree of P: {}", p.degree());

    let points = vec![(0.0, 1.0), (1.0, 6.0), (2.0, 17.0)];
    let q = Polynomial::interpolate(points);
    // println!("Q: {:?}", q);
    println!("Polynomial Coefficients: {:?}", q.coefficients);
}