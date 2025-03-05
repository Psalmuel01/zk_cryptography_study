use std::vec;

use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::MultilinearPolynomial;
use sha3::Keccak256;
use sum_check::transcript::Transcript;

use crate::{
    circuit::Circuit,
    sumcheck::{partial_prove, partial_verify, PartialProof},
};

#[derive(Debug, Clone)]
pub struct Proof<F: PrimeField> {
    pub last_claimed_sum: F,
    pub sumcheck_proofs: Vec<PartialProof<F>>,
    pub wb_evals: Vec<F>,
    pub wc_evals: Vec<F>,
}

pub fn prove<F: PrimeField>(circuit: &mut Circuit<F>) -> Proof<F> {
    circuit.execute();

    let mut transcript: Transcript<Keccak256, F> = Transcript::init(Keccak256::default());
    let mut w_0_polynomial = circuit.w_i_polynomial(0);

    if w_0_polynomial.coefficients.len() == 1 {
        let mut padded_w_0 = w_0_polynomial.coefficients;
        padded_w_0.push(F::zero());
        w_0_polynomial = MultilinearPolynomial::new(padded_w_0);
    }

    transcript.absorb(&w_0_polynomial.convert_to_bytes());
    let challenge_a = transcript.squeeze();
    let mut claimed_sum = w_0_polynomial.evaluate(&vec![challenge_a]);

    let mut f_bc_poly;
    let mut sumcheck_proofs = Vec::new();
    let mut wb_evals = Vec::new();
    let mut wc_evals = Vec::new();
    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut rb_values = Vec::new();
    let mut rc_values = Vec::new();

    for layer_index in 0..circuit.outputs.len() {
        if layer_index == 0 {
            f_bc_poly = circuit.f_b_c(layer_index, vec![challenge_a], None, None, None, None);
        } else {
            f_bc_poly = circuit.f_b_c(
                layer_index,
                vec![challenge_a],
                Some(alpha),
                Some(beta),
                Some(&rb_values),
                Some(&rc_values),
            );
        }

        // Evaluate wb and wc to be used by verifier
        let sumcheck_proof = partial_prove(f_bc_poly, claimed_sum, &mut transcript);
        sumcheck_proofs.push(sumcheck_proof.clone());

        if layer_index < circuit.outputs.len() - 1 {
            let challenges = sumcheck_proof.random_challenges;
            let w_b = circuit.w_i_polynomial(layer_index + 1);
            let w_c = w_b.clone();

            let (wb_eval, wc_eval) = eval_wb_wc(&w_b, &w_c, &challenges);
            wb_evals.push(wb_eval);
            wc_evals.push(wc_eval);

            // use the randomness from the sumcheck proof, split into two vec! for rb and rc
            let middle = challenges.len() / 2;
            let (new_rb_values, new_rc_values) = challenges.split_at(middle);
            rb_values = new_rb_values.to_vec();
            rc_values = new_rc_values.to_vec();

            transcript.absorb(&wb_eval.into_bigint().to_bytes_be().as_slice());
            alpha = transcript.squeeze();
            transcript.absorb(&wc_eval.into_bigint().to_bytes_be().as_slice());
            beta = transcript.squeeze();

            // Compute claimed sum using linear combination form
            claimed_sum = (alpha * wb_eval) + (beta * wc_eval);
        }
    }

    Proof {
        last_claimed_sum: claimed_sum,
        sumcheck_proofs,
        wb_evals,
        wc_evals,
    }
}

pub fn verify<F: PrimeField>(proof: Proof<F>, circuit: &mut Circuit<F>) -> bool {
    let mut transcript: Transcript<Keccak256, F> = Transcript::init(Keccak256::default());
    let mut w_0_polynomial = circuit.w_i_polynomial(0);

    if w_0_polynomial.coefficients.len() == 1 {
        let mut padded_w_0 = w_0_polynomial.coefficients;
        padded_w_0.push(F::zero());
        w_0_polynomial = MultilinearPolynomial::new(padded_w_0);
    }

    transcript.absorb(&w_0_polynomial.convert_to_bytes());
    let challenge_a = transcript.squeeze();
    let mut claimed_sum = w_0_polynomial.evaluate(&vec![challenge_a]);

    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut prev_challenges = Vec::new();

    for layer_index in 0..circuit.outputs.len() {
        if claimed_sum != proof.sumcheck_proofs[layer_index].claimed_sum {
            return false;
        }

        let sumcheck_verif = partial_verify(&proof.sumcheck_proofs[layer_index], &mut transcript);
        if !sumcheck_verif.is_proof_valid {
            return false;
        }

        let wb_eval;
        let wc_eval;
        let challenges = sumcheck_verif.random_challenges;

        if layer_index < circuit.outputs.len() - 1 {
            (wb_eval, wc_eval) = (proof.wb_evals[layer_index], proof.wc_evals[layer_index]);
        } else {
            let w_b = MultilinearPolynomial::new(circuit.inputs.clone());
            let w_c = w_b.clone();
            (wb_eval, wc_eval) = eval_wb_wc(&w_b, &w_c, &challenges);
        }

        let expected_claim;

        if layer_index == 0 {
            expected_claim = compute_initial_claim(
                circuit,
                layer_index,
                challenge_a,
                &challenges,
                wb_eval,
                wc_eval,
            )
        } else {
            expected_claim = compute_folded_claim(
                circuit,
                layer_index,
                &prev_challenges,
                &challenges,
                wb_eval,
                wc_eval,
                alpha,
                beta,
            )
        };

        if expected_claim != sumcheck_verif.last_claimed_sum {
            return false;
        }

        prev_challenges = challenges;

        transcript.absorb(&wb_eval.into_bigint().to_bytes_be().as_slice());
        alpha = transcript.squeeze();

        transcript.absorb(wc_eval.into_bigint().to_bytes_be().as_slice());
        beta = transcript.squeeze();

        claimed_sum = (alpha * wb_eval) + (beta * wc_eval);
    }

    true
}

