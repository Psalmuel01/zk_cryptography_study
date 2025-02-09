pub mod interactive;
pub mod prover;
pub mod transcript;
pub mod verifier;

#[cfg(test)]
mod test {
    use crate::{prover::Prover, verifier::Verify};
    use ark_bn254::Fq;

    pub(crate) fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(|v| Fq::from(v)).collect()
    }

    #[test]
    fn test_sumcheck() {
        let eval_points = to_field(vec![0, 0, 0, 3, 0, 0, 2, 5]);
        let mut proof = Prover::new(&eval_points, Fq::from(10));
        let check_proof = proof.prove();

        let mut verify = Verify::new(&eval_points);
        dbg!(verify.verify(check_proof));
    }
}
