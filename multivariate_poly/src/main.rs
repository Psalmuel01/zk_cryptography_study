use ark_ff::{AdditiveGroup, FftField, Field, PrimeField};

#[derive(Debug, Clone)]
struct HypercubePoint<F: PrimeField> {
    coordinates: Vec<F>,
    result: F,
}

fn boolean_hypercube<F: PrimeField>(points: Vec<F>) -> Vec<HypercubePoint<F>> {
    let size = points.len();
    let dimension = (size as f64).log2().ceil() as usize;
    println!("dimension: {}", dimension);

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
                // Check if all other coordinates are different
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
// fn evaluate_pairs()
fn evaluate_point<F: PrimeField>(pair: (&HypercubePoint<F>, &HypercubePoint<F>), r: F) -> F {
    // assert!(r < pair.0.dimension);
    print!("{} ", pair.0.result);
    print!("{} ", pair.1.result);
    let y1 = pair.0.result;
    let y2 = pair.1.result;
    let eval = y1 + (r * (y2-y1));
    return eval;
}

fn evaluate_at_index<F: PrimeField>(hypercube: &Vec<HypercubePoint<F>>, index: usize, eval_point: F) {
    let pairs = pair_points(hypercube, index);
    for (i, (point1, point2)) in pairs.iter().enumerate() {
        println!("P{i} {:?}, {:?}", point1.coordinates, point2.coordinates);
    }
    for (i, pair) in pairs.iter().enumerate() {
        let pair_eval = evaluate_point(*pair, eval_point);
        println!("Evaluation for pair {}: {:?}", i, pair_eval);
    }
}

fn main() {
    use ark_bn254::Fq;

    // [0, 2, 0, 5] at a=5, b=2 => 34
    // [0, 0, 0, 3, 0, 0, 2, 5] at c = 3, b = 2, c = 1 => 22
    let points = vec![
        // Fq::from(0),
        // Fq::from(0),
        // Fq::from(0),
        // Fq::from(3),
        // Fq::from(0),
        // Fq::from(9),
        Fq::from(18),
        Fq::from(22),
    ];

    let hypercube = boolean_hypercube(points);
    for point in &hypercube {
        println!("{:?}, {:?}", point.coordinates, point.result);
    }

    // define index and eval_point
    let index = 0;
    let eval_point = Fq::from(1);
    evaluate_at_index(&hypercube, index, eval_point);

}
