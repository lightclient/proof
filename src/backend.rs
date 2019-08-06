use super::NodeIndex;
use crate::error::{Error, Result};
use crate::tree_arithmetic::zeroed::expand_tree_index;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Stores the mapping of nodes to their chunks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Backend {
    db: HashMap<NodeIndex, Vec<u8>>,
}

impl Backend {
    /// Instantiate an empty `Cache`.
    pub fn new() -> Self {
        Self { db: HashMap::new() }
    }

    /// Gets a reference to the chunk coresponding to the node index.
    pub fn get(&self, index: NodeIndex) -> Option<&Vec<u8>> {
        self.db.get(&index)
    }

    /// Sets the chunk for the node index and returns the old value.
    pub fn insert(&mut self, index: NodeIndex, chunk: Vec<u8>) -> Option<Vec<u8>> {
        self.db.insert(index, chunk)
    }

    /// Retrieves a vector of loaded node indicies.
    pub fn nodes(&self) -> Vec<NodeIndex> {
        self.db.keys().clone().map(|x| x.to_owned()).collect()
    }

    /// Returns `true` if the cache contains a chunk for the specified node index.
    pub fn contains_node(&self, index: NodeIndex) -> bool {
        self.db.contains_key(&index)
    }

    /// Determines if the current merkle tree is valid.
    pub fn is_valid(&self, root: Vec<u8>) -> bool {
        for node in self.nodes() {
            let (left, right, parent) = expand_tree_index(node);

            if node > 1 {
                let left = self.get(left);
                let right = self.get(right);
                let parent = self.get(parent);

                if let (Some(left), Some(right), Some(parent)) = (left, right, parent) {
                    if hash_children(&left, &right) != *parent {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        &root == self.get(0).expect("Tree to have root node")
    }

    /// Inserts missing nodes into the merkle tree that can be generated from existing nodes.
    pub fn fill(&mut self) -> Result<()> {
        let mut nodes: Vec<u64> = self.nodes();
        nodes.sort_by(|a, b| b.cmp(a));

        let mut position = 0;
        while position < nodes.len() {
            let (left, right, parent) = expand_tree_index(nodes[position]);

            if self.contains_node(left) && self.contains_node(right) && !self.contains_node(parent)
            {
                let h = hash_children(
                    &self.get(left).ok_or(Error::ChunkNotLoaded(left))?,
                    &self.get(right).ok_or(Error::ChunkNotLoaded(right))?,
                );

                self.insert(parent, h);
                nodes.push(parent);
            }

            position += 1;
        }

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        let mut nodes: Vec<u64> = self.nodes();
        nodes.sort_by(|a, b| b.cmp(a));

        let mut position = 0;
        while nodes[position] > 0 {
            let (left, right, parent) = expand_tree_index(nodes[position]);

            if self.contains_node(left) && self.contains_node(right) {
                let h = hash_children(
                    &self.get(left).ok_or(Error::ChunkNotLoaded(left))?,
                    &self.get(right).ok_or(Error::ChunkNotLoaded(right))?,
                );

                self.insert(parent, h);
                nodes.push(parent);
            }

            position += 1;
        }

        Ok(())
    }
}

/// Helper function that appends `right` to `left` and hashes the result.
pub fn hash_children(left: &[u8], right: &[u8]) -> Vec<u8> {
    let children: Vec<u8> = left.iter().cloned().chain(right.iter().cloned()).collect();
    Sha256::digest(&children).as_ref().into()
}

impl std::ops::Index<usize> for Backend {
    type Output = Vec<u8>;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index as u64).expect("node acessible by index")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BYTES_PER_CHUNK;

    #[test]
    fn can_fill() {
        let mut db = Backend::default();

        // leaf nodes
        db.insert(6, vec![6; BYTES_PER_CHUNK]);
        db.insert(5, vec![5; BYTES_PER_CHUNK]);
        db.insert(4, vec![4; BYTES_PER_CHUNK]);
        db.insert(3, vec![3; BYTES_PER_CHUNK]);

        let two = hash_children(&db[5], &db[6]);
        let one = hash_children(&db[3], &db[4]);
        let root = hash_children(&one, &two);

        assert_eq!(db.fill(), Ok(()));
        assert_eq!(db.is_valid(root), true);
    }
    #[test]
    fn is_valid() {
        let mut db = Backend::default();

        // leaf nodes
        db.insert(6, vec![6; BYTES_PER_CHUNK]);
        db.insert(5, vec![5; BYTES_PER_CHUNK]);
        db.insert(4, vec![4; BYTES_PER_CHUNK]);
        db.insert(3, vec![3; BYTES_PER_CHUNK]);

        // intermediate nodes
        db.insert(2, hash_children(&db[5], &db[6]));
        db.insert(1, hash_children(&db[3], &db[4]));

        // root node
        let root = hash_children(&db[1], &db[2]);
        db.insert(0, root.clone());

        assert_eq!(db.is_valid(root), true);
    }
}
