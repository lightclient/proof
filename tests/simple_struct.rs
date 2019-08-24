use ethereum_types::U256;
use proof::node::Node;
use proof::{hash_children, Error, MerkleTreeOverlay, PathElement, Proof, SerializedProof};

// A's merkle tree
//
//        a_root(0)
//       /         \
//     i(1)       i(2)
//    /   \       /   \
//  a(3) b(4)  c,d(5) p(6)
//
// n(i) => n = node type, i = general index
// i = intermediate, p = padding
#[derive(Debug, Default)]
struct S {
    a: U256,
    b: U256,
    c: u128,
    d: u128,
}

// Implemented by derive macro
impl MerkleTreeOverlay for S {
    fn height() -> u64 {
        2
    }

    fn min_repr_size() -> u64 {
        32
    }

    fn is_list() -> bool {
        false
    }

    fn get_node(path: Vec<PathElement>) -> Result<Node, Error> {
        let p1 = path.first();

        if p1 == Some(&PathElement::from_ident_str("a")) {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("a"),
                    index: 3,
                    size: 32,
                    height: 0,
                    offset: 0,
                    is_list: false,
                })
            } else {
                // not sure if this will work
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&PathElement::from_ident_str("b")) {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("b"),
                    index: 4,
                    size: 32,
                    offset: 0,
                    height: 0,
                    is_list: false,
                })
            } else {
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&PathElement::from_ident_str("c")) {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("c"),
                    index: 5,
                    size: 16,
                    offset: 0,
                    height: 0,
                    is_list: false,
                })
            } else {
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&PathElement::from_ident_str("d")) {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("d"),
                    index: 5,
                    size: 16,
                    offset: 16,
                    height: 0,
                    is_list: false,
                })
            } else {
                U256::get_node(path[1..].to_vec())
            }
        } else if let Some(p) = p1 {
            Err(Error::InvalidPath(p.clone()))
        } else {
            Err(Error::EmptyPath())
        }
    }
}

#[test]
fn roundtrip_partial() {
    let mut arr = [0_u8; 96];

    let three = U256::from(1);
    let four = U256::from(2);
    four.to_little_endian(&mut arr[0..32]);
    three.to_little_endian(&mut arr[32..64]);

    let two: &[u8] = &hash_children(&arr[64..96], &arr[64..96]);
    arr[64..96].copy_from_slice(two);

    let sp = SerializedProof {
        indices: vec![4, 3, 2],
        chunks: arr.to_vec(),
    };

    let mut p = Proof::<S>::new(sp.clone());
    assert_eq!(p.fill(), Ok(()));
    assert_eq!(Ok(sp), p.extract(vec![PathElement::from_ident_str("a")]));
}
