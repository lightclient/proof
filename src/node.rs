use super::NodeIndex;
use crate::path::PathElement;

/// Represents any valid node value.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub index: NodeIndex,
    pub ident: PathElement,
    pub size: u8,
    pub offset: u8,
    pub height: u64,
    pub is_list: bool,
}
