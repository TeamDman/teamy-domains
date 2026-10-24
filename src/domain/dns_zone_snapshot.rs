use super::DnsRecord;
use facet::Facet;

/// Authoritative DNS-provider state, separate from registrar delegation.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct DnsZoneSnapshot {
    pub domain: String,
    pub provider: String,
    pub complete: bool,
    pub records: Vec<DnsRecord>,
}

impl DnsZoneSnapshot {
    /// # Errors
    /// Returns an error for malformed fixture state.
    pub fn validate(&self) -> eyre::Result<()> {
        eyre::ensure!(
            self.domain.contains('.') && !self.domain.chars().any(char::is_whitespace),
            "invalid zone domain"
        );
        eyre::ensure!(!self.provider.is_empty(), "DNS provider required");
        Ok(())
    }
}
