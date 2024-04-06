// #![forbid(unsafe_code)]
// #![warn(clippy::all)]

mod ops;
mod polynomial;
use std::ffi::CStr;
use std::os::raw::c_char;
use polynomial::Polynomial;
use hex;

/// A secret to be split into shares.
/// The secret is a byte slice that represents the original secret.
#[repr(C)]
#[derive(Clone)]
pub struct Secret {
    pub value: *mut String,
}

/// A share data that consists of multiple shares.
/// The share data is a vector of shares and the length of the vector.
#[repr(C)] // Explicit C-compatible struct layout
pub struct ShareData {
    pub secrets: *mut Secret, // Raw pointer to the share data
    pub len: usize,    // Length of the share data
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
pub fn split(secret: &[u8], parts: usize, threshold: usize) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
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

/// Splits a secret into multiple shares.
/// 
/// ## Arguments
/// * `secret` - The secret to be split in string.
/// * `parts` - Total number of shares to create.
/// * `threshold` - Minimum number of shares required to reconstruct the secret.
/// 
/// ## Returns
/// * A string of hex shares if successful; otherwise, an error.
/// 
/// ## Errors
/// * Returns an error if parameters are invalid (e.g., `parts` < `threshold`).
pub fn split_string(secret: &str, parts: usize, threshold: usize) -> (Vec<String>, Box<dyn std::error::Error>) {
    // Split the secret into shares.¯
    let shamir = split(secret.as_bytes(), parts, threshold);
    
    match shamir {
        Ok(data) => {
            let mut encoded_shamir: Vec<String> = Vec::new();

            // Convert the shares to hex strings.
            // This is done to make the shares human-readable.
            for share in data.iter() {
                encoded_shamir.push(hex::encode(share));
            }

            (encoded_shamir, "Success".into())
        },
        Err(e) => (vec![], e)
    }
}

/// A C-compatible function to split a secret into multiple shares.
/// 
/// ## Arguments
/// * `secret` - The secret to be split in C-compatible struct.
/// * `secret_len` - Length of the secret.
/// * `parts` - Total number of shares to create.
/// * `threshold` - Minimum number of shares required to reconstruct the secret.
/// 
/// ## Returns
/// * `true` if successful; otherwise, `false`.
#[no_mangle]
pub extern "C" fn split_string_c(secret: *const c_char, parts: usize, threshold: usize) -> *mut ShareData {

    // Convert the secret to a byte slice.
    let secret_slice = unsafe { CStr::from_ptr(secret) };

    // Split the secret into shares.
    let shamir = split(secret_slice.to_bytes(), parts, threshold);

    match shamir {
        Ok(data) => {
            let mut encoded_shamir: Vec<Secret> = vec![];

            // Convert the shares to hex strings.
            // This is done to make the shares human-readable.
            for share in data.iter() {
                let encoded_share = hex::encode(share);
                // encoded_shamir.push(Box::into_raw(Box::new(encoded_share)));
                encoded_shamir.push(Secret {
                    value: Box::into_raw(Box::new(encoded_share)),
                });
            }

            // Return the shares as a C-compatible struct.
            Box::into_raw(Box::new(ShareData {
                secrets: Box::into_raw(encoded_shamir.into_boxed_slice()) as *mut Secret,
                len: parts
            }))
        },
        Err(_) => {

            Box::into_raw(Box::new(ShareData {
                secrets: Box::into_raw(Box::new(Secret {
                    value: Box::into_raw(Box::new("".to_string())),
                })),
                len: 0
            }))
        }
        
    }
}

/// A C-compatible function to clear the memory allocated for the shares.
/// 
/// ## Arguments
/// * `data` - The shares to be cleared.
/// 
/// ## Returns
/// * `true` if successful; otherwise, `false`.
#[no_mangle]
pub extern "C" fn create_share_data(value: *const c_char) -> *mut ShareData {
    // Convert the secret to a byte slice.
    let secret_slice = unsafe { CStr::from_ptr(value) };

    // Create a vector of shares.
    let mut secret_vec: Vec<Secret> = Vec::new();
    secret_vec.push(Secret {
        value: Box::into_raw(Box::new(secret_slice.to_str().unwrap().to_string())),
    });

    // Convert the shares to a C-compatible struct.
    let secret = ShareData {
        secrets: Box::into_raw(secret_vec.into_boxed_slice()) as *mut Secret,
        len: 1,
    };
    Box::into_raw(Box::new(secret))
}

/// A C-compatible function to add shares to the existing shares.
/// 
/// ## Arguments
/// * `data` - The existing shares.
/// * `value` - The share to be added.
/// 
/// ## Returns
/// * The updated shares if successful; otherwise, an error.
#[no_mangle]
pub extern "C" fn add_share_data(data: *mut ShareData, value: *const c_char) -> *mut ShareData {
    let secret_slice = unsafe { CStr::from_ptr(value) };
    // convert to a mutable reference to the Vec share data.
    let secret_vec = unsafe {
        let data = data.as_mut().unwrap();
        std::slice::from_raw_parts_mut(data.secrets, data.len)
    };

    let mut temp = secret_vec.to_vec();
    temp.push(Secret {
        value: Box::into_raw(Box::new(secret_slice.to_str().unwrap().to_string())),
    });

    let secret = ShareData {
        secrets: Box::into_raw(temp.into_boxed_slice()) as *mut Secret,
        len: unsafe {
            data.as_mut().unwrap().len + 1
        }
    };
    Box::into_raw(Box::new(secret))
}

/// A C-compatible function to clear the memory allocated for the shares.
/// 
/// ## Arguments
/// * `data` - The shares to be cleared.
/// 
/// ## Returns
/// * `true` if successful; otherwise, `false`.
#[no_mangle]
pub extern "C" fn clear_share_data(data: *mut ShareData) {
    if !data.is_null() {
        unsafe {
            let _ = Box::from_raw(data);
        }
    }
}

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
pub fn combine(shares: &[Vec<u8>]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Validate the parts for consistency and sufficiency.
    if shares.len() < 2 || shares[0].len() < 2 {
        return Err("invalid parts".into());
    }

    // Ensure all parts are of the same length.
    let first_part_len = shares[0].len();
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

/// Combines shares to reconstruct the secret.
/// 
/// ## Arguments
/// * `shares` - Shares of the secret in string.
/// 
/// ## Returns
/// * The original secret if successful; otherwise, an error.
/// 
/// ## Errors
/// * Returns an error if shares cannot be decoded
/// * Returns an error if shares are inconsistent or insufficient.
pub fn combile_string(shares: &[String]) -> (String, Box<dyn std::error::Error>) {
    let mut decoded_shares: Vec<Vec<u8>> = Vec::new();

    // Convert the shares from hex strings to bytes.
    for share in shares.iter() {
        match hex::decode(share) {
            Ok(data) => decoded_shares.push(data),
            Err(e) => return ("".into(), e.into())
        }
    }

    // Combine the shares to reconstruct the secret.
    let shamir = combine(&decoded_shares);

    match shamir {
        Ok(data) => (String::from_utf8(data).unwrap(), "Success".into()),
        Err(e) => ("".into(), e)
    }
}

/// A C-compatible function to combine shares to reconstruct the secret.
/// 
/// ## Arguments
/// * `shares` - Shares of the secret in C-compatible struct.
/// * `shares_len` - Number of shares.
/// * `secret` - The original secret.
/// * `error_message` - Error message if the operation fails.
/// 
/// ## Returns
/// * `true` if successful; otherwise, `false`.
#[no_mangle]
pub extern "C" fn combile_string_c(shares: *mut ShareData) -> *mut String {
    // Convert the shares to a slice of ShareData.
    let (configs, len) = unsafe {
        let shares = shares.as_ref().unwrap();
        (shares.secrets, shares.len)
    };

    // convert the shares to a vector of bytes.
    let mut decoded_shares: Vec<Vec<u8>> = Vec::new();
    for i in 0..len {
        let config = unsafe { configs.add(i) };
        let value = unsafe { &*config }.value;
        let value_str = unsafe { &*value };
        // decode the shares from hex strings to bytes.
        match hex::decode(value_str) {
            Ok(data) => decoded_shares.push(data),
            Err(_) => return Box::into_raw(Box::new(String::from(""))),
        }
    }

    // Combine the shares to reconstruct the secret.
    let shamir = combine(&decoded_shares);

    match shamir {
        Ok(data) => {
            let secret_string = String::from_utf8(data).unwrap();
            return Box::into_raw(Box::new(secret_string));
        },
        Err(_) => {
            return Box::into_raw(Box::new(String::from("")));
        }
    }
}

/// A C-compatible function to clear the memory allocated for the shares.
/// 
/// ## Arguments
/// * `data` - The shares to be cleared.
/// 
/// ## Returns
/// * `true` if successful; otherwise, `false`.
#[no_mangle]
pub extern "C" fn clear_string_c(data: *mut String) {
    if !data.is_null() {
        unsafe {
            let _ = Box::from_raw(data);
        }
    }
}

// Test cases for the `lib` module.
#[cfg(test)]
mod tests {
    use super::*;

    // The 'split' function with valid inputs.
    #[test]
    fn it_splits_secret() {
        let secret = b"test_secret";
        let threshold = 3;
        let parts = 5;

        // Split the secret into shares.
        let shares = split(secret, parts, threshold).expect("split failed");
        // The number of shares should match the specified number of parts.
        assert_eq!(shares.len(), parts);

        // Each share should be one byte longer than the secret to store the x-coordinate.
        for share in shares.iter() {
            assert_eq!(share.len(), secret.len() + 1);
        }
    }

    // The 'split' function with invalid inputs.
    #[test]
    fn it_fails_when_split_parts_less_than_thresshold() {
        let secret = b"test_secret";
        let threshold = 3;
        let parts = 2; // Less than the threshold

        assert!(split(secret, parts, threshold,).is_err());
    }

    // The 'combine' function with shares randomly generated from the split function.
    #[test]
    fn it_combines_from_random_shares() {
        let secret = [1, 2, 3]; // Original secret
        let threshold = 3;
        let parts = 5;

        let shares = split(&secret, parts, threshold).expect("split failed");
        // Choose a subset of shares that meet the threshold
        let selected_shares = &shares[..threshold];

        let reconstructed = combine(selected_shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // The `combine` function with known shares.
    #[test]
    fn it_combines_from_known_shares() {
        let secret = b"test_secret";
        // Valid known shares
        let shares = vec![
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
            vec![162, 176, 148, 45, 83, 38, 153, 204, 80, 141, 4, 1],
            vec![35, 165, 19, 114, 53, 31, 70, 25, 74, 248, 145, 132],
        ];

        // Combine the shares to reconstruct the secret.
        let reconstructed = combine(&shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // The 'combine' function with invalid or insufficient shares.
    #[test]
    fn it_fails_to_combine_invalid_shares_input() {
        // Inconsistent shares
        let shares = vec![vec![1, 2], vec![3, 4, 3]];
        assert!(combine(&shares).is_err());

        // Invalid number of shares
        let shares = vec![vec![1, 2]];
        assert!(combine(&shares).is_err());
    }

    // The 'combine' function with duplicate shares.
    #[test]
    fn it_fails_to_combine_duplicate_shares() {
        let shares = vec![
            vec![35, 165, 19, 114, 53, 31, 70, 25, 74, 248, 145, 132],
            //duplicate shares
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
            vec![137, 206, 171, 244, 28, 176, 109, 4, 12, 168, 87, 50],
        ];

        assert!(combine(&shares).is_err());
    }
}