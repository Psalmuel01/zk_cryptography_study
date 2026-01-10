use ark_ff::FftField;
use std::marker::PhantomData;

/// Polynomial operations using Fast Fourier Transform over finite fields.
/// This provides efficient conversion between coefficient and evaluation
/// representations of polynomials, which is crucial for cryptographic operations
/// like polynomial commitments and zero-knowledge proofs.
pub struct PolynomialFFT<F: FftField> {
    _field: PhantomData<F>,
}

impl<F: FftField> PolynomialFFT<F> {
    fn split_even_odd(sequence: &[F]) -> (Vec<F>, Vec<F>) {
        let even_indexed: Vec<F> = sequence
            .iter()
            .step_by(2)
            .copied()
            .collect();

        let odd_indexed: Vec<F> = sequence
            .iter()
            .skip(1)
            .step_by(2)
            .copied()
            .collect();

        (even_indexed, odd_indexed)
    }

    /// Performs the Cooley-Tukey FFT algorithm over a finite field.
    /// * `inverse` - If true, performs inverse FFT (IFFT)

    fn cooley_tukey_fft(sequence: &[F], inverse: bool) -> Vec<F> {
        let n = sequence.len();

        if n == 0 || !n.is_power_of_two() {
            panic!("FFT input length must be a power of 2 and non-zero");
        }

        // Base case: single element is already transformed
        if n == 1 {
            return sequence.to_vec();
        }

        // Divide: split into even and odd subsequences
        let (even_seq, odd_seq) = Self::split_even_odd(sequence);

        // Conquer: recursively compute FFT of both halves
        let even_fft = Self::cooley_tukey_fft(&even_seq, inverse);
        let odd_fft = Self::cooley_tukey_fft(&odd_seq, inverse);

        // Get the primitive nth root of unity in the field
        let root_of_unity = F::get_root_of_unity(n as u64)
            .expect("Field must support nth root of unity for FFT");

        // For inverse FFT, use ω⁻¹ instead of ω
        let omega = if inverse {
            root_of_unity
                .inverse()
                .expect("Root of unity must be invertible")
        } else {
            root_of_unity
        };

        // Combine: butterfly operations to merge results
        let mut result = vec![F::zero(); n];
        let half_n = n / 2;

        for k in 0..half_n {
            // Twiddle factor: ωᵏ
            let twiddle_factor = omega.pow([k as u64]);

            // Butterfly operation:
            // y[k] = even[k] + ωᵏ · odd[k]
            // y[k + n/2] = even[k] - ωᵏ · odd[k]
            let twiddle_product = twiddle_factor * odd_fft[k];

            result[k] = even_fft[k] + twiddle_product;
            result[k + half_n] = even_fft[k] - twiddle_product;
        }

        result
    }

    /// Converts polynomial from coefficient representation to evaluation representation.
    /// Given polynomial p(x) = c₀ + c₁x + c₂x² + ... + cₙ₋₁xⁿ⁻¹,
    /// computes evaluations at the nth roots of unity: [p(ω⁰), p(ω¹), ..., p(ωⁿ⁻¹)]
    pub fn forward_fft(coefficients: &[F]) -> Vec<F> {
        Self::cooley_tukey_fft(coefficients, false)
    }

    /// Converts polynomial from evaluation representation back to coefficient representation.
    /// Given evaluations [p(ω⁰), p(ω¹), ..., p(ωⁿ⁻¹)],
    /// recovers the coefficients [c₀, c₁, c₂, ..., cₙ₋₁]

    pub fn inverse_fft(evaluations: &[F]) -> Vec<F> {
        let n = evaluations.len();
        Self::cooley_tukey_fft(evaluations, true)
            .iter()
            .map(|&coefficient| coefficient / F::from(n as u64))
            .collect()
    }

