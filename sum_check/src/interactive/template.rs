use crate::multilinear::MultilinearPoly;
use crate::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};

#[derive(Debug)]
struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(poly: &MultilinearPoly<F>, claimed_sum: F) -> Proof<F> {
    let mut round_polys = vec![];

    let mut transcript = Transcript::new();
    // &[u8]
    // [&[u8], &[u8], ...]
    // [[1, 2], [3, 4]]
    // [1, 2, 3, 4]
    transcript.append(
        poly.evals
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut poly = poly.clone();

    for _ in 0..poly.n_vars {
        let round_poly: [F; 2] = [
            poly.partial_evaluate(0, &F::zero()).evals.iter().sum(),
            poly.partial_evaluate(0, &F::one()).evals.iter().sum(),
        ];

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        round_polys.push(round_poly);

        let challenge = transcript.sample_field_element();

        poly = poly.partial_evaluate(0, &challenge);
    }

    Proof {
        claimed_sum,
        round_polys,
    }
}

fn verify<F: PrimeField>(poly: &MultilinearPoly<F>, proof: &Proof<F>) -> bool {
    if proof.round_polys.len() != poly.n_vars {
        return false;
    }

    let mut challenges = vec![];

    let mut transcript = Transcript::new();
    transcript.append(
        poly.evals
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut claimed_sum = proof.claimed_sum;

    for round_poly in &proof.round_polys {
        if claimed_sum != round_poly.iter().sum() {
            return false;
        }

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let challenge = transcript.sample_field_element();
        claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
        challenges.push(challenge);
    }

    if claimed_sum != poly.evaluate(&challenges) {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    use crate::multilinear::tests::to_field;
    use crate::multilinear::MultilinearPoly;
    use crate::sumcheck::{prove, verify};
    use ark_bn254::Fr as ArkField;

    use field_tracker::{end_tscope, print_summary, start_tscope, Ft};

    type Fr = Ft!(ArkField);

    #[test]
    fn test_sumcheck() {
        start_tscope!("sumcheck");
        let poly = MultilinearPoly::new(3, to_field(vec![0, 0, 0, 3, 0, 0, 2, 5]));
        let proof = prove(&poly, Fr::from(20));
        end_tscope!();
        print_summary!();
        // dbg!(verify(&poly, &proof));
    }
}
