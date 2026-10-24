use super::{NamecheapAdapterError, NamecheapDomainId, NamecheapRequestFields};

/// Explicit allowlist of read methods; no caller-provided command or endpoint.
#[derive(Clone, PartialEq, Eq)]
pub enum NamecheapRequest {
    ListDomains { page: u32, page_size: u8 },
    DomainInfo(NamecheapDomainId),
    Nameservers(NamecheapDomainId),
    HostRecords(NamecheapDomainId),
}

impl std::fmt::Debug for NamecheapRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapRequest([redacted])")
    }
}

impl NamecheapRequest {
    /// # Errors
    /// Rejects invalid paging and never includes API authentication fields.
    pub fn fields(&self) -> Result<NamecheapRequestFields, NamecheapAdapterError> {
        let (method, fields) = match self {
            Self::ListDomains { page, page_size } => {
                if *page == 0 || !(10..=100).contains(page_size) {
                    return Err(NamecheapAdapterError::InvalidRequest);
                }
                (
                    "namecheap.domains.getList",
                    vec![
                        ("Page".into(), page.to_string()),
                        ("PageSize".into(), page_size.to_string()),
                    ],
                )
            }
            Self::DomainInfo(domain) => {
                let full_name = domain.full_name();
                if full_name.len() > 70 {
                    return Err(NamecheapAdapterError::InvalidDomain);
                }
                (
                    "namecheap.domains.getInfo",
                    vec![("DomainName".into(), full_name)],
                )
            }
            Self::Nameservers(domain) => ("namecheap.domains.dns.getList", domain_fields(domain)),
            Self::HostRecords(domain) => ("namecheap.domains.dns.getHosts", domain_fields(domain)),
        };
        Ok(NamecheapRequestFields::new(method, fields))
    }
}

fn domain_fields(domain: &NamecheapDomainId) -> Vec<(String, String)> {
    vec![
        ("SLD".into(), domain.sld().into()),
        ("TLD".into(), domain.tld().into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_allowlist_matches_documented_read_parameters_without_secrets() {
        let id = NamecheapDomainId::new("example", "co.uk").unwrap();
        let hosts = NamecheapRequest::HostRecords(id.clone()).fields().unwrap();
        assert_eq!(hosts.method(), "namecheap.domains.dns.getHosts");
        assert_eq!(
            hosts.params(),
            &[
                ("SLD".into(), "example".into()),
                ("TLD".into(), "co.uk".into())
            ]
        );
        let info = NamecheapRequest::DomainInfo(id).fields().unwrap();
        assert_eq!(
            info.params(),
            &[("DomainName".into(), "example.co.uk".into())]
        );
        for key in ["ApiKey", "ApiUser", "UserName", "ClientIp"] {
            assert!(!hosts.params().iter().any(|(name, _)| name == key));
        }
    }

    #[test]
    fn list_paging_stays_in_documented_range() {
        assert_eq!(
            NamecheapRequest::ListDomains {
                page: 0,
                page_size: 20
            }
            .fields(),
            Err(NamecheapAdapterError::InvalidRequest)
        );
        assert_eq!(
            NamecheapRequest::ListDomains {
                page: 1,
                page_size: 9
            }
            .fields(),
            Err(NamecheapAdapterError::InvalidRequest)
        );
        let request = NamecheapRequest::ListDomains {
            page: 2,
            page_size: 100,
        }
        .fields()
        .unwrap();
        assert_eq!(
            request.params(),
            &[
                ("Page".into(), "2".into()),
                ("PageSize".into(), "100".into())
            ]
        );
    }
}
