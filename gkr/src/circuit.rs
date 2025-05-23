use ark_ff::PrimeField;
use multivariate_poly::{
    add_polynomials, product_poly::ProductPoly, sum_poly::SumPoly, tensor_add, tensor_mul,
    MultilinearPolynomial,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Gate {
    op: char,
    left: usize,
    right: usize,
    output: usize,
}

#[allow(dead_code)]
impl Gate {
    pub fn new(op: char, left: usize, right: usize, output: usize) -> Self {
        Self {
            op,
            left,
            right,
            output,
        }
    }

    fn operate<F: PrimeField>(&mut self, inputs: Vec<F>) -> F {
        let a = inputs[self.left];
        let b = inputs[self.right];
        // self.output = output_index;

        match self.op {
            '+' => a + b,
            '*' => a * b,
            _ => panic!("Invalid operation"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Layer<F: PrimeField> {
    gates: Vec<Gate>,
    outputs: Vec<F>,
}

#[allow(dead_code)]
impl<F: PrimeField> Layer<F> {
    pub fn init(gates: Vec<Gate>) -> Self {
        Self {
            gates,
            outputs: vec![],
        }
    }

    fn compute(&mut self, inputs: Vec<F>) -> Vec<F> {
        for gate in self.gates.iter_mut() {
            let output = gate.operate(inputs.clone());
            self.outputs.push(output);
        }

        self.outputs.clone()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Circuit<F: PrimeField> {
    pub inputs: Vec<F>,
    pub layers: Vec<Layer<F>>,
    pub outputs: Vec<Vec<F>>,
}

#[allow(dead_code)]
impl<F: PrimeField> Circuit<F> {
    pub fn create(inputs: Vec<F>, layers: Vec<Layer<F>>) -> Self {
        Self {
            inputs,
            layers,
            outputs: Vec::new(),
        }
    }

    pub fn execute(&mut self) -> Vec<Vec<F>> {
        let mut inputs = self.inputs.clone();

        for layer in self.layers.iter_mut().rev() {
            inputs = layer.compute(inputs);
            self.outputs.insert(0, inputs.clone());
        }

        self.outputs.clone()
    }

    pub fn w_i_polynomial(&self, layer_index: usize) -> MultilinearPolynomial<F> {
        assert!(layer_index <= self.outputs.len(), "layer doesn't exist");
        if layer_index == self.outputs.len() {
            let layer_coeffs = self.inputs.clone();
            return MultilinearPolynomial::new(layer_coeffs);
        } else {
            let layer_coeffs = self.outputs[layer_index].clone();
            MultilinearPolynomial::new(layer_coeffs)
        }
    }

    pub fn add_i_n_mul_i_arrays(
        &self,
        layer_index: usize,
    ) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>) {
        let index_combinations = gate_index_combinations(layer_index);
        let boolean_combination = 1 << index_combinations;

        let mut add_i_values = vec![F::zero(); boolean_combination];
        let mut mul_i_values = vec![F::zero(); boolean_combination];

        for gate in self.layers[layer_index].gates.iter() {
            match gate.op {
                '+' => {
                    let valid_index =
                        arrange_gate_index(layer_index, gate.output, gate.left, gate.right);
                    add_i_values[valid_index] = F::one();
                }
                '*' => {
                    let valid_index =
                        arrange_gate_index(layer_index, gate.output, gate.left, gate.right);
                    mul_i_values[valid_index] = F::one();
                }
                _ => panic!("Invalid operation"),
            }
        }

        let add_i_poly = MultilinearPolynomial::new(add_i_values.clone());
        let mul_i_poly = MultilinearPolynomial::new(mul_i_values.clone());

        (add_i_poly, mul_i_poly)
    }

    // New addi+1 = alpha * addi+1(rb, b, c) + beta * addi+1(rc, b, c)
    // where alpha & beta are squeezed from transcript, rb = first half of random chal sent from the sumcheck prover and rc = second half of random chal sent from the sumcheck prover
    pub fn alpha_beta_add_n_mul_bc(
        &self,
        alpha: F,
        beta: F,
        r_bs: &Vec<F>,
        r_cs: &Vec<F>,
        layer_index: usize,
    ) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>) {
        let (add_i_poly, mul_i_poly) = self.add_i_n_mul_i_arrays(layer_index);

        let mut add_r_b = add_i_poly.partial_evaluate(0, r_bs[0]);
        let mut add_r_c = add_i_poly.partial_evaluate(0, r_cs[0]);

        let mut mul_r_b = mul_i_poly.partial_evaluate(0, r_bs[0]);
        let mut mul_r_c = mul_i_poly.partial_evaluate(0, r_cs[0]);

        for rb in r_bs.iter().skip(1) {
            add_r_b = add_r_b.partial_evaluate(0, *rb);
            mul_r_b = mul_r_b.partial_evaluate(0, *rb);
        }

        for rc in r_cs.iter().skip(1) {
            add_r_c = add_r_c.partial_evaluate(0, *rc);
            mul_r_c = mul_r_c.partial_evaluate(0, *rc);
        }

        let new_add_i = add_polynomials(add_r_b.scalar_mul(alpha), add_r_c.scalar_mul(beta));
        let new_mul_i = add_polynomials(mul_r_b.scalar_mul(alpha), mul_r_c.scalar_mul(beta));

        (new_add_i, new_mul_i)
    }

    pub fn f_b_c(
        &self,
        layer_index: usize,
        a_s: Vec<F>,
        alpha: Option<F>,
        beta: Option<F>,
        r_bs: Option<&Vec<F>>,
        r_cs: Option<&Vec<F>>,
    ) -> SumPoly<F> {
        let (add_i_poly, mul_i_poly) = self.add_i_n_mul_i_arrays(layer_index);

        let mut add_bc = add_i_poly;
        let mut mul_bc = mul_i_poly;

        let (add_bc, mul_bc) = if layer_index == 0 {
            for i in 0..a_s.len() {
                add_bc = add_bc.partial_evaluate(0, a_s[i]);
                mul_bc = mul_bc.partial_evaluate(0, a_s[i]);
            }
            (add_bc, mul_bc)
        } else {
            self.alpha_beta_add_n_mul_bc(
                alpha.unwrap(),
                beta.unwrap(),
                r_bs.unwrap(),
                r_cs.unwrap(),
                layer_index,
            )
        };

        let w_i = self.w_i_polynomial(layer_index + 1);
        let w_add_bc = tensor_add(w_i.clone(), w_i.clone());
        let w_mul_bc = tensor_mul(w_i.clone(), w_i.clone());
        println!("add_bc: {:?}", add_bc.clone());
        println!("w_add_bc: {:?}", w_add_bc.clone());
        SumPoly::new(vec![
            ProductPoly::new(vec![add_bc, w_add_bc]),
            ProductPoly::new(vec![mul_bc, w_mul_bc]),
        ])
    }
}

fn gate_index_combinations(layer_index: usize) -> usize {
    if layer_index == 0 {
        return 3;
    }

    let a = layer_index;
    let b = layer_index + 1;
    let c = layer_index + 1;

    a + b + c
}

fn arrange_gate_index(
    layer_index: usize,
    output: usize,
    left_index: usize,
    right_index: usize,
) -> usize {
    let output_binary = decimal_to_padded_binary(output, layer_index);
    let left_binary = decimal_to_padded_binary(left_index, layer_index + 1);
    let right_binary = decimal_to_padded_binary(right_index, layer_index + 1);
    let result = output_binary + &left_binary + &right_binary;

    usize::from_str_radix(&result, 2).unwrap_or(0)
}

fn decimal_to_padded_binary(n: usize, bit_length: usize) -> String {
    format!("{:0>width$b}", n, width = bit_length)
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    pub fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(Fq::from).collect()
    }

    #[test]
    fn test_f_b_c() {
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
        circuit.execute();
        let f_b_c = circuit.f_b_c(0, to_field(vec![5]), None, None, None, None);
        dbg!(f_b_c);
    }

    #[test]
    fn test_w_i_polynomial() {
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
        circuit.execute();
        let w_i = circuit.w_i_polynomial(1);
        assert_eq!(w_i.coefficients, to_field(vec![15, 1680]));
    }

    #[test]
    fn test_gate_index_combinations() {
        let index = gate_index_combinations(1);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_arrange_gate_index() {
        let index = arrange_gate_index(1, 1, 2, 3);
        let expected = usize::from_str_radix("11011", 2).unwrap_or(0);
        assert_eq!(index, expected);
    }

    #[test]
    fn test_decimal_to_padded_binary() {
        let binary = decimal_to_padded_binary(3, 4);
        assert_eq!(binary, "0011");
    }

    #[test]
    fn test_add_i_n_mul_i_arrays() {
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

        let circuit = Circuit::create(inputs, vec![layer_0, layer_1, layer_2]);
        let (add_i_poly, mul_i_poly) = circuit.add_i_n_mul_i_arrays(1);
        assert_eq!(
            add_i_poly.coefficients,
            to_field(vec![
                0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
        );
        assert_eq!(
            mul_i_poly.coefficients,
            to_field(vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 0, 0
            ])
        );
        dbg!(add_i_poly);
        dbg!(mul_i_poly);

        // assert_eq!(add_i_values, vec!["00000001"]);
        // assert_eq!(mul_i_values, vec!["01010011", "10100101", "11110111"]);
    }

    #[test]
    fn test_gate_operate() {
        let inputs = to_field(vec![1, 2]);
        let mut add_gate = Gate::new('+', 0, 1, 0);
        let mut mul_gate = Gate::new('*', 0, 1, 1);

        let add_output = add_gate.operate(inputs.clone());
        let mul_output = mul_gate.operate(inputs);
        assert_eq!(add_output, Fq::from(3));
        assert_eq!(mul_output, Fq::from(2));
    }

    #[test]
    fn test_layer_compute() {
        let inputs = to_field(vec![1, 2, 3, 4]);
        let add_gate = Gate::new('+', 0, 1, 0);
        let mul_gate = Gate::new('*', 2, 3, 0);
        let mut layer = Layer::init(vec![add_gate, mul_gate]);
        let outputs = layer.compute(inputs);
        assert_eq!(outputs, to_field(vec![3, 12]));
    }

    #[test]
    fn test_circuit_execute() {
        let inputs = to_field(vec![1, 2, 3, 4]);

        let gate_1: Gate = Gate::new('+', 0, 1, 0);
        let gate_2: Gate = Gate::new('*', 2, 3, 1);

        let gate_3: Gate = Gate::new('+', 0, 1, 0);

        let layer_0 = Layer::init(vec![gate_3]);
        let layer_1 = Layer::init(vec![gate_1, gate_2]);

        let mut circuit = Circuit::create(inputs, vec![layer_0, layer_1]);
        let circuit_eval = circuit.execute();

        assert_eq!(
            circuit_eval,
            vec![to_field(vec![15]), to_field(vec![3, 12])]
        );
        dbg!(circuit);
    }

    #[test]
    fn test_circuit_execute_2() {
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
        let circuit_eval = circuit.execute();

        assert_eq!(
            circuit_eval,
            vec![
                to_field(vec![1695]),
                to_field(vec![15, 1680]),
                to_field(vec![3, 12, 30, 56])
            ]
        );
        // dbg!(circuit);
    }
}

// let mut add_indices = vec![];
// let mut mul_indices = vec![];
// let indices: Vec<usize> = add_i.iter().enumerate().filter(|(_, &x)| x == 1).map(|(i, _)| i).collect();
// for (i, x) in add_i_values.iter().enumerate() {
//     if *x == F::one() {
//         let i_binary = decimal_to_padded_binary(i, index_combinations);
//         add_indices.push(i_binary);
//     }
// }
// for (i, x) in mul_i_values.iter().enumerate() {
//     if *x == F::one() {
//         let i_binary = decimal_to_padded_binary(i, index_combinations);
//         mul_indices.push(i_binary);
//     }
// }
// dbg!(add_indices, mul_indices);
