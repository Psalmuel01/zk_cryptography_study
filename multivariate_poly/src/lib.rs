pub mod multilinear;
use ark_ff::{BigInteger, PrimeField};

#[derive(Debug, Clone, PartialEq)]
pub struct HypercubePoint<F: PrimeField> {
    coordinates: Vec<F>,
    result: F,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultilinearPolynomial<F: PrimeField> {
    pub coefficients: Vec<F>,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    pub fn new(coefficients: Vec<F>) -> Self {
        Self { coefficients }
    }

    pub fn dimension(&self) -> usize {
        let size = self.coefficients.len();
        (size as f64).log2().ceil() as usize
    }

    pub fn evaluate(&self, evaluations: &Vec<F>) -> F {
        if evaluations.len() != Self::dimension(&self) {
            panic!("Invalid number of evaluations");
        } else {
            let mut poly = self.clone();
            for (i, eval_point) in evaluations.iter().enumerate() {
                poly.coefficients = partial_evaluate(poly.coefficients, 0, *eval_point);
            }
            poly.coefficients[0]
        }
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        self.coefficients
            .iter()
            .flat_map(|coeff| coeff.into_bigint().to_bytes_be())
            .collect()
    }
}

fn boolean_hypercube<F: PrimeField>(points: Vec<F>) -> Vec<HypercubePoint<F>> {
    let size = points.len(); //8
    let dimension = (size as f64).log2().ceil() as usize;
    // println!("dimension: {}", dimension);

    let mut hypercube = Vec::new();

    for i in 0..size {
        let mut coordinates = Vec::new();
        let mut temp = i;
        for _ in 0..dimension {
            coordinates.push(F::from((temp & 1) as u32));
            temp >>= 1;
        }
        coordinates.reverse();
        hypercube.push(HypercubePoint {
            coordinates,
            result: points[i],
        });
    }

    hypercube
}

// pair up points in twos in such a way that whichever index i pass representing the index on the hypercubes
// ( for example a=1, b=2, c=3 depending on dimension 3), it pairs up points in which the index passed has constant values only
// and others are different and return them in vector pairs

fn pair_points<F: PrimeField>(
    hypercube: &Vec<HypercubePoint<F>>,
    index: usize,
) -> Vec<(&HypercubePoint<F>, &HypercubePoint<F>)> {
    let mut pairs = Vec::new();
    for i in 0..hypercube.len() {
        for j in i + 1..hypercube.len() {
            let point1 = &hypercube[i];
            let point2 = &hypercube[j];
            if point1.coordinates[index] != point2.coordinates[index] {
                // Check if all other coordinates are same
                let mut all_same = true;
                for coord in 0..hypercube[0].coordinates.len() {
                    if coord != index && point1.coordinates[coord] != point2.coordinates[coord] {
                        all_same = false;
                        break;
                    }
                }
                if all_same {
                    pairs.push((point1, point2));
                }
            }
        }
    }
    pairs
}

// evaluation at y1 + r(y2-y1) where r=3 and y1 and y2 are the result of the pairs
fn evaluate_point<F: PrimeField>(pair: (&HypercubePoint<F>, &HypercubePoint<F>), r: F) -> F {
    // assert!(r < pair.0.dimension);
    // print!("{} ", pair.0.result);
    // print!("{} ", pair.1.result);
    let y1 = pair.0.result;
    let y2 = pair.1.result;
    let eval = y1 + (r * (y2 - y1));
    eval
}

pub fn partial_evaluate<F: PrimeField>(points: Vec<F>, index: usize, eval_point: F) -> Vec<F> {
    let hypercube = boolean_hypercube(points);
    let pairs = pair_points(&hypercube, index);
    // for (i, (point1, point2)) in pairs.iter().enumerate() {
    //     println!("P{i} {:?}, {:?}", point1.coordinates, point2.coordinates);
    // }
    let mut eval_points = Vec::new();

    for (i, pair) in pairs.iter().enumerate() {
        let pair_eval = evaluate_point(*pair, eval_point);
        // println!("Evaluation for pair {}: {:?}", i, pair_eval);
        eval_points.push(pair_eval);
    }
    // println!("eval points: {:?}", eval_points[0]);
    eval_points
}

pub fn total_evaluate<F: PrimeField>(mut points: Vec<F>, evaluations: Vec<F>) -> Vec<F> {
    let dim = (points.len() as f64).log2().ceil() as usize;
    let mut hypercube = boolean_hypercube(points.clone());

    for i in 0..dim {
        points = partial_evaluate(points, dim - 1 - i, evaluations[dim - 1 - i]);
        hypercube = boolean_hypercube(points.clone());
    }

    hypercube.iter().map(|point| point.result).collect() // Return final results
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_boolean_hypercube() {
        let points = vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)];

        let hypercube = boolean_hypercube(points);

