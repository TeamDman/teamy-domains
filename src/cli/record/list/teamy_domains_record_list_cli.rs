use crate::cli::output::CliOutput;
use crate::domain::OfflineSnapshot;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Show an authoritative provider's zone from a synthetic offline snapshot.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RecordListArgs {
    /// Path to a synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
}
impl RecordListArgs {
    /// # Errors
    /// Returns an error for invalid offline input.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(
            OfflineSnapshot::load(&self.snapshot)?.zone,
        ))
    }
}
