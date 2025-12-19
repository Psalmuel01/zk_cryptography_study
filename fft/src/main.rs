use std::f64::consts::PI;

// Complex number struct (simple version)
#[derive(Clone, Copy, Debug)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    fn add(self, other: Self) -> Self {
        Complex::new(self.re + other.re, self.im + other.im)
    }

    fn sub(self, other: Self) -> Self {
        Complex::new(self.re - other.re, self.im - other.im)
    }

    fn mul(self, other: Self) -> Self {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

// Recursive FFT from scratch!
fn fft(a: &[Complex], inverse: bool) -> Vec<Complex> {
    let n = a.len();
    if n <= 1 {
        return a.to_vec();
    }

    // Split into even and odd
    let even: Vec<Complex> = a.iter().step_by(2).copied().collect();
    let odd: Vec<Complex> = a.iter().skip(1).step_by(2).copied().collect();

    let even_fft = fft(&even, inverse);
    let odd_fft = fft(&odd, inverse);

    // Twiddle factors
    let sign = if inverse { 1.0 } else { -1.0 };
    let mut wk = vec![Complex::new(0.0, 0.0); n / 2];
    for k in 0..n/2 {
        let angle = sign * 2.0 * PI * k as f64 / n as f64;
        wk[k] = Complex::new(angle.cos(), angle.sin());
    }

    // Combine with butterfly magic
    let mut result = vec![Complex::new(0.0, 0.0); n];
    for k in 0..n/2 {
        let temp = wk[k].mul(odd_fft[k]);
        result[k] = even_fft[k].add(temp);
        result[k + n/2] = even_fft[k].sub(temp);
    }

    result
}

fn multiply_polynomials(p1: &[f64], p2: &[f64]) -> Vec<f64> {
    let deg1 = p1.len().saturating_sub(1);
    let deg2 = p2.len().saturating_sub(1);
    let product_deg = deg1 + deg2;
    let mut n = 1;
    while n <= product_deg {
        n *= 2;
    }

    // Pad to power of 2 with zeros
    let mut a: Vec<Complex> = p1.iter().map(|&x| Complex::new(x, 0.0)).collect();
    a.resize(n, Complex::new(0.0, 0.0));

    let mut b: Vec<Complex> = p2.iter().map(|&x| Complex::new(x, 0.0)).collect();
    b.resize(n, Complex::new(0.0, 0.0));

    // Forward FFT
    let a_fft = fft(&a, false);
    let b_fft = fft(&b, false);

    // Point-wise multiply
    let mut prod_fft = vec![Complex::new(0.0, 0.0); n];
    for i in 0..n {
        prod_fft[i] = a_fft[i].mul(b_fft[i]);
    }

    // Inverse FFT
    let prod = fft(&prod_fft, true);

    // Normalize and round to integers
    let mut result: Vec<f64> = prod.iter()
        .map(|c| (c.re / n as f64).round())
        .collect();

    // Trim trailing zeros
    while result.len() > 1 && result.last().unwrap().abs() < 1e-6 {
        result.pop();
    }

    result
}

fn main() {
    let poly1 = vec![1.0, 2.0, 3.0]; // 1 + 2x + 3x²
    let poly2 = vec![4.0, 5.0];      // 4 + 5x

    let product = multiply_polynomials(&poly1, &poly2);

    println!("Product: {:?}", product);
    // Prints: [4.0, 13.0, 22.0, 15.0]  Yay!
}