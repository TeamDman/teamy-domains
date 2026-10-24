/// Credential-free provider parameters for a fixed method. Broker adds auth internally later.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapRequestFields {
    method: &'static str,
    params: Vec<(String, String)>,
}

impl std::fmt::Debug for NamecheapRequestFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapRequestFields([redacted])")
    }
}

impl NamecheapRequestFields {
    pub(crate) fn new(method: &'static str, params: Vec<(String, String)>) -> Self {
        Self { method, params }
    }
    #[must_use]
    pub fn method(&self) -> &'static str {
        self.method
    }
    #[must_use]
    pub fn params(&self) -> &[(String, String)] {
        &self.params
    }
}
