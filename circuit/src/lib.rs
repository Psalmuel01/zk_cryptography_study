use ark_ff::PrimeField;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Gate<F: PrimeField> {
    op: char,
    left: usize,
    right: usize,
    output: F,
}

#[allow(dead_code)]
impl<F: PrimeField> Gate<F> {
    fn new(op: char, left: usize, right: usize) -> Self {
        Self {
            op,
            left,
            right,
            output: F::zero(),
        }
    }

    fn operate(&mut self, inputs: Vec<F>) -> F {
        let a = inputs[self.left];
        let b = inputs[self.right];

        self.output = match self.op {
            '+' => a + b,
            '*' => a * b,
            _ => panic!("Invalid operation"),
        };

        self.output
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Layer<F: PrimeField> {
    gates: Vec<Gate<F>>,
    outputs: Vec<F>,
}

#[allow(dead_code)]
impl<F: PrimeField> Layer<F> {
    fn init(gates: Vec<Gate<F>>) -> Self {
        Self {
            gates,
            outputs: vec![],
        }
    }

    fn compute(&mut self, inputs: Vec<F>) -> Vec<F> {
        for gate in &mut self.gates {
            gate.operate(inputs.clone());
            self.outputs.push(gate.output);
        }

        self.outputs.clone()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Circuit<F: PrimeField> {
    inputs: Vec<F>,
    layers: Vec<Layer<F>>,
    outputs: Vec<Vec<F>>,
}

#[allow(dead_code)]
impl<F: PrimeField> Circuit<F> {
    fn create(inputs: Vec<F>, layers: Vec<Layer<F>>) -> Self {
        Self {
            inputs,
            layers,
            outputs: Vec::new(),
        }
    }

    fn execute(&mut self) -> Vec<Vec<F>> {
        let mut inputs = self.inputs.clone();

        for layer in &mut self.layers {
            inputs = layer.compute(inputs);
            self.outputs.push(inputs.clone());
        }

        self.outputs.clone()
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use ark_bn254::Fq;

    fn to_field(input: Vec<u64>) -> Vec<Fq> {
        input.into_iter().map(Fq::from).collect()
    }

    #[test]
    fn test_gate_operate() {
        let inputs = to_field(vec![1, 2]);
        let mut add_gate = Gate::new('+', 0, 1);
        let mut mul_gate = Gate::new('*', 0, 1);

        let add_output = add_gate.operate(inputs.clone());
        let mul_output = mul_gate.operate(inputs);
        assert_eq!(add_output, Fq::from(3));
        assert_eq!(mul_output, Fq::from(2));
    }

    #[test]
    fn test_layer_compute() {
        let inputs = to_field(vec![1, 2, 3, 4]);
        let add_gate = Gate::new('+', 0, 1);
        let mul_gate = Gate::new('*', 2, 3);
        let mut layer = Layer::init(vec![add_gate, mul_gate]);
        let outputs = layer.compute(inputs);
        assert_eq!(outputs, to_field(vec![3, 12]));
    }

    #[test]
    fn test_circuit_execute() {
        let inputs = to_field(vec![1, 2, 3, 4]);
        let gate_1: Gate<Fq> = Gate::new('+', 0, 1);
        let gate_2: Gate<Fq> = Gate::new('*', 2, 3);

        let gate_3: Gate<Fq> = Gate::new('+', 0, 1);

        let layer_1 = Layer::init(vec![gate_1, gate_2]);
        let layer_2 = Layer::init(vec![gate_3]);

        let mut circuit = Circuit::create(inputs, vec![layer_1, layer_2]);
        let circuit_eval = circuit.execute();

        assert_eq!(
            circuit_eval,
            vec![to_field(vec![3, 12]), to_field(vec![15])]
        );
        dbg!(circuit);
    }
}
