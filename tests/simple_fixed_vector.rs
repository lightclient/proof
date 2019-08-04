use ethereum_types::U256;
use proof::field::{Composite, Node};
use proof::{Error, MerkleTreeOverlay, Proof, Path, SerializedProof, hash_children};
use ssz_types::FixedVector;
use typenum::U4;

// S's merkle tree
//
//        c_root(0)
//       /         \
//     i(1)       i(2)
//     /  \       /  \
//   a[0] a[1]  a[2] a[3]
#[derive(Debug, Default)]
struct S {
    a: FixedVector<U256, U4>,
}

// Implemented by derive macro
impl MerkleTreeOverlay for S {
    fn height() -> u8 {
        0
    }

    fn get_node(path: Vec<Path>) -> Result<Node, Error> {
        if Some(&Path::Ident("a".to_string())) == path.first() {
            if path.len() == 1 {
                Ok(Node::Composite(Composite {
                    ident: "a".to_owned(),
                    index: 0,
                    height: FixedVector::<U256, U4>::height().into(),
                }))
            } else {
                FixedVector::<U256, U4>::get_node(path[1..].to_vec())
            }
        } else if let Some(p) = path.first() {
            Err(Error::InvalidPath(p.clone()))
        } else {
            Err(Error::EmptyPath())
        }
    }
}

#[test]
fn get_partial_vector() {
    let mut chunk = [0_u8; 96];
    chunk[31] = 1;
    chunk[64..96].copy_from_slice(&hash_children(&[0; 32], &[0; 32]));

    let proof = SerializedProof {
        indices: vec![5, 6, 1],
        chunks: chunk.to_vec(),
    };

    let mut p = Proof::<S>::new(proof.clone());
    assert_eq!(p.fill(), Ok(()));
    assert_eq!(
        Ok(proof),
        p.extract(vec![Path::Ident("a".to_string()), Path::Index(2)])
    );
}
