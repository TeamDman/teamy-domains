use super::{NamecheapAdapterError, NamecheapRequestFields};

/// Injectable offline transport boundary. No production implementation exists.
pub trait NamecheapTransport {
    /// # Errors
    /// Returns a typed, redacted transport failure.
    fn exchange(&self, request: &NamecheapRequestFields) -> Result<String, NamecheapAdapterError>;
}
