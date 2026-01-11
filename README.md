# Zero-Knowledge Proofs – Study & Implementations

This repository contains my ongoing work studying and implementing the core mathematical and cryptographic building blocks behind modern zero-knowledge proof systems (SNARKs, PLONK-style systems, and related protocols).

The focus is on **first-principles implementations**: understanding the algebra and protocols deeply rather than relying on black-box libraries.

The code here is exploratory, educational, and evolves as my understanding improves.

---

## Repository Structure

Each directory focuses on a specific primitive or protocol commonly used in zero-knowledge systems.

### `fft/`
Implementations of the Fast Fourier Transform over suitable fields and domains.

Used for:
- Polynomial evaluation and interpolation
- Fast polynomial multiplication
- Foundations for quotient polynomials in zk systems

Includes:
- Recursive FFT
- Inverse FFT
- Polynomial multiplication via FFT

---

### `univariate_poly/`
Core univariate polynomial operations over finite fields.

Includes:
- Polynomial arithmetic
- Evaluation
- Interpolation
- Utilities reused across multiple protocols

---

### `multivariate_poly/`
Multivariate polynomial representations and operations.

Relevant for:
- Multilinear extensions
- Sum-check protocol
- GKR-style protocols

---

### `lagrange interpolation over prime fields/`
Standalone implementations of Lagrange interpolation over finite fields.

Used for:
- Polynomial reconstruction from evaluations
- Foundations for commitments and proof systems

---

### `shamir_secret_sharing/`
Implementation of Shamir’s Secret Sharing scheme.

Covers:
- Polynomial-based secret sharing
- Threshold reconstruction
- Finite-field arithmetic

Serves as an early bridge between polynomial algebra and cryptographic protocols.

---

### `sum_check/`
Implementation of the Sum-Check protocol.

Used in:
- GKR
- Multilinear polynomial verification
- Interactive proof systems

Includes:
- Iterative sum-check rounds
- Polynomial reductions
- Verifier checks

---

### `gkr/`
Implementation of the Goldwasser–Kalai–Rothblum (GKR) protocol.

Focuses on:
- Verifying computations over layered arithmetic circuits
- Combining sum-check with polynomial techniques
- Scalable verification of computation

---

### `kzg/`
Implementations related to the Kate–Zaverucha–Goldberg (KZG) polynomial commitment scheme.

Includes:
- Univariate KZG commitments
- Multilinear KZG constructions
- Witness computation and verification logic

This module connects polynomial algebra directly to cryptographic commitments and pairings.

---

### `impl simple circuit/`
Early experiments with representing and evaluating arithmetic circuits.

Used to:
- Ground abstract protocols in concrete computation
- Prepare for circuit-based proof systems and arithmetization

---

## Goals of This Repository

- Build zero-knowledge primitives from scratch
- Understand how algebra, polynomials, and cryptography connect
- Explore how FFTs, commitments, and interactive proofs fit together
- Serve as a long-term reference while studying zk systems

This is **not** a production library.  
Correctness, clarity, and understanding take priority over optimization.

---

## Planned / Future Additions

Possible future work includes:
- FFT-based polynomial division
- Multi-point KZG openings
- PLONK-style quotient polynomial construction
- Lookup arguments
- Constraint systems and arithmetization
- Comparisons with STARK-style constructions

---

## Notes

- Most modules assume familiarity with finite fields and basic cryptography
- Implementations may change as understanding improves
- Some code intentionally favors clarity over performance
