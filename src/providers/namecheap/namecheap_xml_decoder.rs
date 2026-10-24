use super::{NamecheapAdapterError, NamecheapXmlNode};

/// XML tokenization boundary. The exact-source Facet implementation is qualified
/// in a separate test project; no live transport or production decoder is enabled.
pub trait NamecheapXmlDecoder {
    /// # Errors
    /// Rejects malformed or oversized XML without exposing raw contents in errors.
    fn decode(&self, xml: &str) -> Result<NamecheapXmlNode, NamecheapAdapterError>;
}
