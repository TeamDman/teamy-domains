use super::{DnsZoneSnapshot, RegistrarSnapshot};
use eyre::Context;
use facet::Facet;
use std::path::Path;

/// Synthetic fixture with distinct registrar and DNS-provider state.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct OfflineSnapshot {
    pub registrar: RegistrarSnapshot,
    pub zone: DnsZoneSnapshot,
}

impl OfflineSnapshot {
    /// # Errors
    /// Returns an error for unreadable, oversized, or malformed fixture data.
    pub fn load(path: &str) -> eyre::Result<Self> {
        let path = Path::new(path);
        let metadata = std::fs::metadata(path).wrap_err("cannot read offline snapshot metadata")?;
        eyre::ensure!(
            metadata.is_file() && metadata.len() <= 1_048_576,
            "offline snapshot must be a file of at most 1 MiB"
        );
        let raw = std::fs::read_to_string(path).wrap_err("cannot read offline snapshot")?;
        let snapshot: Self =
            facet_json::from_str(&raw).wrap_err("invalid offline snapshot JSON")?;
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// # Errors
    /// Returns an error if registrar and DNS state refer to different domains.
    pub fn validate(&self) -> eyre::Result<()> {
        self.registrar.validate()?;
        self.zone.validate()?;
        eyre::ensure!(
            self.registrar.domain == self.zone.domain,
            "registrar and DNS domains differ"
        );
        Ok(())
    }
}
