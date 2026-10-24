use facet::Facet;

/// Registrar capabilities: domain identity and delegated nameservers.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct RegistrarSnapshot {
    pub domain: String,
    pub provider: String,
    pub nameservers: Vec<String>,
}

impl RegistrarSnapshot {
    /// # Errors
    /// Returns an error for malformed fixture state.
    pub fn validate(&self) -> eyre::Result<()> {
        eyre::ensure!(
            self.domain.contains('.') && !self.domain.chars().any(char::is_whitespace),
            "invalid domain"
        );
        eyre::ensure!(!self.provider.is_empty(), "registrar provider required");
        eyre::ensure!(
            !self.nameservers.is_empty()
                && self
                    .nameservers
                    .iter()
                    .all(|s| !s.is_empty() && !s.chars().any(char::is_whitespace)),
            "invalid nameservers"
        );
        Ok(())
    }
}