    /// Multiplies two polynomials efficiently using FFT.
    /// This is much faster than naive O(n²) multiplication for large polynomials.
    /// # Time Complexity -> O(n log n) vs O(n²) for naive multiplication
    pub fn multiply_polynomials(poly_a: &[F], poly_b: &[F]) -> Vec<F> {
        if poly_a.is_empty() || poly_b.is_empty() {
            return vec![];
        }

        // Handle zero polynomials
        let is_zero_a = poly_a.iter().all(|&x| x == F::zero());
        let is_zero_b = poly_b.iter().all(|&x| x == F::zero());
        if is_zero_a || is_zero_b {
            return vec![F::zero()];
        }

        // Result degree is deg(a) + deg(b)
        let result_length = poly_a.len() + poly_b.len() - 1;

        // Find next power of 2 for efficient FFT
        let fft_size = result_length.next_power_of_two();
        dbg!(fft_size);

        // Pad polynomials with zeros
        let mut padded_a = poly_a.to_vec();
        let mut padded_b = poly_b.to_vec();
        padded_a.resize(fft_size, F::zero());
        padded_b.resize(fft_size, F::zero());

        // Transform to evaluation domain
        let evals_a = Self::forward_fft(&padded_a);
        let evals_b = Self::forward_fft(&padded_b);

        // Point-wise multiplication (O(n) instead of O(n²))
        let product_evals: Vec<F> = evals_a
            .iter()
            .zip(evals_b.iter())
            .map(|(&a, &b)| a * b)
            .collect();

        // Transform back to coefficient domain
        let result = Self::inverse_fft(&product_evals);
        // result.truncate(result_length);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_377::Fr;

    #[test]
    fn test_fft_round_trip() {
        // Polynomial: 4 + 5x
        let coefficients = vec![
            Fr::from(4),
            Fr::from(5),
            Fr::from(0),
            Fr::from(0),
        ];

        // Forward FFT: coefficients -> evaluations
        let evaluations = PolynomialFFT::forward_fft(&coefficients);

        // Inverse FFT: evaluations -> coefficients
        let recovered = PolynomialFFT::inverse_fft(&evaluations);

        assert_eq!(recovered, coefficients, "Round-trip FFT failed");
    }

    #[test]
    fn test_polynomial_multiplication() {
        // p(x) = 1 + 2x + 3x²
        let poly_a = vec![Fr::from(1), Fr::from(2), Fr::from(3)];

        // q(x) = 4 + 5x
        let poly_b = vec![Fr::from(4), Fr::from(5)];

        // Expected: (1 + 2x + 3x²)(4 + 5x) = 4 + 13x + 22x² + 15x³
        let expected = vec![
            Fr::from(4),
            Fr::from(13),
            Fr::from(22),
            Fr::from(15),
        ];

        let product = PolynomialFFT::multiply_polynomials(&poly_a, &poly_b);

        assert_eq!(product, expected, "Polynomial multiplication failed");
    }

    #[test]
    fn test_identity_polynomial() {
        // p(x) = x
        let identity = vec![Fr::from(0), Fr::from(1), Fr::from(0), Fr::from(0)];

        let evaluations = PolynomialFFT::forward_fft(&identity);
        let recovered = PolynomialFFT::inverse_fft(&evaluations);

        assert_eq!(recovered, identity);
    }

    #[test]
    fn test_constant_polynomial() {
        // p(x) = 7
        let constant = vec![Fr::from(7), Fr::from(0), Fr::from(0), Fr::from(0)];

        let evaluations = PolynomialFFT::forward_fft(&constant);

        // Constant polynomial evaluates to 7 at all points
        for &eval in &evaluations {
            assert_eq!(eval, Fr::from(7));
        }
    }

    #[test]
    fn test_multiply_by_zero() {
        let poly = vec![Fr::from(1), Fr::from(2), Fr::from(3)];
        let zero = vec![Fr::from(0)];

        let product = PolynomialFFT::multiply_polynomials(&poly, &zero);

        assert_eq!(product, vec![Fr::from(0)], "Multiplying by zero should give zero");
    }
}