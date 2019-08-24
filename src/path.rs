use crate::NodeIndex;

/// An identifier for the location of a distinct value in a partial.
#[derive(Clone, Debug, PartialEq)]
pub enum PathElement {
    /// An identifier for a member of a container object or for the length of a list.
    Ident(String),
    /// An identifier for the position of a value in a homogeneous collection.
    Index(NodeIndex),
}

impl PathElement {
    pub fn from_ident_str<S>(ident: S) -> PathElement
    where
        S: Into<String>,
    {
        PathElement::Ident(ident.into())
    }
}

impl std::fmt::Display for PathElement {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PathElement::Ident(s) => fmt.write_str(s),
            PathElement::Index(i) => fmt.write_str(&i.to_string()),
        }?;

        Ok(())
    }
}

impl From<&str> for PathElement {
    fn from(s: &str) -> PathElement {
        match s.parse::<u64>() {
            Ok(n) => PathElement::Index(n),
            Err(_) => PathElement::from_ident_str(s),
        }
    }
}
