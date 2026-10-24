use crate::cli::output::CliOutput;
use crate::domain::OfflineSnapshot;
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// List domains from one synthetic offline snapshot.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct DomainListArgs {
    /// Path to a synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
}
impl DomainListArgs {
    /// # Errors
    /// Returns an error for invalid offline input.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        let snapshot = OfflineSnapshot::load(&self.snapshot)?;
        Ok(CliOutput::facet(vec![snapshot.registrar]))
    }
}
