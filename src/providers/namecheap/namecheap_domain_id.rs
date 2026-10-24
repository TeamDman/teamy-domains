use super::NamecheapAdapterError;

/// Explicit SLD and TLD from the registrar API. The TLD can contain labels (e.g. co.uk).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamecheapDomainId {
    sld: String,
    tld: String,
}

impl NamecheapDomainId {
    /// # Errors
    /// Rejects labels that cannot be safely passed as Namecheap SLD/TLD parameters.
    pub fn new(sld: &str, tld: &str) -> Result<Self, NamecheapAdapterError> {
        if sld.is_empty()
            || sld.len() > 70
            || sld.contains('.')
            || !valid_label(sld)
            || tld.is_empty()
            || tld.len() > 10
            || !tld.split('.').all(valid_label)
        {
            return Err(NamecheapAdapterError::InvalidDomain);
        }
        Ok(Self {
            sld: sld.to_ascii_lowercase(),
            tld: tld.to_ascii_lowercase(),
        })
    }

    #[must_use]
    pub fn sld(&self) -> &str {
        &self.sld
    }

    #[must_use]
    pub fn tld(&self) -> &str {
        &self.tld
    }

    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.sld, self.tld)
    }
}

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        && label
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && label
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_explicit_tld_boundaries_and_accepts_punycode() {
        let id = NamecheapDomainId::new("XN--BCHER-KVA", "Co.Uk").unwrap();
        assert_eq!(id.full_name(), "xn--bcher-kva.co.uk");
        assert_eq!(
            NamecheapDomainId::new("example.co", "uk"),
            Err(NamecheapAdapterError::InvalidDomain)
        );
        assert_eq!(
            NamecheapDomainId::new("-bad", "com"),
            Err(NamecheapAdapterError::InvalidDomain)
        );
    }
}
