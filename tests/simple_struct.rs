use ethereum_types::U256;
use proof::cache::hash_children;
use proof::field::{Node, Primitive};
use proof::{Error, MerkleTreeOverlay, Proof, Path, SerializedProof};

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
    fn height() -> u8 {
        2
    }

    fn get_node(path: Vec<Path>) -> Result<Node, Error> {
        let p1 = path.first();

        if p1 == Some(&Path::Ident("a".to_string())) {
            if path.len() == 1 {
                Ok(Node::Primitive(vec![Primitive {
                    ident: "a".to_owned(),
                    index: 3,
                    size: 32,
                    offset: 0,
                }]))
            } else {
                // not sure if this will work
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&Path::Ident("b".to_string())) {
            if path.len() == 1 {
                Ok(Node::Primitive(vec![Primitive {
                    ident: "b".to_owned(),
                    index: 4,
                    size: 32,
                    offset: 0,
                }]))
            } else {
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&Path::Ident("c".to_string())) {
            if path.len() == 1 {
                Ok(Node::Primitive(vec![
                    Primitive {
                        ident: "c".to_owned(),
                        index: 5,
                        size: 16,
                        offset: 0,
                    },
                    Primitive {
                        ident: "d".to_owned(),
                        index: 5,
                        size: 16,
                        offset: 16,
                    },
                ]))
            } else {
                U256::get_node(path[1..].to_vec())
            }
        } else if p1 == Some(&Path::Ident("d".to_string())) {
            if path.len() == 1 {
                Ok(Node::Primitive(vec![
                    Primitive {
                        ident: "c".to_owned(),
                        index: 5,
                        size: 16,
                        offset: 0,
                    },
                    Primitive {
                        ident: "d".to_owned(),
                        index: 5,
                        size: 16,
                        offset: 16,
                    },
                ]))
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
    assert_eq!(
        Ok(sp),
        p.extract(vec![Path::Ident("a".to_string())])
    );
}
