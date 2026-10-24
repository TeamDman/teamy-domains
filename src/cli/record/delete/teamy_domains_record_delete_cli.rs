use crate::cli::output::CliOutput;
use crate::domain::{OfflineSnapshot, RecordChange, RecordPlan};
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Plan deleting one host row by its snapshot host ID offline.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RecordDeleteArgs {
    /// Path to a complete synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
    /// Host ID from the exact snapshot being planned.
    #[facet(args::named)]
    pub host_id: String,
}
impl RecordDeleteArgs {
    /// # Errors
    /// Returns an error for unsafe, unsupported, or ambiguous plans.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        let snapshot = OfflineSnapshot::load(&self.snapshot)?;
        Ok(CliOutput::facet(RecordPlan::build(
            snapshot.zone,
            RecordChange::Delete {
                host_id: self.host_id,
            },
        )?))
    }
}
