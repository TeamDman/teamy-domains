use super::{NamecheapAdapterError, NamecheapXmlNode};

/// XML tokenization boundary. No production implementation is enabled until dependency review.
pub trait NamecheapXmlDecoder {
    /// # Errors
    /// Rejects malformed or oversized XML without exposing raw contents in errors.
    fn decode(&self, xml: &str) -> Result<NamecheapXmlNode, NamecheapAdapterError>;
}
