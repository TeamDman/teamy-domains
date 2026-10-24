use crate::cli::output::CliOutput;
use crate::domain::{NameserverPlan, OfflineSnapshot};
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Plan registrar nameserver delegation offline; this never applies it.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct NameserverSetArgs {
    /// Path to a synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
    /// Comma-separated nameserver hostnames, in intended order.
    #[facet(args::named)]
    pub nameservers: String,
}
impl NameserverSetArgs {
    /// # Errors
    /// Returns an error for invalid or unchanged nameservers.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        let snapshot = OfflineSnapshot::load(&self.snapshot)?;
        let nameservers = self.nameservers.split(',').map(str::to_owned).collect();
        Ok(CliOutput::facet(NameserverPlan::build(
            snapshot.registrar,
            nameservers,
        )?))
    }
}
