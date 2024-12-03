use crate::polynomial::Polynomial;

/// A type that can be used as a secret.
pub trait CombineSecret {
    /// Returns the number of shares.
    fn len(&self) -> usize;

    /// Returns an iterator over the shares.
    fn iter(&self) -> std::slice::Iter<Vec<u8>>;

    /// Returns the share at the specified index.
    /// 
    /// The default implementation calls `self.iter().nth(index).unwrap()`.
    /// When possible, it is recommended to override this method
    /// with a more efficient implementation.
    fn get(&self, index: usize) -> &Vec<u8> {
        self.iter().nth(index).unwrap()
    }
}

impl<const N: usize> CombineSecret for [Vec<u8>; N] {
    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn iter(&self) -> std::slice::Iter<Vec<u8>> {
        self[..].iter()
    }

    #[inline]
    fn get(&self, index: usize) -> &Vec<u8> {
        self[..].get(index).unwrap()
    }
}

macro_rules! impl_array {
    ($($t:ty),+ $(,)?) => {$(
        impl CombineSecret for $t {
            #[inline]
            fn len(&self) -> usize {
                self[..].len()
            }
        
            #[inline]
            fn iter(&self) -> std::slice::Iter<Vec<u8>> {
                self[..].iter()
            }
        
            #[inline]
            fn get(&self, index: usize) -> &Vec<u8> {
                self[..].get(index).unwrap()
            }
        }
    )+};
}

impl_array!(&[Vec<u8>], [Vec<u8>]);

macro_rules! impl_vec {
    ($($t:ty),+ $(,)?) => {$(
        impl CombineSecret for $t {
            #[inline]
            fn len(&self) -> usize {
                self.as_slice().len()
            }

            #[inline]
            fn iter(&self) -> std::slice::Iter<Vec<u8>> {
                self.as_slice().iter()
            }

            #[inline]
            fn get(&self, index: usize) -> &Vec<u8> {
                self.as_slice().get(index).unwrap()
            }
        }
    )+};
}

impl_vec!(Vec<Vec<u8>>, &Vec<Vec<u8>>);

/// Combines shares to reconstruct the secret.
///
/// ## Arguments
/// * `parts` - Shares of the secret.
///
/// ## Returns
/// * The original secret if successful; otherwise, an error.
///
/// ## Errors
/// * Returns an error if shares are inconsistent or insufficient.
pub fn combine<T: CombineSecret>(shares: T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Validate the parts for consistency and sufficiency.
    if shares.len() < 2 || shares.get(0).len() < 2 {
        return Err("invalid parts".into());
    }

    // Ensure all parts are of the same length.
    let first_part_len = shares.get(0).len();
    for part in shares.iter().skip(1) {
        if part.len() != first_part_len {
            return Err("all parts must be the same length".into());
        }
    }

    // Initialize vectors to store the secret and the x and y samples.
    let mut secret = vec![0u8; first_part_len - 1];
    let mut x_samples = vec![0u8; shares.len()];
    let mut y_samples = vec![0u8; shares.len()];

    // Ensure that the x-coordinates are unique.
    let mut check_set = std::collections::HashSet::new();
    for (idx, part) in shares.iter().enumerate() {
        let sample = part[first_part_len - 1];
        if check_set.contains(&sample) {
            return Err("duplicate part detected".into());
        }
        check_set.insert(sample);
        x_samples[idx] = sample;
    }

    // Reconstruct each byte of the secret using polynomial interpolation.
    for idx in 0..(first_part_len - 1) {
        for (i, part) in shares.iter().enumerate() {
            y_samples[i] = part[idx];
        }
        let val = Polynomial::interpolate(&x_samples, &y_samples, 0);
        secret[idx] = val;
    }

    Ok(secret)
}