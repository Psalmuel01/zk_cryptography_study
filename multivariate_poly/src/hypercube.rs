use ark_ff::PrimeField;

#[derive(Debug, Clone)]
struct HypercubePoint<F: PrimeField> {
    coordinates: Vec<F>,
}

fn boolean_hypercube<F: PrimeField>(points: Vec<F>) -> Vec<HypercubePoint<F>> {
    let size = points.len();
    let num_dimensions = (size as f64).log2().ceil() as usize;
    println!("num_dimensions: {}", num_dimensions);

    let mut hypercube = Vec::new();
    for i in 0..size {
        let mut coordinates = Vec::new();
        let mut temp = i;
        for _ in 0..num_dimensions {
            coordinates.push(F::from((temp & 1) as u32));
            temp >>= 1;
        }
        coordinates.reverse();
        hypercube.push(HypercubePoint { coordinates });
    }

    hypercube
}

// pair up points in twos in such a way that whichever index i pass representing the index on the hypercubes 
// ( for example a=1, b=2, c=3 depending on dimension 3), it pairs up points in which the index passed has constant values only 
// and others are different and return them in vector pairs

fn pair_points<F: PrimeField>(hypercube: &Vec<HypercubePoint<F>>, index: usize) -> Vec<(&HypercubePoint<F>, &HypercubePoint<F>)> {
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


// fn main() {
//     use ark_bn254::Fq;

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
//     let hypercube = boolean_hypercube(points);
//     for point in &hypercube {
//         println!("{:?}", point.coordinates);
//     }

//     // to evaluate a, pass index 0
//     let pairs = pair_points(&hypercube, 0);
//     for (i, (point1, point2)) in pairs.iter().enumerate() {
//         println!("P{i} {:?}, {:?}", point1.coordinates, point2.coordinates);
//     }
// }
