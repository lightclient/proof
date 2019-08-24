use proof::impls::replace_index;
use proof::node::Node;
use proof::tree_arithmetic::zeroed::subtree_index_to_general;
use proof::types::{FixedVector, VariableList};
use proof::{hash_children, Error, MerkleTreeOverlay, PathElement, Proof, SerializedProof};
use typenum::{U32, U8};

#[derive(Debug, Default)]
struct Message {
    timestamp: u64,
    message: FixedVector<u8, U32>,
}

#[derive(Debug, Default)]
struct State {
    messages: VariableList<Message, U8>,
}

impl MerkleTreeOverlay for Message {
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
        if Some(&PathElement::from_ident_str("timestamp")) == path.first() {
            Ok(Node {
                ident: PathElement::from_ident_str("timestamp"),
                index: 1,
                offset: 0,
                size: 8,
                height: 0,
                is_list: false,
            })
        } else if Some(&PathElement::from_ident_str("message")) == path.first() {
            match FixedVector::<u8, U32>::get_node(path[1..].to_vec()) {
                Ok(n) => Ok(replace_index(
                    n.clone(),
                    subtree_index_to_general(2, n.index),
                )),
                e => e,
            }
        } else if let Some(p) = path.first() {
            Err(Error::InvalidPath(p.clone()))
        } else {
            Err(Error::EmptyPath())
        }
    }
}

impl MerkleTreeOverlay for State {
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
        if Some(&PathElement::from_ident_str("messages")) == path.first() {
            if path.len() == 1 {
                Ok(Node {
                    ident: PathElement::from_ident_str("messages"),
                    index: 0,
                    offset: 0,
                    size: 0,
                    height: VariableList::<Message, U8>::height().into(),
                    is_list: true,
                })
            } else {
                VariableList::<Message, U8>::get_node(path[1..].to_vec())
            }
        } else if let Some(p) = path.first() {
            Err(Error::InvalidPath(p.clone()))
        } else {
            Err(Error::EmptyPath())
        }
    }
}

fn zero_hash(depth: u8) -> Vec<u8> {
    if depth == 0 {
        vec![0; 32]
    } else if depth == 1 {
        hash_children(&[0; 32], &[0; 32])
    } else {
        let last = zero_hash(depth - 1);
        hash_children(&last, &last)
    }
}

#[test]
fn roundtrip_partial() {
    let mut arr = vec![0; 224];

    // 31 `message[0].timestamp`
    arr[0] = 1;

    // 32 `message[0].message`
    arr[32..64].copy_from_slice(&vec![1_u8; 32]);

    // 33 `message[1].timestamp`
    arr[64] = 2;

    // 34 `message[1].message`
    arr[96..128].copy_from_slice(&vec![42_u8; 32]);

    // 8 `hash of message[2] and message[3]`
    arr[128..160].copy_from_slice(&zero_hash(2));

    // 4 `hash of message[4..7]`
    arr[160..192].copy_from_slice(&zero_hash(3));

    // 2 length mixin
    arr[223] = 2;

    let sp = SerializedProof {
        indices: vec![31, 32, 33, 34, 8, 4, 2],
        chunks: arr.clone(),
    };

    let mut proof = Proof::<State>::new(sp);
    assert_eq!(proof.fill(), Ok(()));

    // TESTING TIMESTAMPS
    assert_eq!(
        proof.get_bytes(vec![
            PathElement::from_ident_str("messages"),
            PathElement::Index(0),
            PathElement::from_ident_str("timestamp"),
        ]),
        Ok(vec![1, 0, 0, 0, 0, 0, 0, 0])
    );

    assert_eq!(
        proof.get_bytes(vec![
            PathElement::from_ident_str("messages"),
            PathElement::Index(1),
            PathElement::from_ident_str("timestamp"),
        ]),
        Ok(vec![2, 0, 0, 0, 0, 0, 0, 0])
    );

    // TESTING MESSAGES
    assert_eq!(
        proof.get_bytes(vec![
            PathElement::from_ident_str("messages"),
            PathElement::Index(0),
            PathElement::from_ident_str("message"),
            PathElement::Index(1),
        ]),
        Ok(vec![1])
    );

    assert_eq!(
        proof.get_bytes(vec![
            PathElement::from_ident_str("messages"),
            PathElement::Index(1),
            PathElement::from_ident_str("message"),
            PathElement::Index(31),
        ]),
        Ok(vec![42])
    );
}
