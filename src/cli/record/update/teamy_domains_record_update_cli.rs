use crate::cli::output::CliOutput;
use crate::domain::{DnsRecord, OfflineSnapshot, RecordChange, RecordPlan};
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Plan replacing one host row by its snapshot host ID offline.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RecordUpdateArgs {
    /// Path to a complete synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
    /// Host ID from the exact snapshot being planned.
    #[facet(args::named)]
    pub host_id: String,
    /// Replacement host label.
    #[facet(args::named)]
    pub host: String,
    /// A, AAAA or CNAME in this first planning slice.
    #[facet(args::named)]
    pub record_type: String,
    /// Replacement value.
    #[facet(args::named)]
    pub address: String,
    /// Replacement TTL in seconds, 60 through 60000.
    #[facet(args::named)]
    pub ttl: u32,
}
impl RecordUpdateArgs {
    /// # Errors
    /// Returns an error for unsafe, unsupported, or ambiguous plans.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        let snapshot = OfflineSnapshot::load(&self.snapshot)?;
        let record = DnsRecord {
            host_id: Some(self.host_id.clone()),
            host: self.host,
            record_type: self.record_type,
            address: self.address,
            ttl: self.ttl,
            mx_pref: None,
        };
        Ok(CliOutput::facet(RecordPlan::build(
            snapshot.zone,
            RecordChange::Update {
                host_id: self.host_id,
                replacement: record,
            },
        )?))
    }
}
