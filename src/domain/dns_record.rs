use eyre::Context;
use facet::Facet;

/// A host row from a complete offline snapshot. Unknown types remain visible.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct DnsRecord {
    pub host_id: Option<String>,
    pub host: String,
    pub record_type: String,
    pub address: String,
    pub ttl: u32,
    pub mx_pref: Option<u16>,
}

impl DnsRecord {
    /// # Errors
    /// Returns an error for a record the first offline mutation slice cannot safely target.
    pub fn validate_target(&self) -> eyre::Result<()> {
        eyre::ensure!(
            matches!(self.record_type.as_str(), "A" | "AAAA" | "CNAME"),
            "record type is not enabled for mutation"
        );
        eyre::ensure!(
            !self.host.is_empty() && !self.host.chars().any(char::is_control),
            "invalid host"
        );
        eyre::ensure!(
            !self.address.is_empty() && !self.address.chars().any(char::is_control),
            "invalid address"
        );
        eyre::ensure!(
            (60..=60_000).contains(&self.ttl),
            "TTL outside Namecheap range"
        );
        eyre::ensure!(
            self.mx_pref.is_none(),
            "MX preference is not valid for this record type"
        );
        match self.record_type.as_str() {
            "A" => {
                self.address
                    .parse::<std::net::Ipv4Addr>()
                    .wrap_err("invalid IPv4 address")?;
            }
            "AAAA" => {
                self.address
                    .parse::<std::net::Ipv6Addr>()
                    .wrap_err("invalid IPv6 address")?;
            }
            "CNAME" => {
                eyre::ensure!(
                    self.address.contains('.') && !self.address.chars().any(char::is_whitespace),
                    "invalid CNAME target"
                );
            }
            _ => unreachable!("record type was checked above"),
        }
        Ok(())
    }
}
