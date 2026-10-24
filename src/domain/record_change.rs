use super::DnsRecord;
use facet::Facet;

#[derive(Facet, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordChange {
    Add(DnsRecord),
    Update {
        host_id: String,
        replacement: DnsRecord,
    },
    Delete {
        host_id: String,
    },
}
