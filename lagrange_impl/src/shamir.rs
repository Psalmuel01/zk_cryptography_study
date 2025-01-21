use rand::Rng;

/// Generates `n` shares of a secret using Shamir's Secret Sharing scheme.
/// At least `threshold` shares are needed to reconstruct the secret.
fn generate_shares(secret: i32, n: usize, threshold: usize) -> Vec<(i32, i32)> {
    assert!(threshold <= n, "Threshold must be <= n");
    
    // Generate random coefficients for the polynomial
    let mut coefficients = vec![secret];
    let mut rng = rand::thread_rng();
    for _ in 1..threshold {
        coefficients.push(rng.gen_range(0..256)); // Random coefficients in the range [0, 255]
    }

    // Evaluate the polynomial at different x values
    let mut shares = Vec::new();
    for x in 1..=n as i32 {
        let mut y = 0;
        for (i, &coeff) in coefficients.iter().enumerate() {
            y += coeff * x.pow(i as u32);
        }
        shares.push((x, y));
    }

    shares
}

/// Reconstructs the secret from the given shares using Lagrange interpolation.
fn reconstruct_secret(shares: &[(i32, i32)]) -> i32 {
    let mut secret = 0;

    for (i, &(x_i, y_i)) in shares.iter().enumerate() {
        let mut numerator = 1;
        let mut denominator = 1;

        for (j, &(x_j, _)) in shares.iter().enumerate() {
            if i != j {
                numerator *= x_j;
                denominator *= x_j - x_i;
            }
        }

        // Add the contribution of this share to the secret
        secret += y_i * numerator / denominator;
    }

    secret
}

fn main() {
    let secret = 123; // The secret to be shared
    let n = 5; // Total number of shares
    let threshold = 3; // Minimum number of shares needed to reconstruct the secret

    // Generate shares
    let shares = generate_shares(secret, n, threshold);
    println!("Shares: {:?}", shares);

    // Select any `threshold` shares to reconstruct the secret
    let selected_shares = &shares[..threshold];
    let reconstructed_secret = reconstruct_secret(selected_shares);
    println!("Reconstructed Secret: {}", reconstructed_secret);
}
