use std::vec;

use ark_ff::{BigInteger, PrimeField};
use multivariate_poly::MultilinearPolynomial;
use sha3::Keccak256;
use sum_check::transcript::Transcript;

use crate::{
    circuit::Circuit,
    sumcheck::{PartialProver, SumCheckProof},
};

#[derive(Debug, Clone)]
pub struct Proof<F: PrimeField> {
    pub claimed_sum: F,
    pub sumcheck_proofs: Vec<SumCheckProof<F>>,
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
        let mut sumcheck_proof = PartialProver::new(f_bc_poly, claimed_sum).prove();
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
        claimed_sum,
        sumcheck_proofs,
        wb_evals,
        wc_evals,
    }
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
