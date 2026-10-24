use crate::cli::output::CliOutput;
use crate::domain::OfflineSnapshot;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Show a domain and its separate registrar/DNS providers from an offline snapshot.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DomainInfoArgs {
    /// Path to a synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
}
impl DomainInfoArgs {
    /// # Errors
    /// Returns an error for invalid offline input.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(OfflineSnapshot::load(&self.snapshot)?))
    }
}
