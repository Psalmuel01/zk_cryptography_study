pub mod interactive;
pub mod prover;
pub mod transcript;
pub mod verifier;

#[cfg(test)]
mod test {
    use crate::{prover::Prover, verifier::Verify};
    use field_tracker::{end_tscope, print_summary, start_tscope, Ft};
    type Fq = Ft!(ark_bn254::Fq);
    // use ark_bn254::Fq;

    // pub(crate) fn to_field(input: Vec<u64>) -> Vec<Fq> {
    //     input.into_iter().map(|v| Fq::from(v)).collect()
    // }

    #[test]
    fn test_sumcheck() {
        // let eval_points = to_field(vec![0, 0, 0, 3, 0, 0, 2, 5]);
        let eval_points = vec![Fq::from(1); 1 << 10];

        start_tscope!("Prove");
        let mut proof = Prover::new(&eval_points, Fq::from(1 << 10));
        let check_proof = proof.prove();
        end_tscope!();

        start_tscope!("Verify");
        let mut verify = Verify::new(&eval_points);
        assert_eq!(verify.verify(check_proof), true);
        end_tscope!();

        print_summary!();
    }
}
