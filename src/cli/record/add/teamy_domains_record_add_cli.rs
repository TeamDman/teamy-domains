use crate::cli::output::CliOutput;
use crate::domain::{DnsRecord, OfflineSnapshot, RecordChange, RecordPlan};
use arbitrary::Arbitrary;
use facet::Facet;
use figue as args;

/// Plan adding one Namecheap `BasicDNS` A, AAAA or CNAME host row offline.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RecordAddArgs {
    /// Path to a complete synthetic offline JSON snapshot.
    #[facet(args::named)]
    pub snapshot: String,
    /// Namecheap host label, such as docs or @.
    #[facet(args::named)]
    pub host: String,
    /// A, AAAA or CNAME in this first planning slice.
    #[facet(args::named)]
    pub record_type: String,
    /// New record value.
    #[facet(args::named)]
    pub address: String,
    /// Time to live in seconds, 60 through 60000.
    #[facet(args::named)]
    pub ttl: u32,
}
impl RecordAddArgs {
    /// # Errors
    /// Returns an error for unsafe, unsupported, or incomplete plans.
    pub fn invoke(self) -> eyre::Result<CliOutput> {
        let snapshot = OfflineSnapshot::load(&self.snapshot)?;
        let record = DnsRecord {
            host_id: None,
            host: self.host,
            record_type: self.record_type,
            address: self.address,
            ttl: self.ttl,
            mx_pref: None,
        };
        Ok(CliOutput::facet(RecordPlan::build(
            snapshot.zone,
            RecordChange::Add(record),
        )?))
    }
}
