use crate::cli::output::CliOutput;
use crate::domain::OfflineSnapshot;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Show registrar delegation from a synthetic offline snapshot.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct NameserverListArgs {
    /// Path to a synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
}
impl NameserverListArgs {
    /// # Errors
    /// Returns an error for invalid offline input.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(
            OfflineSnapshot::load(&self.snapshot)?.registrar,
        ))
    }
}
