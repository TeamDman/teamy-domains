use super::NamecheapAdapterError;

/// Opaque Namecheap `getHosts` row selector. It is not a write precondition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamecheapHostId(String);

impl NamecheapHostId {
    /// # Errors
    /// Rejects empty, oversized or control-bearing provider IDs.
    pub fn new(raw: &str) -> Result<Self, NamecheapAdapterError> {
        if raw.is_empty()
            || raw.len() > 64
            || raw.chars().any(|c| c.is_control() || c.is_whitespace())
        {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        Ok(Self(raw.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