        let expected = vec![
            HypercubePoint {
                coordinates: vec![Fq::from(0), Fq::from(0)],
                result: Fq::from(0),
            },
            HypercubePoint {
                coordinates: vec![Fq::from(0), Fq::from(1)],
                result: Fq::from(2),
            },
            HypercubePoint {
                coordinates: vec![Fq::from(1), Fq::from(0)],
                result: Fq::from(0),
            },
            HypercubePoint {
                coordinates: vec![Fq::from(1), Fq::from(1)],
                result: Fq::from(5),
            },
        ];

        assert_eq!(hypercube, expected);
    }

    #[test]
    fn test_pair_points() {
        let points = vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)];
        let hypercube = boolean_hypercube(points);

        // Test pair_points for index 0 (expected pairs: points differ only at index 0)
        let pairs_index_0 = pair_points(&hypercube, 0);
        let expected_pairs_index_0 = vec![
            (&hypercube[0], &hypercube[2]), // 00 <-> 10
            (&hypercube[1], &hypercube[3]), // 01 <-> 11
        ];
        assert_eq!(pairs_index_0, expected_pairs_index_0);

        // Test pair_points for index 1 (expected pairs: points differ only at index 1)
        let pairs_index_1 = pair_points(&hypercube, 1);
        let expected_pairs_index_1 = vec![
            (&hypercube[0], &hypercube[1]), // 00 <-> 01
            (&hypercube[2], &hypercube[3]), // 10 <-> 11
        ];
        assert_eq!(pairs_index_1, expected_pairs_index_1);
    }

    #[test]
    fn test_evaluate_point() {
        let pair_1 = HypercubePoint {
            coordinates: vec![Fq::from(0), Fq::from(0)],
            result: Fq::from(0),
        };
        let pair_2 = HypercubePoint {
            coordinates: vec![Fq::from(0), Fq::from(1)],
            result: Fq::from(2),
        };
        let pairs = (&pair_1, &pair_2);
        let evaluation = evaluate_point(pairs, Fq::from(2));
        assert_eq!(evaluation, Fq::from(4));
    }

    #[test]
    fn test_partial_evaluate() {
        let points = vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)];
        // let hypercube = boolean_hypercube(points);
        let index = 0;
        let eval_point = Fq::from(5);
        let partial_eval = partial_evaluate(points, index, eval_point);
        let expected_partial_eval = vec![Fq::from(0), Fq::from(17)];
        assert_eq!(partial_eval, expected_partial_eval);
    }

    #[test]
    fn test_total_evaluate() {
        let points = vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)];
        let evaluations = vec![Fq::from(5), Fq::from(2)];
        let total_eval = total_evaluate(points, evaluations);
        let expected_total_eval = vec![Fq::from(34)];
        assert_eq!(total_eval, expected_total_eval);
    }

    #[test]
    fn test_evaluate() {
        let points = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5)];
        let polynomial = MultilinearPolynomial::new(points);
        println!("{:?}", polynomial);
        let evaluations = vec![Fq::from(1), Fq::from(2), Fq::from(3)];
        let eval = polynomial.evaluate(&evaluations);
        let expected_eval = Fq::from(22);
        assert_eq!(eval, expected_eval);
    }

    #[test]
    fn test_convert_to_bytes() {
        let points = vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)];
        let polynomial = MultilinearPolynomial::new(points);
        let poly_bytes = polynomial.convert_to_bytes();
        println!("{:?}", poly_bytes);
    }
}

// fn main() {
//     use ark_bn254::Fq;

//     // [0, 2, 0, 5] at a=5, b=2 => 34
//     // [0, 0, 0, 3, 0, 0, 2, 5] at a = 1, b = 2, c = 3 => 22

//     let points = vec![
//         Fq::from(0),
//         Fq::from(0),
//         Fq::from(0),
//         Fq::from(3),
//         Fq::from(0),
//         Fq::from(0),
//         Fq::from(2),
//         Fq::from(5),
//     ];

//     let hypercube = boolean_hypercube(points.clone());

//     let index = 0;
//     let eval_point = Fq::from(1);

//     // for point in &hypercube {
//     //     println!("{:?}, {:?}", point.coordinates, point.result);
//     // }

//     partial_evaluate(&hypercube, index, eval_point);

//     // insert evaluations in ascending order
//     let evaluations = vec![Fq::from(1), Fq::from(2), Fq::from(3)];
//     total_evaluate(points, evaluations);
// }

// fn evaluate(&self, point: &[u32]) -> u32 {
//     let mut result = 0;
//     for (i, &coeff) in self.coefficients.iter().enumerate() {3
//         let mut term = coeff;
//         for (j, &coord) in point.iter().enumerate() {2
//             if (i >> j) & 1 == 1 {
//                 term *= coord;
//             }
//         }
//         result += term;
//     }

//     let coefficients = [3, 2, 4, 1];
//     let point = [2, 3];
//     let result = evaluate(&coefficients, &point);
//     println!("Result: {}", result); // Output: 25

//     result
// }
