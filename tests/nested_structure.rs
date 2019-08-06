use ethereum_types::U256;
use proof::impls::replace_index;
use proof::node::Node;
use proof::tree_arithmetic::zeroed::subtree_index_to_general;
use proof::types::VariableList;
use proof::{hash_children, Error, MerkleTreeOverlay, PathElement, Proof, SerializedProof};
use typenum::U8;

// S's merkle tree
//
//         root(0)
//        /       \
//      a(1)      b(2)
//                 /   \
//           data(5) len(6)
//           /      \
//      i(11)        i(12)
//      /   \       /     \
//  b0(23) b2(24) b4(26) b6(27)
#[derive(Debug, Default)]
struct S {
    a: U256,
    b: VariableList<u128, U8>,
}

impl MerkleTreeOverlay for S {
    fn height() -> u64 {
        1
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
                    index: 1,
                    size: 32,
                    offset: 0,
                    height: 0,
                    is_list: false,
                })
            } else {
                match U256::get_node(path[1..].to_vec()) {
                    Ok(n) => Ok(replace_index(
                        n.clone(),
                        subtree_index_to_general(1, n.index),
                    )),
                    e => e,
                }
            }
        } else if Some(&PathElement::from_ident_str("b")) == path.first() {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("b"),
                    index: 2,
                    offset: 0,
                    size: 0,
                    height: 3,
                    is_list: true,
                })
            } else {
                match VariableList::<u128, U8>::get_node(path[1..].to_vec()) {
                    Ok(n) => Ok(replace_index(
                        n.clone(),
                        subtree_index_to_general(2, n.index),
                    )),
                    e => e,
                }
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
    let mut arr = [0_u8; 160];
    arr[15] = 0;
    arr[31] = 1;
    arr[47] = 2;
    arr[63] = 3;
    arr[127] = 3; // length

    let twelve: &[u8] = &hash_children(&arr[64..96], &arr[64..96]);

    arr[64..96].copy_from_slice(twelve);

    let sp = SerializedProof {
        indices: vec![23, 24, 12, 6, 1],
        chunks: arr.to_vec(),
    };

    let mut p = Proof::<S>::default();

    assert_eq!(p.load(sp.clone()), Ok(()));
    assert_eq!(p.fill(), Ok(()));
    assert_eq!(
        p.extract(vec![
            PathElement::Ident("b".to_string()),
            PathElement::Index(2)
        ]),
        Ok(sp)
    );

    // Check for `Error::ChunkNotLoaded(_)`
    let generate_path = || vec![PathElement::Ident("b".to_string()), PathElement::Index(5)];

    assert_eq!(p.get_bytes(generate_path()), Err(Error::ChunkNotLoaded(25)));
    assert_eq!(
        p.set_bytes(generate_path(), vec![]),
        Err(Error::ChunkNotLoaded(25))
    );
}

#[test]
fn get_and_set_by_path() {
    let mut arr = [0_u8; 160];

    arr[31] = 1;
    arr[47] = 0;
    arr[63] = 1;
    arr[79] = 2;
    arr[95] = 3;
    arr[111] = 4;
    arr[127] = 5;
    arr[143] = 6;
    arr[159] = 7;

    let sp = SerializedProof {
        indices: vec![1, 23, 24, 25, 26],
        chunks: arr.to_vec(),
    };

    let mut p = Proof::<S>::new(sp.clone());

    // Check S.a
    assert_eq!(
        p.get_bytes(vec![PathElement::from_ident_str("a")]),
        Ok(arr[0..32].to_vec())
    );

    // Check S.b[0..8] and set each to S.b[7]
    for i in 0_usize..8_usize {
        assert_eq!(
            p.get_bytes(vec![
                PathElement::from_ident_str("b"),
                PathElement::Index(i as u64)
            ]),
            Ok(arr[(32 + i * 16)..(32 + ((i + 1) * 16))].to_vec())
        );

        assert_eq!(
            p.set_bytes(
                vec![
                    PathElement::from_ident_str("b"),
                    PathElement::Index(i as u64)
                ],
                arr[144..160].to_vec()
            ),
            Ok(()),
        );
    }

    // Verfiy that each index was set to S.b[7]
    for i in 0_usize..8_usize {
        assert_eq!(
            p.get_bytes(vec![
                PathElement::from_ident_str("b"),
                PathElement::Index(i as u64)
            ]),
            Ok(arr[144..160].to_vec())
        );
    }

    // Check for `Error::EmptyPath()`
    assert_eq!(p.get_bytes(vec![]), Err(Error::EmptyPath()),);
    assert_eq!(p.set_bytes(vec![], vec![]), Err(Error::EmptyPath()));

    // Check for `Error::OutOfBounds(Path::Index(_))`
    let generate_path = || vec![PathElement::from_ident_str("b"), PathElement::Index(8)];

    assert_eq!(
        p.get_bytes(generate_path()),
        Err(Error::IndexOutOfBounds(8))
    );
    assert_eq!(
        p.set_bytes(generate_path(), vec![]),
        Err(Error::IndexOutOfBounds(8))
    );

    // Check for `Error::InvalidPath(Path::Ident(_))`
    let generate_path = || vec![PathElement::from_ident_str("c")];

    assert_eq!(
        p.get_bytes(generate_path()),
        Err(Error::InvalidPath(generate_path()[0].clone()))
    );
    assert_eq!(
        p.set_bytes(generate_path(), vec![]),
        Err(Error::InvalidPath(generate_path()[0].clone()))
    );
}

#[test]
fn readme_test() {
    let one = vec![0u8; 32];
    let six = vec![0u8; 32];
    let twelve = hash_children(&[0u8; 32], &[0u8; 32]);
    let twenty_three = vec![1u8; 32];
    let twenty_four = vec![2u8; 32];

    let serialized_proof = SerializedProof {
        indices: vec![1, 6, 12, 23, 24],
        chunks: vec![one, six, twelve, twenty_three, twenty_four]
            .into_iter()
            .flatten()
            .collect(),
    };
    let mut proof = Proof::<S>::new(serialized_proof.clone());

    assert_eq!(proof.fill(), Ok(()));
    assert_eq!(
        proof.extract(vec![
            PathElement::from_ident_str("b"),
            PathElement::Index(2)
        ]),
        Ok(serialized_proof)
    );
}
