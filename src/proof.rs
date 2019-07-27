use super::{NodeIndex, SerializedProof, BYTES_PER_CHUNK};
use crate::cache::Cache;
use crate::error::{Error, Result};
use crate::field::Node;
use crate::merkle_tree_overlay::MerkleTreeOverlay;
use crate::path::Path;
use crate::tree_arithmetic::zeroed::sibling_index;

use std::marker::PhantomData;

/// A `Proof` is generated from a `SerializedProof` and can manipulate / verify data in the
/// merkle tree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Proof<T: MerkleTreeOverlay> {
    cache: Cache,
    _phantom: PhantomData<T>,
}

impl<T: MerkleTreeOverlay> Proof<T> {
    /// Initialize `Proof` directly from a `SerializedProof`.
    pub fn new(proof: SerializedProof) -> Self {
        let mut ret = Self {
            cache: Cache::new(),
            _phantom: PhantomData,
        };

        // This will always return `Ok(())` since the `cache` is starting empty.
        ret.load(proof).unwrap();

        ret
    }

    /// Populate the struct's cache with a `SerializedProof`.
    pub fn load(&mut self, proof: SerializedProof) -> Result<()> {
        for (i, index) in proof.indices.iter().enumerate() {
            let chunk = proof.chunks[i * BYTES_PER_CHUNK..(i + 1) * BYTES_PER_CHUNK].to_vec();
            self.cache.insert(*index, chunk.clone());
        }

        Ok(())
    }

    /// Generates a `SerializedProof` proving that `path` is a part of the current merkle tree.
    pub fn extract(&self, path: Vec<Path>) -> Result<SerializedProof> {
        if path.len() == 0 {
            return Err(Error::EmptyPath());
        }

        let node = T::get_node(path.clone())?;

        let mut visitor = node.get_index();
        let mut indices: Vec<NodeIndex> = vec![visitor];
        let mut chunks: Vec<u8> = self
            .cache
            .get(visitor)
            .ok_or(Error::ChunkNotLoaded(visitor))?
            .clone();

        while visitor > 0 {
            let sibling = sibling_index(visitor);
            let left = 2 * sibling + 1;
            let right = 2 * sibling + 2;

            if !(indices.contains(&left) && indices.contains(&right)) {
                indices.push(sibling);
                chunks.extend(
                    self.cache
                        .get(sibling)
                        .ok_or(Error::ChunkNotLoaded(sibling))?,
                );
            }

            // visitor /= 2, when 1 indexed
            visitor = (visitor + 1) / 2 - 1;
        }

        Ok(SerializedProof { indices, chunks })
    }

    /// Returns the bytes representation of the object associated with `path`
    pub fn get_bytes(&self, path: Vec<Path>) -> Result<Vec<u8>> {
        if path.len() == 0 {
            return Err(Error::EmptyPath());
        }

        let (index, begin, end) = bytes_at_path_helper::<T>(path)?;

        Ok(self.cache.get(index).ok_or(Error::ChunkNotLoaded(index))?[begin..end].to_vec())
    }

    /// Replaces the bytes at `path` with `bytes`.
    pub fn set_bytes(&mut self, path: Vec<Path>, bytes: Vec<u8>) -> Result<()> {
        if path.len() == 0 {
            return Err(Error::EmptyPath());
        }

        let (index, begin, end) = bytes_at_path_helper::<T>(path)?;

        let chunk = self
            .cache
            .get(index)
            .ok_or(Error::ChunkNotLoaded(index))?
            .to_vec()
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, b)| {
                if i >= begin && i < end {
                    bytes[i - begin]
                } else {
                    b
                }
            })
            .collect();

        self.cache.insert(index, chunk);
        Ok(())
    }

    /// Determines if the current merkle tree is valid.
    pub fn is_valid(&self, root: Vec<u8>) -> bool {
        self.cache.is_valid(root)
    }

    /// Inserts missing nodes into the merkle tree that can be generated from existing nodes.
    pub fn fill(&mut self) -> Result<()> {
        self.cache.fill()
    }

    /// Returns the root node of the proof if it has been calculated.
    pub fn root(&self) -> Option<&Vec<u8>> {
        self.cache.get(0)
    }

    /// Recalculates all intermediate nodes and root using the available leaves.
    pub fn refresh(&mut self) -> Result<()> {
        self.cache.refresh()
    }
}

/// Recursively traverse the tree structure, matching the appropriate `path` element with its index,
/// eventually returning the chunk index, beginning offset, and end offset of the associated value.
fn bytes_at_path_helper<T: MerkleTreeOverlay + ?Sized>(
    path: Vec<Path>,
) -> Result<(NodeIndex, usize, usize)> {
    if path.len() == 0 {
        return Err(Error::EmptyPath());
    }

    match T::get_node(path.clone())? {
        Node::Composite(c) => Ok((c.index, 0, 32)),
        Node::Length(l) => Ok((l.index, 0, 32)),
        Node::Primitive(l) => {
            for p in l.clone() {
                if p.ident == path.last().unwrap().to_string() {
                    return Ok((p.index, p.offset as usize, (p.offset + p.size) as usize));
                }
            }

            Err(Error::InvalidPath(path[0].clone()))
        }
    }
}