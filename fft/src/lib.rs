use std::marker::PhantomData;
use ark_ff::FftField;

pub struct Polynomial<T: FftField> {
    _marker: PhantomData<T>,
}

impl<T:FftField> Polynomial<T> {
    fn split_even_odd_sequences(sequence: &[T]) -> (Vec<T>, Vec<T>) {
        let  (mut even_sequence, mut odd_sequence) = (vec![], vec![]);

        sequence.iter().enumerate().for_each(|(i, x)| {
            if i % 2 == 0 {
                even_sequence.push(*x);
            } else {
                odd_sequence.push(*x);
            }
        });

        (even_sequence, odd_sequence)
    }

    fn fft(sequence: &[T], is_inverse: bool) -> Vec<T> {
        let n = sequence.len();

        // if just one length, return
        if n == 1 {
            return sequence.to_vec();
        }

        let (even_sequence, odd_sequence) = Self::split_even_odd_sequences(sequence);

        let (ye, yo) = (
            Self::fft(&even_sequence, is_inverse),
            Self::fft(&odd_sequence, is_inverse)
        );

        let root_of_unity = T::get_root_of_unity(n as u64);

        let w = match root_of_unity {
            Some(root) => {
                if is_inverse {
                    root.inverse()
                } else {
                    Some(root)
                }
            },
            None => None
        };

        let mut y = vec![T::from(0); n];

        (0..n/2).into_iter().for_each(|j| {
            let wj = w.unwrap().pow(vec![j as u64]);

            y[j] = ye[j] + wj * yo[j];
            y[j + (n / 2)] = ye[j] - wj * yo[j];
        });
        dbg!(y.clone());

        y
    }

    // Perform Fast Fourier Transforms to convert Polynomial to Values (Samples) Representation

    pub fn convert_to_evaluations(coeffs: &[T]) -> Vec<T> {
        Self::fft(coeffs, false)
    }

    pub fn convert_to_coefficents(values: &[T]) -> Vec<T> {
        Self::fft(&values, true)
            .iter()
            .map(|x| *x / T::from(values.len() as u64))
            .collect()
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use ark_bls12_377::Fr;

    #[test]
    pub fn test_fft_and_ifft() {
        // let coefficients = vec![Fr::from(5), Fr::from(3), Fr::from(2), Fr::from(1)];
        let coefficients2 = vec![Fr::from(4), Fr::from(5), Fr::from(0), Fr::from(0)];

        let values = Polynomial::convert_to_evaluations(&coefficients2);
        // dbg!(&values);
        let result_coefficients = Polynomial::convert_to_coefficents(&values);

        assert_eq!(result_coefficients, coefficients2)
    }
}