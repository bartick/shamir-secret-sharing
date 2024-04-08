<h1 align="center"> Shamir Secret Sharing </h1>

<p align="center">
  <img src="https://img.shields.io/badge/Version-1.0-brightgreen" alt="Version">
  <img src="https://img.shields.io/badge/License-MIT-brightgreen" alt="License">
</p>

<p align="center">
  Shamir Secret Sharing is a cryptographic algorithm that allows you to split a secret into multiple parts and distribute them among a group of people. The secret can only be reconstructed when a minimum number of parts are combined together.
</p>

## 📝 Table of Contents
- [About](#about)
- [How it works](#how-it-works)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)

## 🧐 About
Shamir Secret Sharing is a cryptographic algorithm that allows you to split a secret into multiple parts and distribute them among a group of people. The secret can only be reconstructed when a minimum number of parts are combined together.

## 🚀 How it works

The algorithm works by generating a polynomial of degree `k-1` where `k` is the minimum number of parts required to reconstruct the secret. The polynomial is generated in such a way that the constant term is the secret itself. The polynomial is then evaluated at `n` different points to generate `n` parts. The secret can only be reconstructed when `k` parts are combined together.

## 📦 Usage

```js
const ref = require('ref-napi')
const ffi = require('ffi-napi')

const ArrayType = require('ref-array-di')(ref)
const StructType = require('ref-struct-di')(ref)

const Secret = StructType({
  value: stringPtr,
});
const ShareData = StructType({
    secrets: ArrayType(Secret),
    len: ref.types.size_t,
})
const ShareDataPtr = ref.refType(ShareData);

const libPath = 'libshamir.dylib'
const lib = ffi.Library(libPath, {
  split_string_c: [ ShareDataPtr, [ 'String', 'size_t', 'size_t' ] ],
  combile_string_c: [ stringPtr, [ShareDataPtr] ],
  clear_share_data: [ 'void', [ ShareDataPtr ] ],
  clear_string_c: [ 'void', [ stringPtr ] ],
  create_share_data: [ ShareDataPtr, ['String'] ],
  add_share_data: [ ShareDataPtr, [ ShareDataPtr, 'String' ] ],
});

const secret = 'Hello World';
const parts = 5;
const threshold = 3;

const dataRef = lib.split_string_c(secret, parts, threshold);

const data = dataRef.deref();

data.secrets.length = data.len;

for (let i = 0; i < data.len; i++) {
    console.log('Secret:', data.secrets[i].value.deref());
};

// Remember to always clear the data after you are done with it
lib.clear_share_data(dataRef);
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

