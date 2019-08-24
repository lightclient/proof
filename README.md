# Proof
[![Build Status](https://travis-ci.com/lightclient/proof.svg?branch=master)](https://travis-ci.com/lightclient/proof)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](https://github.com/c-o-l-o-r/proof#license)

`Proof` is library for validating and manipulating [merkle proofs](https://blog.ethereum.org/2015/11/15/merkling-in-ethereum/)
of partial objects. 

The library conforms with the evolving Ethereum 2.0 
[specification](https://github.com/ethereum/eth2.0-specs/blob/dev/specs/light_client/merkle_proofs.md#merklepartial)
merkle proof partials. Until version `1.0`, expect the API to be unstable.

## Getting Started
Add the following to your projects `Cargo.toml` under `[dependencies]`:

```
proof = { git = "https://github.com/c-o-l-o-r/proof" }
proof_derive = { git = "https://github.com/c-o-l-o-r/proof" }
```
If you plan to use `ssz_types`, also add:

```
ssz_types = { git = "https://github.com/sigp/lighthouse" }
```

## Example
```rust
// S's merkle tree representation
//
//            root(0)
//          /        \
//        a(1)        b(2) -----+
//                   /           \
//            +-- data(5) --+     len(6)
//           /               \
//      i(11)                i(12)
//    /      \             /       \
// b[0,1](23) b[2,3](24) b[4,5](26) b[6,7](27)

use proof::{Proof, Path, SerializedProof, hash_children};

#[derive(Provable)]
struct S {
    a: u64,
    b: VariableList<u128, U8>,
}

fn main() {
    // Build the 32-byte chunks
    let one = vec![0u8; 32];
    let six = vec![0u8; 32];
    let twelve = hash_children(&[0u8; 32], &[0u8; 32]);
    let twenty_three = vec![1u8; 32];
    let twenty_four = vec![2u8; 32];

    // Generate the proof
    let serialized_proof = SerializedProof {
        indices: vec![1, 6, 12, 23, 24],
        chunks: vec![one, six, twelve, twenty_three, twenty_four].into_iter().flatten().collect(),
    };

    // Load the proof
    let mut proof = Proof::<S>::new(serialized_proof.clone());

    // Fill in chunks that can be inferred
    assert_eq!(proof.fill(), Ok(()));

    // Extract a proof to `S.b[2]`
    assert_eq!(
        proof.extract(vec![Path::Ident("b".to_string()), Path::Index(2)]),
        Ok(serialized_proof)
    );
}
```

For additional usage examples, see the [tests](tests/) directory.

## License
Licensed under Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
