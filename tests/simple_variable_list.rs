use proof::node::Node;
use proof::types::VariableList;
use proof::{hash_children, Error, MerkleTreeOverlay, PathElement, Proof, SerializedProof};
use typenum::U4;

// S's merkle tree
//
//           root(0)
//          /       \
//    data_root(1) len(2)
//      /     \
// a[0,1](3) a[2,3](4)
#[derive(Debug, Default)]
struct S {
    a: VariableList<u128, U4>,
}

impl MerkleTreeOverlay for S {
    fn height() -> u64 {
        0
    }

    fn min_repr_size() -> u64 {
        32
    }

    fn is_list() -> bool {
        true
    }

    fn get_node(path: Vec<PathElement>) -> Result<Node, Error> {
        if Some(&PathElement::from_ident_str("a")) == path.first() {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("a"),
                    index: 0,
                    offset: 0,
                    size: 32,
                    height: VariableList::<u128, U4>::height().into(),
                    is_list: true,
                })
            } else {
                VariableList::<u128, U4>::get_node(path[1..].to_vec())
            }
        } else if let Some(p) = path.first() {
            Err(Error::InvalidPath(p.clone()))
        } else {
            Err(Error::EmptyPath())
        }
    }
}

#[test]
fn roundtrip_partial() {
    let mut chunk = [0_u8; 96];
    chunk[15] = 1;
    chunk[31] = 2;
    chunk[47] = 3;
    chunk[63] = 4;
    chunk[64..96].copy_from_slice(&hash_children(&[0; 32], &[0; 32]));

    let proof = SerializedProof {
        indices: vec![3, 4, 2],
        chunks: chunk.to_vec(),
    };

    let mut p = Proof::<S>::new(proof.clone());
    assert_eq!(p.fill(), Ok(()));

    assert_eq!(
        Ok(proof),
        p.extract(vec![
            PathElement::from_ident_str("a"),
            PathElement::Index(0)
        ])
    );
}
