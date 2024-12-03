// #![forbid(unsafe_code)]
// #![warn(clippy::all)]

mod ops;
mod polynomial;
mod split;
mod combine;

pub use split::split;
pub use combine::combine;

// Test cases for the `lib` module.
#[cfg(test)]
mod tests {
    use super::*;

    // The 'split' function with valid inputs.
    #[test]
    fn it_splits_secret() {
        let secret = "test_secret";
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
        let secret = "test_secret";
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
        let reconstructed = combine(shares).expect("combine failed");
        assert_eq!(reconstructed, secret);
    }

    // The 'combine' function with invalid or insufficient shares.
    #[test]
    fn it_fails_to_combine_invalid_shares_input() {
        // Inconsistent shares
        let shares = vec![vec![1, 2], vec![3, 4, 3]];
        assert!(combine(shares).is_err());

        // Invalid number of shares
        let shares = vec![vec![1, 2]];
        assert!(combine(shares).is_err());
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

        assert!(combine(shares).is_err());
    }
}