pub fn eval_wb_wc<F: PrimeField>(
    wb_poly: &MultilinearPolynomial<F>,
    wc_poly: &MultilinearPolynomial<F>,
    challenges: &Vec<F>,
) -> (F, F) {
    let middle = challenges.len() / 2;
    let (rb_values, rc_values) = challenges.split_at(middle);

    let wb_poly_evaluated = wb_poly.evaluate(&rb_values.to_vec());
    let wc_poly_evaluated = wc_poly.evaluate(&rc_values.to_vec());

    (wb_poly_evaluated, wc_poly_evaluated)
}

pub fn compute_initial_claim<F: PrimeField>(
    circuit: &mut Circuit<F>,
    layer_index: usize,
    challenge_ra: F,
    challenges: &Vec<F>,
    wb_eval: F,
    wc_eval: F,
) -> F {
    let (add_i_poly, mul_i_poly) = circuit.add_i_n_mul_i_arrays(layer_index);
    let (add_i_bc, mul_i_bc) = (
        add_i_poly.partial_evaluate(0, challenge_ra),
        mul_i_poly.partial_evaluate(0, challenge_ra),
    );

    let add_r = add_i_bc.evaluate(challenges);
    let mul_r = mul_i_bc.evaluate(challenges);

    (add_r * (wb_eval + wc_eval)) + (mul_r * (wb_eval * wc_eval))
}

pub fn compute_folded_claim<F: PrimeField>(
    circuit: &mut Circuit<F>,
    layer_index: usize,
    prev_challenges: &Vec<F>,
    challenges: &Vec<F>,
    wb_eval: F,
    wc_eval: F,
    alpha: F,
    beta: F,
) -> F {
    let mid = prev_challenges.len() / 2;
    let (challenge_rb, challenge_rc) = prev_challenges.split_at(mid);

    let (new_add_i, new_mul_i) = circuit.alpha_beta_add_n_mul_bc(
        alpha,
        beta,
        &challenge_rb.to_vec(),
        &challenge_rc.to_vec(),
        layer_index,
    );

    let add_r = new_add_i.evaluate(challenges);
    let mul_r = new_mul_i.evaluate(challenges);

    (add_r * (wb_eval + wc_eval)) + (mul_r * (wb_eval * wc_eval))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{Gate, Layer};
    use ark_bn254::Fq;

    fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(Fq::from).collect()
    }

    #[test]
    fn test_gkr() {
        let inputs = to_field(vec![1, 2, 3, 4, 5, 6, 7, 8]);

        let gate_1: Gate = Gate::new('+', 0, 1, 0);
        let gate_2: Gate = Gate::new('*', 2, 3, 1);
        let gate_3: Gate = Gate::new('*', 4, 5, 2);
        let gate_4: Gate = Gate::new('*', 6, 7, 3);

        let gate_5: Gate = Gate::new('+', 0, 1, 0);
        let gate_6: Gate = Gate::new('*', 2, 3, 1);

        let gate_7: Gate = Gate::new('+', 0, 1, 0);

        let layer_0 = Layer::init(vec![gate_7]);
        let layer_1 = Layer::init(vec![gate_5, gate_6]);
        let layer_2 = Layer::init(vec![gate_1, gate_2, gate_3, gate_4]);

        let mut circuit = Circuit::create(inputs, vec![layer_0, layer_1, layer_2]);
        let proof = prove(&mut circuit);
        dbg!(&proof);
        let verify = verify(proof, &mut circuit);
        assert_eq!(verify, true);
    }
}
