# ffts

## The Code Structure

### Complex Number Operations

I implemented basic complex arithmetic (add, sub, mul, div).  
This is the foundation needed since FFT works in the complex domain.

### FFT Implementation

The `fft` function is a clean recursive Cooley-Tukey implementation:

- Splits input into even/odd indices
- Recursively computes FFT on both halves
- Combines using twiddle factors (the `wk` array with rotation angles)
- Supports both forward and inverse FFT via the `inverse` flag

## The Cryptography Applications

### 1. Polynomial Multiplication (`multiply_polynomials`)

This is exactly what I mentioned earlier for lattice-based crypto! Here's what it does:  
*(1 + 2x + 3x²) × (4 + 5x) = 4 + 13x + 22x² + 15x³*

The clever trick:

- Instead of O(n²) coefficient-by-coefficient multiplication
- Convert to frequency domain → O(n log n)
- Multiply point-wise → O(n)
- Convert back → O(n log n)
- Total: O(n log n) instead of O(n²)

This is crucial for schemes like:

- NTRU encryption: multiplies polynomials in Z[x]/(x^n - 1)
- Ring-LWE: operations in polynomial rings
- Post-quantum cryptography: many NIST finalists use this

### 2. Polynomial Division (`divide_by_linear`)

Dividing by (x - z) using FFT - this is more advanced!  
*(-6 - x + 2x²) ÷ (x - 2) = 3 + 2x*

The approach:

- Evaluate polynomial at roots of unity via FFT
- Divide each evaluation by (ωᵢ - z)
- Inverse FFT to get quotient coefficients

Use cases:

- Secret sharing schemes: reconstructing secrets from shares
- Error correction codes: syndrome computation
- Polynomial factorization: needed in some cryptanalysis

## Performance Impact

For a degree-1024 polynomial (common in lattice crypto):

- Naive multiplication: ~1 million operations
- FFT multiplication: ~10,000 operations
- Speedup: ~100x faster!

This transforms impractical schemes into deployable systems.