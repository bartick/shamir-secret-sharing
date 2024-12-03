<h1 align="center"> Shamir Secret Sharing </h1>

<p align="center">
  <img src="https://img.shields.io/badge/Version-1.0-brightgreen" alt="Version">
  <img src="https://img.shields.io/badge/License-MIT-brightgreen" alt="License">
</p>

<p align="center">
  Shamir Secret Sharing is a cryptographic algorithm that allows you to split a secret into multiple parts and distribute them among a group of people. The secret can only be reconstructed when a minimum number of parts are combined together.
</p>

## 📝 Table of Contents
- [About](#-about)
- [How it works](#-how-it-works)
- [Usage](#-usage)
- [Contributing](#-contributing)
- [License](#-license)

## 🧐 About
Shamir Secret Sharing is a cryptographic algorithm that allows you to split a secret into multiple parts and distribute them among a group of people. The secret can only be reconstructed when a minimum number of parts are combined together.

## 🚀 How it works

The algorithm works by generating a polynomial of degree `k-1` where `k` is the minimum number of parts required to reconstruct the secret. The polynomial is generated in such a way that the constant term is the secret itself. The polynomial is then evaluated at `n` different points to generate `n` parts. The secret can only be reconstructed when `k` parts are combined together.

## 📦 Usage

```rust
use shamir::{split, combine};

fn main() {
    // Define the secret to be split.
    let secret = "Hello, World!";

    // Split the secret into 5 shares with a threshold of 3.
    let shares = split(secret, 5, 3).unwrap();

    // Display the shares.
    for (idx, share) in shares.iter().enumerate() {
        println!("Share {}: {:?}", idx + 1, share);
    }

    // Combine the shares to reconstruct the secret.
    let reconstructed = combine(&shares[0..3]).unwrap();
    println!("Reconstructed: {}", String::from_utf8(reconstructed).unwrap());
}
```

## 🤝 Contributing
Contributions, issues and feature requests are welcome. After cloning & setting up project locally, you can just submit a PR to this repo and it will be deployed once it's accepted.

## 📝 License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

```bash
# Clone this repository
$ git clone

# Go into the repository
$ cd shamir-secret-sharing

# Build the project
$ cargo build
```

