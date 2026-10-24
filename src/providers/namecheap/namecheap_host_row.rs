use super::{NamecheapHostId, NamecheapXmlAttribute};

/// Known `Host` fields plus every unrecognized attribute from `getHosts`.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapHostRow {
    pub host_id: NamecheapHostId,
    pub name: String,
    pub record_type: String,
    pub address: String,
    pub mx_pref: Option<String>,
    pub ttl: Option<String>,
    pub extra_attributes: Vec<NamecheapXmlAttribute>,
}

impl std::fmt::Debug for NamecheapHostRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapHostRow([redacted])")
    }
}
