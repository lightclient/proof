use ethereum_types::U256;
use proof::node::Node;
use proof::types::FixedVector;
use proof::{hash_children, Error, MerkleTreeOverlay, PathElement, Proof, SerializedProof};
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
    fn height() -> u64 {
        0
    }

    fn min_repr_size() -> u64 {
        32
    }

    fn is_list() -> bool {
        false
    }

    fn get_node(path: Vec<PathElement>) -> Result<Node, Error> {
        if Some(&PathElement::from_ident_str("a")) == path.first() {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("a"),
                    index: 0,
                    offset: 0,
                    size: 32,
                    height: FixedVector::<U256, U4>::height().into(),
                    is_list: false,
                })
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
        p.extract(vec![
            PathElement::from_ident_str("a"),
            PathElement::Index(2)
        ])
    );
}
