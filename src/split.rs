// #![forbid(unsafe_code)]
// #![warn(clippy::all)]

use std::marker::{PhantomData, PhantomPinned};

use crate::polynomial::Polynomial;

/// A type that can be used as a secret.
pub trait Secret {
    /// Returns `true` if the secret is empty.
    fn is_empty(&self) -> bool;

    /// Returns the length of the secret.
    /// 
    /// The default implementation calls `self.iter().count()`.
    /// When possible, it is recommended to override this method
    /// with a more efficient implementation.
    #[inline]
    fn len(&self) -> usize {
        self.iter().count()
    }

    /// Returns an iterator over the bytes of the secret.
    fn iter(&self) -> std::slice::Iter<u8>;
}

// The existence of this function makes the compiler catch if the Secret
// trait is "object-safe" or not.
fn _assert_trait_object(_b: &dyn Secret) {}

impl<const N: usize> Secret for &[u8; N] {
    #[inline]
    fn is_empty(&self) -> bool {
        self[..].is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self[..].len()
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<u8> {
        self[..].iter()
    }
}

macro_rules! impl_usize {
    ($($t:ty),+ $(,)?) => {$(
        impl Secret for $t {
            #[inline]
            fn is_empty(&self) -> bool {
                self[..].is_empty()
            }
        
            #[inline]
            fn len(&self) -> usize {
                self[..].len()
            }
        
            #[inline]
            fn iter(&self) -> std::slice::Iter<u8> {
                self[..].iter()
            }
        }
    )+};
}

impl_usize!([u8], &[u8]);

macro_rules! impl_vec {
    ($($t:ty),+ $(,)?) => {$(
        impl Secret for $t {
            #[inline]
            fn is_empty(&self) -> bool {
                // Check if self is a Vec<u8> or [u8]
                self.as_slice().is_empty()
            }
        
            #[inline]
            fn len(&self) -> usize {
                self.as_slice().len()
            }
        
            #[inline]
            fn iter(&self) -> std::slice::Iter<u8> {
                self.as_slice().iter()
            }
        }
    )+};
}

impl_vec!(Vec<u8>, &Vec<u8>);

macro_rules! string_impl {
    ($($t:ty),+ $(,)?) => {$(
        impl Secret for $t {
            #[inline]
            fn is_empty(&self) -> bool {
                self.as_bytes().is_empty()
            }

            #[inline]
            fn len(&self) -> usize {
                self.as_bytes().len()
            }

            #[inline]
            fn iter(&self) -> std::slice::Iter<u8> {
                self.as_bytes().iter()
            }
        }
    )+};
}

string_impl!(&str, str, &String, String);

impl Secret for PhantomPinned {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<u8> {
        [].iter()
    }
}

impl<T: ?Sized> Secret for PhantomData<T> {
    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<u8> {
        [].iter()
    }
}

/// Splits a secret into multiple shares.
///
/// ## Arguments
/// * `secret` - The secret to be split.
/// * `threshold` - Minimum number of shares required to reconstruct the secret.
/// * `parts` - Total number of shares to create.
///
/// ## Returns
/// * A vector of shares if successful; otherwise, an error.
///
/// ## Errors
/// * Returns an error if parameters are invalid (e.g., `parts` < `threshold`).
pub fn split<T: Secret>(secret: T, parts: usize, threshold: usize) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    // Validate the input parameters.
    if parts < threshold || parts > 255 || !(2..=255).contains(&threshold) || secret.is_empty() {
        return Err("invalid input parameters".into());
    }

    // Generate a sequence of non-zero values in GF(2^8)
    let mut x_coordinates: Vec<_> = (1..=255).collect();

    // Shuffle to create a random permutation of the x-coordinates.
    let mut rng = rand::thread_rng();
    rand::seq::SliceRandom::shuffle(x_coordinates.as_mut_slice(), &mut rng);

    // Set `share_size` to be equal to the length of the secret.
    let share_size = secret.len();
    // Initialize the output vector to store shares where each share
    // will consist of the y-coordinates plus one additional byte
    // for the x-coordinate.
    let mut shares = vec![vec![0u8; share_size + 1]; parts];

    // Assign the x-coordinates to the last position of each share.
    for idx in 0..parts {
        shares[idx][share_size] = x_coordinates[idx];
    }

    // For a polynomial of degree `k−1`, you need `k` distinct points to uniquely determine it,
    // therefor we generate a polynomial of degree `threshold - 1`.
    let degree = (threshold - 1) as u8;

    // For each byte in the secret, create a polynomial and evaluate it at each x-coordinate.
    for (s_idx, &secret_byte) in secret.iter().enumerate() {
        // Generate a polynomial for the current byte of the secret.
        let polynomial = Polynomial::generate(secret_byte, degree);

        for p_idx in 0..parts {
            // Access the x-coordinate for the current share.
            let x = x_coordinates[p_idx];
            // Evaluate the polynomial at the x-coordinate. This calculates
            // the y-value of the polynomial, effectively generating a part
            // of the share.
            let y = polynomial.evaluate(x);
            // Assign the evaluated y-value to the current share.
            shares[p_idx][s_idx] = y;
        }
    }

    Ok(shares)
}